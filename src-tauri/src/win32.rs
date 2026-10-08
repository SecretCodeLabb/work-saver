//! Acceso a la API Win32: ventana activa, nombres de proceso y envío de teclas.

use std::mem::size_of;
use std::path::Path;

use windows::core::PWSTR;
use windows::Win32::Foundation::{CloseHandle, HWND};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
    VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
};

/// Información de una ventana de nivel superior.
#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub hwnd: isize,
    pub pid: u32,
    /// Nombre del ejecutable en minúsculas (ej. "blender.exe").
    pub exe: String,
    pub title: String,
}

/// Devuelve la ventana en primer plano, si existe y se puede consultar su proceso.
pub fn foreground_window() -> Option<WindowInfo> {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.0.is_null() {
        return None;
    }
    window_info(hwnd)
}

/// Indica si `hwnd` sigue siendo la ventana en primer plano.
pub fn is_foreground(hwnd: isize) -> bool {
    unsafe { GetForegroundWindow().0 as isize == hwnd }
}

fn window_info(hwnd: HWND) -> Option<WindowInfo> {
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    if pid == 0 {
        return None;
    }
    Some(WindowInfo {
        hwnd: hwnd.0 as isize,
        pid,
        exe: process_exe_name(pid)?,
        title: window_title(hwnd),
    })
}

/// Nombre del ejecutable de un proceso, en minúsculas.
pub fn process_exe_name(pid: u32) -> Option<String> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        // Rutas largas: no limitar a MAX_PATH.
        let mut buffer = vec![0u16; 4096];
        let mut len = buffer.len() as u32;
        let result = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut len,
        );
        let _ = CloseHandle(handle);
        result.ok()?;

        let path = String::from_utf16_lossy(&buffer[..len as usize]);
        Path::new(&path)
            .file_name()
            .map(|name| name.to_string_lossy().to_lowercase())
    }
}

fn window_title(hwnd: HWND) -> String {
    unsafe {
        let len = GetWindowTextLengthW(hwnd);
        if len <= 0 {
            return String::new();
        }
        let mut buffer = vec![0u16; len as usize + 1];
        let copied = GetWindowTextW(hwnd, &mut buffer);
        String::from_utf16_lossy(&buffer[..copied.max(0) as usize])
    }
}

/// Pulsa las teclas en orden y las suelta en orden inverso (ej. Ctrl↓ S↓ S↑ Ctrl↑).
/// Devuelve `false` si Windows no aceptó todos los eventos.
pub fn send_keys(keys: &[u16]) -> bool {
    let mut inputs: Vec<INPUT> = Vec::with_capacity(keys.len() * 2);
    inputs.extend(keys.iter().map(|&vk| key_input(vk, false)));
    inputs.extend(keys.iter().rev().map(|&vk| key_input(vk, true)));

    let sent = unsafe { SendInput(&inputs, size_of::<INPUT>() as i32) };
    sent as usize == inputs.len()
}

fn key_input(vk: u16, key_up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                wScan: 0,
                dwFlags: if key_up { KEYEVENTF_KEYUP } else { KEYBD_EVENT_FLAGS(0) },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}
