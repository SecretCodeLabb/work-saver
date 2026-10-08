//! Icono y menú de la bandeja del sistema.
//!
//! El icono lleva un punto de color según el estado: verde (activo), ámbar (en pausa),
//! rojo (hay un problema) o sin punto (apagado).

use std::sync::Mutex;

use tauri::image::Image;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Wry};

use crate::actions;
use crate::config::Config;
use crate::state::lock;
use crate::texts::{tr, Text};

const TRAY_ID: &str = "main";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayState {
    Off,
    Active,
    Paused,
    Problem,
}

struct TrayHandles {
    enabled: CheckMenuItem<Wry>,
    resume: MenuItem<Wry>,
    icons: [Image<'static>; 4],
    /// Último estado y texto aplicados, para no redibujar en cada ciclo.
    last: Mutex<Option<(TrayState, String)>>,
}

pub fn create(app: &AppHandle, config: &Config) -> tauri::Result<()> {
    let item = |id: &str, text: Text| MenuItem::with_id(app, id, tr(text, &[]), true, None::<&str>);
    let enabled = CheckMenuItem::with_id(app, "enabled", tr(Text::TrayEnabled, &[]), true, config.enabled, None::<&str>)?;
    let resume = item("resume", Text::TrayResume)?;
    let pause = Submenu::with_items(
        app,
        tr(Text::TrayPause, &[]),
        true,
        &[
            &item("pause15", Text::TrayPause15)?,
            &item("pause30", Text::TrayPause30)?,
            &item("pause60", Text::TrayPause60)?,
            &PredefinedMenuItem::separator(app)?,
            &resume,
        ],
    )?;
    let menu = Menu::with_items(
        app,
        &[
            &item("open", Text::TrayOpen)?,
            &PredefinedMenuItem::separator(app)?,
            &enabled,
            &pause,
            &PredefinedMenuItem::separator(app)?,
            &item("quit", Text::TrayQuit)?,
        ],
    )?;

    let base = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| tauri::Error::AssetNotFound("icono de la app".into()))?;
    let icons = [
        base.clone().to_owned(),
        with_dot(&base, [52, 211, 153]),
        with_dot(&base, [251, 191, 36]),
        with_dot(&base, [248, 113, 113]),
    ];

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icons[0].clone())
        .tooltip("Don't Crash Now")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => show_main_window(app),
            "enabled" => {
                let enabled = app.state::<TrayHandles>().enabled.is_checked().unwrap_or(false);
                actions::set_enabled(app, enabled);
            }
            "pause15" => actions::pause(app, 15),
            "pause30" => actions::pause(app, 30),
            "pause60" => actions::pause(app, 60),
            "resume" => actions::resume(app),
            "quit" => {
                if let Some(activity) = app.try_state::<std::sync::Arc<crate::activity::Activity>>() {
                    activity.flush();
                }
                app.exit(0);
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
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    let _ = resume.set_enabled(config.paused_until.is_some());
    app.manage(TrayHandles {
        enabled,
        resume,
        icons,
        last: Mutex::new(None),
    });
    Ok(())
}

/// Refleja la configuración en el menú (casilla de activado, opción de reanudar).
pub fn sync_menu(app: &AppHandle, config: &Config) {
    if let Some(handles) = app.try_state::<TrayHandles>() {
        let _ = handles.enabled.set_checked(config.enabled);
        let _ = handles.resume.set_enabled(config.paused_until.is_some());
    }
}

/// Actualiza icono y texto emergente si cambiaron.
pub fn update(app: &AppHandle, state: TrayState, tooltip: String) {
    let Some(handles) = app.try_state::<TrayHandles>() else {
        return;
    };
    let mut last = lock(&handles.last);
    if last.as_ref().is_some_and(|(s, t)| *s == state && *t == tooltip) {
        return;
    }
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let index = match state {
            TrayState::Off => 0,
            TrayState::Active => 1,
            TrayState::Paused => 2,
            TrayState::Problem => 3,
        };
        let _ = tray.set_icon(Some(handles.icons[index].clone()));
        let _ = tray.set_tooltip(Some(&tooltip));
    }
    *last = Some((state, tooltip));
}

pub fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Copia del icono con un punto de color en la esquina inferior derecha.
fn with_dot(base: &Image<'_>, color: [u8; 3]) -> Image<'static> {
    let (w, h) = (base.width() as i32, base.height() as i32);
    let mut rgba = base.rgba().to_vec();
    let radius = (w.min(h) as f32 * 0.24).max(3.0);
    let border = (radius * 0.28).max(1.0);
    let (cx, cy) = (w as f32 - radius - 1.0, h as f32 - radius - 1.0);

    for y in 0..h {
        for x in 0..w {
            let d = ((x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2)).sqrt();
            if d > radius + border {
                continue;
            }
            let pixel = [color[0], color[1], color[2], 255];
            let pixel = if d > radius { [0, 0, 0, 255] } else { pixel };
            let i = ((y * w + x) * 4) as usize;
            rgba[i..i + 4].copy_from_slice(&pixel);
        }
    }
    Image::new_owned(rgba, w as u32, h as u32)
}
