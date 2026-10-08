//! Hilo de autoguardado: decide cuándo enviar el atajo de guardado de cada programa.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::actions;
use crate::activity::{self, Activity, Kind};
use crate::backup::Backups;
use crate::config::{Config, Profile};
use crate::shortcut;
use crate::state::{ForegroundInfo, ProfileStatus, SaveInfo, Shared, Status};
use crate::texts::{tr, Text};
use crate::tray::{self, TrayState};
use crate::win32::{self, WindowInfo};

const TICK: Duration = Duration::from_secs(1);

/// Diagnóstico: con la variable de entorno `DCN_TRACE` se imprime cada ciclo por consola.
fn tracing() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("DCN_TRACE").is_some())
}

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
    /// Cuenta atrás del aviso previo al guardado.
    Countdown,
}

enum Decision {
    Save,
    Wait(Reason),
    Skip(Reason),
}

pub fn spawn(app: AppHandle, shared: Arc<Shared>, backups: Arc<Backups>) {
    thread::spawn(move || {
        let mut scheduler = Scheduler {
            backups,
            self_elevated: win32::is_elevated(None),
            timers: HashMap::new(),
            due_since: None,
            reasons: HashMap::new(),
            dirty_seen: HashSet::new(),
            warned_at: None,
            notified: HashSet::new(),
        };
        loop {
            thread::sleep(TICK);
            let config = shared.config();
            if actions::resume_if_expired(&app, &config) {
                continue;
            }
            let foreground = win32::foreground_window();
            scheduler.tick(&app, &shared, &config, foreground.as_ref());
        }
    });
}

struct Scheduler {
    backups: Arc<Backups>,
    /// Momento del último guardado (o del último reinicio) de cada perfil.
    timers: HashMap<String, Instant>,
    /// Desde cuándo espera el guardado pendiente del programa activo.
    due_since: Option<Instant>,
    /// Último motivo de espera u omisión de cada perfil.
    reasons: HashMap<String, Reason>,
    /// Perfiles cuyo título ya mostró la marca de cambios: a partir de ahí se confía en ella.
    dirty_seen: HashSet<String>,
    self_elevated: bool,
    /// Momento en que se mostró el aviso previo del guardado pendiente.
    warned_at: Option<Instant>,
    /// Problemas ya notificados en el ciclo actual ("perfil:motivo").
    notified: HashSet<String>,
}

