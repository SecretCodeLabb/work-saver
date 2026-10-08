//! Acceso a la API Win32: ventana activa, nombres de proceso y envío de teclas.

use std::mem::size_of;
use std::path::Path;

use windows::core::PWSTR;
use windows::Win32::Foundation::{CloseHandle, BOOL, HANDLE, HWND, LPARAM};
use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
use windows::Win32::System::SystemInformation::GetTickCount;
use windows::Win32::System::Threading::{
    GetCurrentProcess, OpenProcess, OpenProcessToken, QueryFullProcessImageNameW,
    PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, GetLastInputInfo, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT,
    KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, LASTINPUTINFO, VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, GetForegroundWindow, GetWindow, GetWindowLongW,
    GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible, GWL_EXSTYLE,
    GW_OWNER, WS_EX_TOOLWINDOW,
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

/// Ventanas principales visibles (las que aparecen en la barra de tareas).
pub fn list_app_windows() -> Vec<WindowInfo> {
    unsafe extern "system" fn collect(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let handles = &mut *(lparam.0 as *mut Vec<HWND>);
        handles.push(hwnd);
        BOOL(1)
    }

    let mut handles: Vec<HWND> = Vec::new();
    unsafe {
        let _ = EnumWindows(Some(collect), LPARAM(&mut handles as *mut Vec<HWND> as isize));
    }

    handles
        .into_iter()
        .filter(|&hwnd| is_app_window(hwnd))
        .filter_map(window_info)
        .filter(|w| !w.title.is_empty())
        .collect()
}

fn is_app_window(hwnd: HWND) -> bool {
    unsafe {
        if !IsWindowVisible(hwnd).as_bool() {
            return false;
        }
        let has_owner = GetWindow(hwnd, GW_OWNER).map(|o| !o.0.is_null()).unwrap_or(false);
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
        !has_owner && ex_style & WS_EX_TOOLWINDOW.0 == 0
    }
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

/// Indica si la ventana es un cuadro de diálogo (ventana con dueño o clase estándar de diálogo).
pub fn is_dialog(hwnd: isize) -> bool {
    let hwnd = HWND(hwnd as *mut _);
    unsafe {
        let has_owner = GetWindow(hwnd, GW_OWNER).map(|o| !o.0.is_null()).unwrap_or(false);
        let mut class = [0u16; 64];
        let len = GetClassNameW(hwnd, &mut class).max(0) as usize;
        has_owner || String::from_utf16_lossy(&class[..len]) == "#32770"
    }
}

/// Milisegundos desde la última pulsación de tecla, movimiento de ratón o lápiz.
pub fn idle_millis() -> u64 {
    let mut info = LASTINPUTINFO {
        cbSize: size_of::<LASTINPUTINFO>() as u32,
        dwTime: 0,
    };
    unsafe {
        if !GetLastInputInfo(&mut info).as_bool() {
            return 0;
        }
        u64::from(GetTickCount().wrapping_sub(info.dwTime))
    }
}

/// Botones del ratón/lápiz, modificadores o espacio (paneo) que estén presionados ahora.
pub fn input_held() -> bool {
    const KEYS: [i32; 11] = [
        0x01, 0x02, 0x04, 0x05, 0x06, // botones del ratón y laterales
        0x10, 0x11, 0x12, // Shift, Ctrl, Alt
        0x5B, 0x5C, // Windows
        0x20, // Espacio
    ];
    KEYS.iter().any(|&vk| unsafe { GetAsyncKeyState(vk) } as u16 & 0x8000 != 0)
}

/// `true` si el proceso (o el propio, con `None`) se ejecuta como administrador.
/// Si no se puede consultar su token se asume que sí (Windows lo protege).
pub fn is_elevated(pid: Option<u32>) -> bool {
    unsafe {
        let process = match pid {
            None => GetCurrentProcess(),
            Some(pid) => match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
                Ok(handle) => handle,
                Err(_) => return true,
            },
        };
        let elevated = token_elevated(process).unwrap_or(pid.is_some());
        if pid.is_some() {
            let _ = CloseHandle(process);
        }
        elevated
    }
}

unsafe fn token_elevated(process: HANDLE) -> Option<bool> {
    let mut token = HANDLE::default();
    OpenProcessToken(process, TOKEN_QUERY, &mut token).ok()?;
    let mut elevation = TOKEN_ELEVATION::default();
    let mut size = 0u32;
    let result = GetTokenInformation(
        token,
        TokenElevation,
        Some(&mut elevation as *mut _ as *mut _),
        size_of::<TOKEN_ELEVATION>() as u32,
        &mut size,
    );
    let _ = CloseHandle(token);
    result.ok()?;
    Some(elevation.TokenIsElevated != 0)
}
