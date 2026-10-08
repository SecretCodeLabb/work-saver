//! Comandos que la interfaz invoca por IPC.

use std::sync::Arc;

use std::path::PathBuf;

use serde::Serialize;
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

use crate::backup::{BackupGroup, Backups};
use crate::config::Config;
use crate::presets::{Preset, PRESETS};
use crate::state::Shared;
use crate::win32;

type SharedState<'a> = State<'a, Arc<Shared>>;
type BackupState<'a> = State<'a, Arc<Backups>>;

#[tauri::command]
pub fn get_config(shared: SharedState) -> Config {
    shared.config()
}

/// Reemplaza la configuración, la guarda y devuelve la versión normalizada.
#[tauri::command]
pub fn set_config(config: Config, shared: SharedState, backups: BackupState) -> Result<Config, String> {
    let config = config.normalized();
    shared.set_config(config.clone()).map_err(|e| e.to_string())?;
    backups.refresh_watches();
    backups.apply_limits();
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

#[tauri::command]
pub fn list_backups(backups: BackupState) -> Vec<BackupGroup> {
    backups.groups()
}

/// Restaura una versión como copia junto al original y devuelve su ruta.
#[tauri::command]
pub fn restore_backup(id: String, app: AppHandle, backups: BackupState) -> Result<String, String> {
    let path = backups.restore(&id).map_err(|e| e.to_string())?;
    let _ = app.opener().reveal_item_in_dir(&path);
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn reveal_backup(id: String, app: AppHandle, backups: BackupState) -> Result<(), String> {
    let entry = backups.find(&id).ok_or("respaldo no encontrado")?;
    app.opener().reveal_item_in_dir(&entry.path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_backup(id: String, backups: BackupState) -> Result<(), String> {
    backups.delete(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_backup_group(original: PathBuf, backups: BackupState) {
    backups.delete_group(&original);
}

/// Carpeta de respaldos en uso.
#[tauri::command]
pub fn backup_root(shared: SharedState, backups: BackupState) -> String {
    backups.root(&shared.config().backups).to_string_lossy().to_string()
}

#[tauri::command]
pub fn open_backup_root(app: AppHandle, shared: SharedState, backups: BackupState) -> Result<(), String> {
    let root = backups.root(&shared.config().backups);
    std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    app.opener()
        .open_path(root.to_string_lossy(), None::<&str>)
        .map_err(|e| e.to_string())
}
