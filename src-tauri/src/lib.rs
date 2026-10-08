#[cfg(not(windows))]
compile_error!("Don't Crash Now solo es compatible con Windows.");

mod actions;
mod backup;
mod commands;
mod config;
mod presets;
mod scheduler;
mod shortcut;
mod state;
mod texts;
mod tray;
mod win32;

use std::sync::Arc;

use tauri::Manager;
use backup::Backups;
use state::Shared;
use tauri_plugin_global_shortcut::ShortcutState;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        actions::toggle_enabled(app);
                    }
                })
                .build(),
        )
        .setup(|app| {
            let config_path = app.path().app_config_dir()?.join("config.json");
            let shared = Arc::new(Shared::load(config_path));
            app.manage(shared.clone());

            let data_dir = app.path().app_data_dir()?;
            let backups = Backups::start(app.handle().clone(), shared.clone(), data_dir);
            app.manage(backups.clone());

            let config = shared.config();
            tray::create(app.handle(), &config)?;
            actions::register_hotkey(app.handle(), &config.toggle_hotkey);
            scheduler::spawn(app.handle().clone(), shared, backups);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::set_config,
            commands::get_presets,
            commands::list_open_apps,
            commands::pause,
            commands::resume,
            commands::list_backups,
            commands::restore_backup,
            commands::reveal_backup,
            commands::delete_backup,
            commands::delete_backup_group,
            commands::backup_root,
            commands::open_backup_root,
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

