//! Hilo de autoguardado: decide cuándo enviar el atajo de guardado de cada programa.

use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter};

use crate::config::{Config, Profile};
use crate::shortcut;
use crate::state::{ForegroundInfo, ProfileStatus, SaveInfo, Shared, Status};
use crate::win32::{self, WindowInfo};

const TICK: Duration = Duration::from_secs(1);

pub fn spawn(app: AppHandle, shared: Arc<Shared>) {
    thread::spawn(move || {
        let mut scheduler = Scheduler::default();
        loop {
            thread::sleep(TICK);
            let config = shared.config();
            let foreground = win32::foreground_window();
            scheduler.tick(&app, &shared, &config, foreground.as_ref());
        }
    });
}

#[derive(Default)]
struct Scheduler {
    /// Momento del último guardado (o del último reinicio) de cada perfil.
    timers: HashMap<String, Instant>,
}

impl Scheduler {
    fn tick(&mut self, app: &AppHandle, shared: &Shared, config: &Config, foreground: Option<&WindowInfo>) {
        let now = Instant::now();
        self.timers.retain(|id, _| config.profiles.iter().any(|p| &p.id == id));
        for profile in &config.profiles {
            let timer = self.timers.entry(profile.id.clone()).or_insert(now);
            if !config.enabled || !profile.enabled {
                // En pausa: reiniciar para no guardar de golpe al reactivar.
                *timer = now;
            }
        }

        let active = foreground.and_then(|w| config.profile_for(&w.exe).map(|p| (w, p)));
        if config.enabled {
            if let Some((window, profile)) = active {
                if self.is_due(profile) && save(window, profile) {
                    self.timers.insert(profile.id.clone(), Instant::now());
                    let info = SaveInfo {
                        time: chrono::Local::now().format("%H:%M:%S").to_string(),
                        app: profile.name.clone(),
                        profile_id: profile.id.clone(),
                    };
                    shared.runtime().last_save = Some(info.clone());
                    let _ = app.emit("saved", info);
                }
            }
        }

        let status = Status {
            enabled: config.enabled,
            foreground: foreground.map(|w| ForegroundInfo {
                exe: w.exe.clone(),
                title: w.title.clone(),
            }),
            active_profile: active.map(|(_, p)| p.id.clone()),
            profiles: config
                .profiles
                .iter()
                .filter(|p| p.enabled)
                .map(|p| ProfileStatus {
                    id: p.id.clone(),
                    next_save_in: self.remaining(p).as_secs(),
                })
                .collect(),
            last_save: shared.runtime().last_save.clone(),
        };
        let _ = app.emit("status", status);
    }

    fn remaining(&self, profile: &Profile) -> Duration {
        let interval = Duration::from_secs(u64::from(profile.interval_minutes) * 60);
        let elapsed = self.timers.get(&profile.id).map(Instant::elapsed).unwrap_or_default();
        interval.saturating_sub(elapsed)
    }

    fn is_due(&self, profile: &Profile) -> bool {
        self.remaining(profile).is_zero()
    }
}

/// Envía el atajo del perfil si la ventana sigue en primer plano.
fn save(window: &WindowInfo, profile: &Profile) -> bool {
    let Some(keys) = shortcut::parse(&profile.shortcut) else {
        return false;
    };
    win32::is_foreground(window.hwnd) && win32::send_keys(&keys)
}
