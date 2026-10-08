//! Hilo de autoguardado: decide cuándo enviar el atajo de guardado de cada programa.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::config::{Config, Profile};
use crate::shortcut;
use crate::state::{ForegroundInfo, ProfileStatus, SaveInfo, Shared, Status};
use crate::win32::{self, WindowInfo};

const TICK: Duration = Duration::from_secs(1);

/// Por qué un guardado pendiente todavía no se hizo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    /// El usuario está dibujando o escribiendo.
    Busy,
    /// Hay un botón del ratón, modificador o espacio presionado.
    InputHeld,
    /// Hay un cuadro de diálogo en primer plano.
    Dialog,
    /// El documento nunca se guardó (abriría «Guardar como»).
    Untitled,
    /// El programa corre como administrador y Don't Crash Now no.
    Elevated,
    /// El título indica que no hay cambios: se omitió el guardado.
    NoChanges,
}

enum Decision {
    Save,
    Wait(Reason),
    Skip(Reason),
}

pub fn spawn(app: AppHandle, shared: Arc<Shared>) {
    thread::spawn(move || {
        let mut scheduler = Scheduler {
            self_elevated: win32::is_elevated(None),
            ..Default::default()
        };
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
    /// Desde cuándo espera el guardado pendiente del programa activo.
    due_since: Option<Instant>,
    /// Último motivo de espera u omisión de cada perfil.
    reasons: HashMap<String, Reason>,
    /// Perfiles cuyo título ya mostró la marca de cambios: a partir de ahí se confía en ella.
    dirty_seen: HashSet<String>,
    self_elevated: bool,
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
                self.reasons.remove(&profile.id);
            }
        }

        let active = foreground.and_then(|w| config.profile_for(&w.exe).map(|p| (w, p)));
        if let Some((window, profile)) = active {
            if profile.is_dirty(&window.title) {
                self.dirty_seen.insert(profile.id.clone());
            }
        }

        match active {
            Some((window, profile)) if config.enabled && self.is_due(profile) => {
                let due_since = *self.due_since.get_or_insert(now);
                match self.decide(config, profile, window, due_since.elapsed()) {
                    Decision::Save => {
                        if save(window, profile) {
                            self.reset(profile);
                            let info = SaveInfo {
                                time: chrono::Local::now().format("%H:%M:%S").to_string(),
                                app: profile.name.clone(),
                                profile_id: profile.id.clone(),
                            };
                            shared.runtime().last_save = Some(info.clone());
                            let _ = app.emit("saved", info);
                        }
                    }
                    Decision::Skip(reason) => {
                        self.reset(profile);
                        self.reasons.insert(profile.id.clone(), reason);
                    }
                    Decision::Wait(reason) => {
                        self.reasons.insert(profile.id.clone(), reason);
                    }
                }
            }
            _ => self.due_since = None,
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
                    reason: self.reasons.get(&p.id).copied(),
                })
                .collect(),
            last_save: shared.runtime().last_save.clone(),
        };
        let _ = app.emit("status", status);
    }

    fn decide(&self, config: &Config, profile: &Profile, window: &WindowInfo, waited: Duration) -> Decision {
        let smart = &config.smart;
        let title = &window.title;

        if !self.self_elevated && win32::is_elevated(Some(window.pid)) {
            return Decision::Wait(Reason::Elevated);
        }
        if smart.skip_untitled && profile.is_untitled(title) {
            return Decision::Wait(Reason::Untitled);
        }
        // Solo se confía en la marca de cambios si alguna vez apareció en este programa.
        if smart.only_when_dirty && self.dirty_seen.contains(&profile.id) && !profile.is_dirty(title) {
            return Decision::Skip(Reason::NoChanges);
        }
        if smart.skip_dialogs && win32::is_dialog(window.hwnd) {
            return Decision::Wait(Reason::Dialog);
        }
        // Nunca se guarda con teclas pulsadas: cambiarían el atajo (Ctrl+Shift+S = «Guardar como»).
        if win32::input_held() {
            return Decision::Wait(Reason::InputHeld);
        }
        let max_wait = Duration::from_secs(u64::from(smart.max_wait_seconds));
        if smart.wait_for_idle
            && waited < max_wait
            && win32::idle_millis() < u64::from(smart.idle_seconds) * 1000
        {
            return Decision::Wait(Reason::Busy);
        }
        Decision::Save
    }

    fn reset(&mut self, profile: &Profile) {
        self.timers.insert(profile.id.clone(), Instant::now());
        self.reasons.remove(&profile.id);
        self.due_since = None;
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
