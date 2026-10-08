//! Acciones que cambian la configuración desde la bandeja, el atajo global o el propio backend,
//! manteniendo sincronizados la interfaz, la bandeja y el atajo.

use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use tauri_plugin_notification::NotificationExt;

use crate::backup::Backups;
use crate::config::Config;
use crate::state::Shared;
use crate::texts::{tr, Text};
use crate::tray;

fn shared(app: &AppHandle) -> Arc<Shared> {
    app.state::<Arc<Shared>>().inner().clone()
}

/// Guarda la configuración y aplica sus efectos (bandeja, atajo, vigilancia de carpetas).
pub fn apply(app: &AppHandle, config: Config) -> Result<Config, String> {
    let config = config.normalized();
    let previous = shared(app).config();
    shared(app).set_config(config.clone()).map_err(|e| e.to_string())?;

    if let Some(backups) = app.try_state::<Arc<Backups>>() {
        backups.refresh_watches();
        backups.apply_limits();
    }
    if previous.toggle_hotkey != config.toggle_hotkey {
        register_hotkey(app, &config.toggle_hotkey);
    }
    tray::sync_menu(app, &config);
    let _ = app.emit("config_changed", ());
    Ok(config)
}

fn update(app: &AppHandle, mutate: impl FnOnce(&mut Config)) {
    let mut config = shared(app).config();
    mutate(&mut config);
    if let Err(error) = apply(app, config) {
        eprintln!("No se pudo guardar la configuración: {error}");
    }
}

pub fn set_enabled(app: &AppHandle, enabled: bool) {
    update(app, |c| {
        c.enabled = enabled;
        c.paused_until = None;
    });
}

pub fn toggle_enabled(app: &AppHandle) {
    let enabled = !shared(app).config().enabled;
    set_enabled(app, enabled);
    notify(app, &tr(if enabled { Text::NotifyEnabled } else { Text::NotifyDisabled }, &[]), "");
}

/// Pausa el autoguardado durante unos minutos (también lo activa si estaba apagado).
pub fn pause(app: &AppHandle, minutes: u32) {
    let until = chrono::Local::now().timestamp_millis() + i64::from(minutes) * 60_000;
    update(app, |c| {
        c.enabled = true;
        c.paused_until = Some(until);
    });
}

pub fn resume(app: &AppHandle) {
    update(app, |c| {
        c.enabled = true;
        c.paused_until = None;
    });
}

/// Reanuda si la pausa ya terminó. Devuelve `true` si se reanudó.
pub fn resume_if_expired(app: &AppHandle, config: &Config) -> bool {
    let expired = config
        .paused_until
        .is_some_and(|until| chrono::Local::now().timestamp_millis() >= until);
    if expired {
        resume(app);
        notify(app, &tr(Text::NotifyResumed, &[]), "");
    }
    expired
}

pub fn register_hotkey(app: &AppHandle, hotkey: &str) {
    let shortcuts = app.global_shortcut();
    let _ = shortcuts.unregister_all();
    if hotkey.is_empty() {
        return;
    }
    let accelerator = hotkey.replace("Win", "Super");
    if let Err(error) = shortcuts.register(accelerator.as_str()) {
        eprintln!("No se pudo registrar el atajo {hotkey}: {error}");
    }
}

/// Notificación de Windows.
pub fn notify(app: &AppHandle, title: &str, body: &str) {
    let mut builder = app.notification().builder().title(title);
    if !body.is_empty() {
        builder = builder.body(body);
    }
    if let Err(error) = builder.show() {
        eprintln!("No se pudo mostrar la notificación: {error}");
    }
}
