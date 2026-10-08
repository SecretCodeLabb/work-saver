//! Comandos que la interfaz invoca por IPC.

use std::sync::Arc;

use serde::Serialize;
use tauri::State;

use crate::config::Config;
use crate::presets::{Preset, PRESETS};
use crate::state::Shared;
use crate::win32;

type SharedState<'a> = State<'a, Arc<Shared>>;

#[tauri::command]
pub fn get_config(shared: SharedState) -> Config {
    shared.config()
}

/// Reemplaza la configuración, la guarda y devuelve la versión normalizada.
#[tauri::command]
pub fn set_config(config: Config, shared: SharedState) -> Result<Config, String> {
    let config = config.normalized();
    shared.set_config(config.clone()).map_err(|e| e.to_string())?;
    Ok(config)
}


#[tauri::command]
pub fn get_presets() -> Vec<Preset> {
    PRESETS.to_vec()
}

#[derive(Serialize)]
pub struct OpenApp {
    exe: String,
    title: String,
}

/// Programas con ventanas abiertas, para elegirlos sin escribir el ejecutable.
#[tauri::command]
pub fn list_open_apps() -> Vec<OpenApp> {
    const IGNORED: [&str; 4] = [
        "explorer.exe",
        "applicationframehost.exe",
        "shellexperiencehost.exe",
        "textinputhost.exe",
    ];
    let own_pid = std::process::id();
    let mut apps: Vec<OpenApp> = Vec::new();
    for window in win32::list_app_windows() {
        if window.pid == own_pid
            || IGNORED.contains(&window.exe.as_str())
            || apps.iter().any(|a| a.exe == window.exe)
        {
            continue;
        }
        apps.push(OpenApp {
            exe: window.exe,
            title: window.title,
        });
    }
    apps.sort_by(|a, b| a.exe.cmp(&b.exe));
    apps
}