impl Scheduler {
    fn tick(&mut self, app: &AppHandle, shared: &Shared, config: &Config, foreground: Option<&WindowInfo>) {
        let now = Instant::now();
        self.timers.retain(|id, _| config.profiles.iter().any(|p| &p.id == id));
        for profile in &config.profiles {
            let timer = self.timers.entry(profile.id.clone()).or_insert(now);
            if !config.is_running() || !profile.enabled {
                // En pausa: reiniciar para no guardar de golpe al reactivar.
                *timer = now;
                self.reasons.remove(&profile.id);
            }
        }

        let active = foreground.and_then(|w| config.profile_for(&w.exe).map(|p| (w, p)));
        if tracing() {
            eprintln!(
                "[trace] fg={:?} active={:?} running={} due={:?}",
                foreground.map(|w| (&w.exe, &w.title, w.hwnd)),
                active.map(|(_, p)| &p.name),
                config.is_running(),
                config.profiles.iter().map(|p| (&p.exe, self.remaining(p).as_secs())).collect::<Vec<_>>()
            );
        }
        if let Some((window, profile)) = active {
            if profile.is_dirty(&window.title) {
                self.dirty_seen.insert(profile.id.clone());
            }
        }

        match active {
            Some((window, profile)) if config.is_running() && self.is_due(profile) => {
                let due_since = *self.due_since.get_or_insert(now);
                let mut decision = self.decide(config, profile, window, due_since.elapsed());
                if matches!(decision, Decision::Save) && config.notifications.warn_before {
                    let seconds = config.notifications.warn_seconds;
                    match self.warned_at {
                        None => {
                            self.warned_at = Some(now);
                            actions::notify(
                                app,
                                &tr(Text::NotifyWarnTitle, &[("app", &profile.name), ("seconds", &seconds.to_string())]),
                                &tr(Text::NotifyWarnBody, &[]),
                            );
                            decision = Decision::Wait(Reason::Countdown);
                        }
                        Some(at) if at.elapsed() < Duration::from_secs(u64::from(seconds)) => {
                            decision = Decision::Wait(Reason::Countdown);
                        }
                        Some(_) => {}
                    }
                }
                if tracing() {
                    eprintln!("[trace] decision={}", match &decision {
                        Decision::Save => "save".to_string(),
                        Decision::Wait(r) => format!("wait {r:?}"),
                        Decision::Skip(r) => format!("skip {r:?}"),
                    });
                }
                match decision {
                    Decision::Save => {
                        let saved = save(window, profile);
                        if tracing() {
                            eprintln!("[trace] save sent={saved}");
                        }
                        if saved {
                            let dirty = self.dirty_seen.contains(&profile.id) && profile.is_dirty(&window.title);
                            self.backups.expect_save(profile, dirty);
                            self.reset(profile);
                            let info = SaveInfo {
                                time: chrono::Local::now().format("%H:%M:%S").to_string(),
                                app: profile.name.clone(),
                                profile_id: profile.id.clone(),
                                verified: None,
                                file: None,
                            };
                            shared.runtime().last_save = Some(info.clone());
                            if config.notifications.on_save {
                                actions::notify(
                                    app,
                                    &tr(Text::NotifySavedTitle, &[("app", &info.app)]),
                                    &tr(Text::NotifySavedBody, &[("time", &info.time)]),
                                );
                            }
                            activity::record(app, Kind::Save, &info.app, &window.title);
                            let _ = app.emit("saved", info);
                        }
                    }
                    Decision::Skip(reason) => {
                        activity::record(app, Kind::Skip, &profile.name, "");
                        self.reset(profile);
                        self.reasons.insert(profile.id.clone(), reason);
                    }
                    Decision::Wait(reason) => {
                        self.reasons.insert(profile.id.clone(), reason);
                        if config.notifications.on_problem {
                            self.notify_problem(app, profile, reason);
                        }
                    }
                }
            }
            _ => {
                self.due_since = None;
                self.warned_at = None;
            }
        }
        if config.is_running() && active.is_some() {
            if let Some(activity) = app.try_state::<Arc<Activity>>() {
                activity.add_protected_seconds(TICK.as_secs());
            }
        }
        let (last_save, crash_alert) = shared.runtime_snapshot();
        self.update_tray(app, config, crash_alert.is_some());

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
            last_save,
            crash_alert,
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
        self.warned_at = None;
        let prefix = format!("{}:", profile.id);
        self.notified.retain(|key| !key.starts_with(&prefix));
    }

    /// Avisa una sola vez por ciclo de los problemas que requieren al usuario.
    fn notify_problem(&mut self, app: &AppHandle, profile: &Profile, reason: Reason) {
        let (title, body) = match reason {
            Reason::Untitled => (Text::NotifyUntitledTitle, Text::NotifyUntitledBody),
            Reason::Elevated => (Text::NotifyElevatedTitle, Text::NotifyElevatedBody),
            _ => return,
        };
        if self.notified.insert(format!("{}:{:?}", profile.id, reason)) {
            let title = tr(title, &[("app", &profile.name)]);
            activity::record(app, Kind::Problem, &profile.name, &title);
            actions::notify(app, &title, &tr(body, &[]));
        }
    }

    fn update_tray(&self, app: &AppHandle, config: &Config, crashed: bool) {
        let problem = crashed
            || self
            .reasons
            .values()
            .any(|r| matches!(r, Reason::Untitled | Reason::Elevated));
        let (state, status) = if !config.enabled {
            (TrayState::Off, tr(Text::StatusOff, &[]))
        } else if let Some(until) = config.paused_until {
            let time = chrono::DateTime::from_timestamp_millis(until)
                .map(|d| d.with_timezone(&chrono::Local).format("%H:%M").to_string())
                .unwrap_or_default();
            (TrayState::Paused, tr(Text::StatusPausedUntil, &[("time", &time)]))
        } else if problem {
            (TrayState::Problem, tr(Text::StatusActive, &[]))
        } else {
            (TrayState::Active, tr(Text::StatusActive, &[]))
        };

        let mut tooltip = format!("Don't Crash Now\n{status}");
        if config.is_running() {
            let next = config
                .profiles
                .iter()
                .filter(|p| p.enabled)
                .min_by_key(|p| self.remaining(p));
            if let Some(profile) = next {
                let minutes = self.remaining(profile).as_secs().div_ceil(60).to_string();
                tooltip.push('\n');
                tooltip.push_str(&tr(Text::StatusNextSave, &[("app", &profile.name), ("minutes", &minutes)]));
            }
        }
        tray::update(app, state, tooltip.chars().take(120).collect());
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
