//! Detección de cierres inesperados de los programas vigilados.
//!
//! Se mantiene abierto un identificador de cada proceso vigilado para poder leer su
//! código de salida cuando termina. Un código distinto de 0 en un programa que el
//! usuario estaba usando se considera un cierre inesperado.

use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::actions;
use crate::activity::{self, Kind};
use crate::backup::{BackupEntry, Backups};
use crate::state::{SaveInfo, Shared};
use crate::texts::{tr, Text};
use crate::win32::{self, ProcessHandle};

const POLL: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Serialize)]
pub struct CrashAlert {
    pub profile_id: String,
    pub app: String,
    /// Hora local "HH:MM".
    pub time: String,
    /// Código de salida en hexadecimal (ej. "0xC0000005").
    pub code: String,
    pub last_save: Option<SaveInfo>,
    pub last_backup: Option<BackupEntry>,
}

struct Tracked {
    handle: ProcessHandle,
    profile_id: String,
    /// El usuario tuvo el programa en primer plano en algún momento.
    used: bool,
}

pub fn spawn(app: AppHandle, shared: Arc<Shared>, backups: Arc<Backups>) {
    thread::spawn(move || {
        let mut tracked: HashMap<u32, Tracked> = HashMap::new();
        loop {
            thread::sleep(POLL);
            let config = shared.config();

            // Nuevos procesos de los programas vigilados.
            for (pid, exe) in win32::list_processes() {
                if tracked.contains_key(&pid) {
                    continue;
                }
                let Some(profile) = config.profile_for(&exe) else { continue };
                if let Some(handle) = ProcessHandle::open(pid) {
                    tracked.insert(
                        pid,
                        Tracked {
                            handle,
                            profile_id: profile.id.clone(),
                            used: false,
                        },
                    );
                }
            }

            if let Some(window) = win32::foreground_window() {
                if let Some(t) = tracked.get_mut(&window.pid) {
                    t.used = true;
                }
            }

            // Procesos que terminaron.
            let exited: Vec<(u32, u32)> = tracked
                .iter()
                .filter_map(|(pid, t)| t.handle.exit_code().map(|code| (*pid, code)))
                .collect();
            for (pid, code) in exited {
                let Some(t) = tracked.remove(&pid) else { continue };
                if code == 0 || !t.used || !config.enabled {
                    continue;
                }
                let Some(profile) = config.profiles.iter().find(|p| p.id == t.profile_id) else {
                    continue;
                };
                let last_save = shared
                    .runtime()
                    .last_save
                    .clone()
                    .filter(|s| s.profile_id == profile.id);
                let alert = CrashAlert {
                    profile_id: profile.id.clone(),
                    app: profile.name.clone(),
                    time: chrono::Local::now().format("%H:%M").to_string(),
                    code: format!("0x{code:08X}"),
                    last_backup: backups.latest_for(&profile.name),
                    last_save,
                };
                if config.notifications.on_problem {
                    let detail = match (&alert.last_save, &alert.last_backup) {
                        (Some(save), _) => tr(Text::NotifyCrashLastSave, &[("time", &save.time)]),
                        (None, Some(_)) => tr(Text::NotifyCrashBackup, &[]),
                        (None, None) => tr(Text::NotifyCrashNothing, &[]),
                    };
                    actions::notify(&app, &tr(Text::NotifyCrashTitle, &[("app", &alert.app)]), &detail);
                }
                activity::record(&app, Kind::Crash, &alert.app, &alert.code);
                shared.runtime().crash_alert = Some(alert.clone());
                let _ = app.emit("crash_detected", alert);
            }
        }
    });
}
