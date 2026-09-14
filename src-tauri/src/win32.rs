use std::ffi::OsString;
use std::mem::size_of;
use std::os::windows::ffi::OsStringExt;
use windows::Win32::Foundation::{CloseHandle, HWND, MAX_PATH};
use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_CONTROL,
};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

/// Obtiene el nombre del ejecutable de la ventana activa (foreground window).
/// Retorna Option<String> con el nombre en minúsculas (ej. "blender.exe").
pub fn get_active_process_name() -> Option<String> {
    unsafe {
        // 1. Obtener la ventana activa
        let hwnd: HWND = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }

        // 2. Obtener el PID (Process ID) asociado a la ventana
        let mut process_id: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));
        if process_id == 0 {
            return None;
        }

        // 3. Abrir el proceso con permisos limitados (suficiente para consultar información)
        let process_handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id).ok()?;

        // 4. Obtener la ruta del ejecutable
        let mut buffer = [0u16; MAX_PATH as usize];
        let length = GetModuleFileNameExW(process_handle, None, &mut buffer);
        let _ = CloseHandle(process_handle);

        if length == 0 {
            return None;
        }

        // 5. Convertir la ruta a String
        let path_os_string = OsString::from_wide(&buffer[..length as usize]);
        let path_string = path_os_string.into_string().ok()?;

        // 6. Extraer solo el nombre del ejecutable y convertir a minúsculas
        let path = std::path::Path::new(&path_string);
        let file_name = path.file_name()?.to_string_lossy().to_string();

        Some(file_name.to_lowercase())
    }
}

/// Simula la pulsación de las teclas Ctrl + S usando SendInput de la API de Win32.
pub fn send_ctrl_s() {
    unsafe {
        let mut inputs = [INPUT::default(); 4];

        // KeyDown Control
        inputs[0].r#type = INPUT_KEYBOARD;
        inputs[0].Anonymous.ki = KEYBDINPUT {
            wVk: VK_CONTROL,
            wScan: 0,
            dwFlags: Default::default(), // 0 para KeyDown
            time: 0,
            dwExtraInfo: 0,
        };

        // KeyDown 'S' (0x53)
        inputs[1].r#type = INPUT_KEYBOARD;
        inputs[1].Anonymous.ki = KEYBDINPUT {
            wVk: windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY(0x53),
            wScan: 0,
            dwFlags: Default::default(),
            time: 0,
            dwExtraInfo: 0,
        };

        // KeyUp 'S' (0x53)
        inputs[2].r#type = INPUT_KEYBOARD;
        inputs[2].Anonymous.ki = KEYBDINPUT {
            wVk: windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY(0x53),
            wScan: 0,
            dwFlags: KEYEVENTF_KEYUP,
            time: 0,
            dwExtraInfo: 0,
        };

        // KeyUp Control
        inputs[3].r#type = INPUT_KEYBOARD;
        inputs[3].Anonymous.ki = KEYBDINPUT {
            wVk: VK_CONTROL,
            wScan: 0,
            dwFlags: KEYEVENTF_KEYUP,
            time: 0,
            dwExtraInfo: 0,
        };

        let size = size_of::<INPUT>() as i32;
        let _ = SendInput(&inputs, size);
    }
}
