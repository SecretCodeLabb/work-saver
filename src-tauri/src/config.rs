//! Configuración del usuario.

use std::fs;
use std::io;
use std::path::Path;

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

/// Lee la configuración. Si el archivo está dañado lo renombra a `.bak` y usa la predeterminada.
pub fn load(path: &Path) -> Config {
    let Ok(text) = fs::read_to_string(path) else {
        return Config::default();
    };
    match serde_json::from_str::<Config>(&text) {
        Ok(config) => config.normalized(),
        Err(error) => {
            eprintln!("config.json inválido ({error}); se usará la configuración predeterminada");
            let _ = fs::rename(path, path.with_extension("json.bak"));
            Config::default()
        }
    }
}

/// Escribe la configuración de forma atómica (archivo temporal + renombrar).
pub fn save(path: &Path, config: &Config) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    let json = serde_json::to_string_pretty(config).map_err(io::Error::other)?;
    fs::write(&tmp, json)?;
    fs::rename(&tmp, path)
}
