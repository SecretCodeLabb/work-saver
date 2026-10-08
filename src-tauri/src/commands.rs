//! Comandos que la interfaz invoca por IPC.

use std::sync::Arc;

use tauri::State;

use crate::config::Config;
use crate::state::{SaveInfo, Shared};

type SharedState<'a> = State<'a, Arc<Shared>>;

#[tauri::command]
pub fn get_config(shared: SharedState) -> Config {
    shared.config()
}

/// Reemplaza la configuración y devuelve la versión normalizada.
#[tauri::command]
pub fn set_config(config: Config, shared: SharedState) -> Config {
    let config = config.normalized();
    shared.set_config(config.clone());
    config
}

#[tauri::command]
pub fn get_last_save(shared: SharedState) -> Option<SaveInfo> {
    shared.runtime().last_save.clone()
}
