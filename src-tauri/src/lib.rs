mod win32;

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{Emitter, Manager};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::menu::{Menu, MenuItem};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppStateData {
    pub is_running: bool,
    pub interval_minutes: u64,
    pub target_processes: Vec<String>,
    pub last_saved_time: Option<String>,
}

impl Default for AppStateData {
    fn default() -> Self {
        Self {
            is_running: false,
            interval_minutes: 1,
            target_processes: vec!["blender.exe".to_string(), "krita.exe".to_string()],
            last_saved_time: None,
        }
    }
}

pub struct AppState(pub Arc<Mutex<AppStateData>>);

#[tauri::command]
fn start_autosaver(
    interval: u64,
    processes: Vec<String>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut data = state.0.lock().map_err(|e| e.to_string())?;
    data.is_running = true;
    data.interval_minutes = interval;
    data.target_processes = processes;
    Ok(())
}

#[tauri::command]
fn stop_autosaver(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut data = state.0.lock().map_err(|e| e.to_string())?;
    data.is_running = false;
    Ok(())
}

#[tauri::command]
fn get_status(state: tauri::State<'_, AppState>) -> Result<AppStateData, String> {
    let data = state.0.lock().map_err(|e| e.to_string())?;
    Ok(data.clone())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = Arc::new(Mutex::new(AppStateData::default()));
    let app_state_clone = app_state.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState(app_state.clone()))
        .invoke_handler(tauri::generate_handler![
            start_autosaver,
            stop_autosaver,
            get_status
        ])
        .setup(|app| {
            let handle = app.handle().clone();

            // Setup tray menu
            let quit_i = MenuItem::with_id(app, "quit", "Salir", true, None::<&str>)?;
            let toggle_i = MenuItem::with_id(app, "toggle", "Abrir UI", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&toggle_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "toggle" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .icon(app.default_window_icon().unwrap().clone())
                .build(app)?;

            // Background timer thread
            let app_handle = handle.clone();
            thread::spawn(move || {
                let mut last_save_time = std::time::Instant::now();

                loop {
                    thread::sleep(Duration::from_secs(1));

                    let data = {
                        let lock = app_state_clone.lock().unwrap();
                        lock.clone()
                    };

                    if data.is_running {
                        let elapsed = last_save_time.elapsed().as_secs();
                        if elapsed >= (data.interval_minutes * 60) {
                            if let Some(active_process) = win32::get_active_process_name() {
                                if data.target_processes.contains(&active_process) {
                                    win32::send_ctrl_s();
                                    last_save_time = std::time::Instant::now();
                                    
                                    let now_str = chrono::Local::now().format("%H:%M:%S").to_string();
                                    
                                    // Update state
                                    if let Ok(mut lock) = app_state_clone.lock() {
                                        lock.last_saved_time = Some(format!("Guardado a las {} en {}", now_str, active_process));
                                    }

                                    // Emit event to frontend
                                    #[derive(Clone, Serialize)]
                                    struct SaveEvent {
                                        time: String,
                                        process: String,
                                    }
                                    
                                    let _ = app_handle.emit("saved_event", SaveEvent {
                                        time: now_str,
                                        process: active_process,
                                    });
                                }
                            }
                        }
                    } else {
                        // Reset timer when paused so it doesn't immediately trigger when unpaused
                        last_save_time = std::time::Instant::now();
                    }
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                window.hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
