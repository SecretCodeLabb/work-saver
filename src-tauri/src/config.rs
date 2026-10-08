//! Configuración del usuario.

use std::fs;
use std::io;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::shortcut;

pub const MIN_INTERVAL: u32 = 1;
pub const MAX_INTERVAL: u32 = 240;
pub const DEFAULT_SHORTCUT: &str = "Ctrl+S";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Autoguardado activado.
    pub enabled: bool,
    /// Un perfil por programa vigilado.
    pub profiles: Vec<Profile>,

    // Campos de la v1.0 (lista global de procesos); solo se leen para migrar.
    #[serde(skip_serializing)]
    processes: Vec<String>,
    #[serde(skip_serializing)]
    interval_minutes: Option<u32>,
}

/// Ajustes de guardado de un programa.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Profile {
    pub id: String,
    /// Nombre visible (ej. "Blender").
    pub name: String,
    /// Ejecutable en minúsculas (ej. "blender.exe").
    pub exe: String,
    pub enabled: bool,
    pub interval_minutes: u32,
    /// Atajo que guarda el documento (ej. "Ctrl+S").
    pub shortcut: String,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            exe: String::new(),
            enabled: true,
            interval_minutes: 5,
            shortcut: DEFAULT_SHORTCUT.into(),
        }
    }
}

impl Profile {
    pub fn new(name: &str, exe: &str) -> Self {
        Self {
            name: name.into(),
            exe: exe.into(),
            ..Default::default()
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: false,
            profiles: vec![
                Profile::new("Blender", "blender.exe"),
                Profile::new("Krita", "krita.exe"),
            ],
            processes: Vec::new(),
            interval_minutes: None,
        }
    }
}

impl Config {
    /// Corrige valores fuera de rango, migra el formato antiguo y quita duplicados.
    pub fn normalized(mut self) -> Self {
        if !self.processes.is_empty() {
            let interval = self.interval_minutes.unwrap_or(5);
            self.profiles = std::mem::take(&mut self.processes)
                .iter()
                .map(|exe| Profile {
                    interval_minutes: interval,
                    ..Profile::new(&display_name(exe), exe)
                })
                .collect();
        }
        self.interval_minutes = None;

        let mut profiles: Vec<Profile> = Vec::new();
        for profile in self.profiles {
            let profile = profile.normalized();
            if !profile.exe.is_empty() && !profiles.iter().any(|p| p.exe == profile.exe) {
                profiles.push(profile);
            }
        }
        self.profiles = profiles;
        self
    }

    /// Perfil activo para un ejecutable.
    pub fn profile_for(&self, exe: &str) -> Option<&Profile> {
        self.profiles.iter().find(|p| p.enabled && p.exe == exe)
    }
}

impl Profile {
    fn normalized(mut self) -> Self {
        self.exe = normalize_exe(&self.exe);
        self.name = self.name.trim().to_string();
        if self.name.is_empty() {
            self.name = display_name(&self.exe);
        }
        if self.id.trim().is_empty() {
            self.id = new_id();
        }
        self.interval_minutes = self.interval_minutes.clamp(MIN_INTERVAL, MAX_INTERVAL);
        self.shortcut = shortcut::normalize(&self.shortcut).unwrap_or_else(|| DEFAULT_SHORTCUT.into());
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

/// "clipstudiopaint.exe" → "Clipstudiopaint".
fn display_name(exe: &str) -> String {
    let stem = exe.trim_end_matches(".exe");
    let mut chars = stem.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

fn new_id() -> String {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    format!("p{:x}{:x}", nanos, COUNTER.fetch_add(1, Ordering::Relaxed))
}

/// Lee la configuración. Si el archivo está dañado lo renombra a `.bak` y usa la predeterminada.
pub fn load(path: &Path) -> Config {
    let Ok(text) = fs::read_to_string(path) else {
        return Config::default().normalized();
    };
    match serde_json::from_str::<Config>(&text) {
        Ok(config) => config.normalized(),
        Err(error) => {
            eprintln!("config.json inválido ({error}); se usará la configuración predeterminada");
            let _ = fs::rename(path, path.with_extension("json.bak"));
            Config::default().normalized()
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
