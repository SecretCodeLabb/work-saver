//! Configuración del usuario.

use serde::{Deserialize, Serialize};

pub const MIN_INTERVAL: u32 = 1;
pub const MAX_INTERVAL: u32 = 240;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Autoguardado activado.
    pub enabled: bool,
    pub interval_minutes: u32,
    /// Ejecutables vigilados, en minúsculas.
    pub processes: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_minutes: 5,
            processes: vec!["blender.exe".into(), "krita.exe".into()],
        }
    }
}

impl Config {
    /// Corrige valores fuera de rango o mal escritos.
    pub fn normalized(mut self) -> Self {
        self.interval_minutes = self.interval_minutes.clamp(MIN_INTERVAL, MAX_INTERVAL);
        let mut processes: Vec<String> = Vec::new();
        for exe in self.processes {
            let exe = normalize_exe(&exe);
            if !exe.is_empty() && !processes.contains(&exe) {
                processes.push(exe);
            }
        }
        self.processes = processes;
        self
    }
}

/// "Blender" → "blender.exe", "  KRITA.EXE " → "krita.exe".
pub fn normalize_exe(exe: &str) -> String {
    let exe = exe.trim().to_lowercase();
    if exe.is_empty() || exe.ends_with(".exe") {
        exe
    } else {
        format!("{exe}.exe")
    }
}
