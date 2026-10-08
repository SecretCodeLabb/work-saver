//! Hilo de autoguardado: decide cuándo enviar el atajo de guardado.

use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter};

use crate::state::{SaveInfo, Shared, Status};
use crate::win32;

const TICK: Duration = Duration::from_secs(1);
const VK_CONTROL: u16 = 0x11;
const VK_S: u16 = 0x53;

pub fn spawn(app: AppHandle, shared: Arc<Shared>) {
    thread::spawn(move || {
        let mut last_save = Instant::now();

        loop {
            thread::sleep(TICK);

            let config = shared.config();
            let foreground = win32::foreground_window();
            let mut next_save_in = None;

            if config.enabled {
                let interval = Duration::from_secs(u64::from(config.interval_minutes) * 60);
                if last_save.elapsed() >= interval {
                    if let Some(window) = &foreground {
                        if config.processes.contains(&window.exe)
                            && win32::is_foreground(window.hwnd)
                            && win32::send_keys(&[VK_CONTROL, VK_S])
                        {
                            last_save = Instant::now();
                            let info = SaveInfo {
                                time: chrono::Local::now().format("%H:%M:%S").to_string(),
                                app: window.exe.clone(),
                            };
                            shared.runtime().last_save = Some(info.clone());
                            let _ = app.emit("saved", info);
                        }
                    }
                }
                next_save_in = Some(interval.saturating_sub(last_save.elapsed()).as_secs());
            } else {
                // En pausa: reiniciar para no guardar de golpe al reactivar.
                last_save = Instant::now();
            }

            let status = Status {
                enabled: config.enabled,
                foreground: foreground.map(|w| w.exe),
                next_save_in,
                last_save: shared.runtime().last_save.clone(),
            };
            let _ = app.emit("status", status);
        }
    });
}
