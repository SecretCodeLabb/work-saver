#[cfg(not(windows))]
compile_error!("Don't Crash Now solo es compatible con Windows.");

mod commands;
mod config;
mod presets;
mod scheduler;
mod shortcut;
mod state;
mod tray;
mod win32;

use std::sync::Arc;

use tauri::Manager;
use state::Shared;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let config_path = app.path().app_config_dir()?.join("config.json");
            let shared = Arc::new(Shared::load(config_path));
            app.manage(shared.clone());

            tray::create(app.handle())?;
            scheduler::spawn(app.handle().clone(), shared);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::set_config,
            commands::get_presets,
            commands::list_open_apps,
        ])
        .on_window_event(|window, event| {
            // Cerrar la ventana solo la oculta; la app sigue en la bandeja.
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error al iniciar Don't Crash Now");
}

