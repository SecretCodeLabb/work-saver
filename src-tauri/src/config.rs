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
    /// Pausa temporal hasta este momento (milisegundos desde 1970).
    pub paused_until: Option<i64>,
    /// Un perfil por programa vigilado.
    pub profiles: Vec<Profile>,
    pub smart: SmartSave,
    pub backups: BackupConfig,
    pub notifications: Notifications,
    /// Atajo global que activa o desactiva el autoguardado; vacío = sin atajo.
    pub toggle_hotkey: String,
    /// Iniciar con Windows.
    pub autostart: bool,
    /// Al iniciar con Windows, quedarse en la bandeja sin abrir la ventana.
    pub start_minimized: bool,

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
    /// Textos del título que indican un documento sin nombre (no se guarda).
    pub untitled_markers: Vec<String>,
    /// Textos del título que indican cambios sin guardar (ej. "*").
    pub dirty_markers: Vec<String>,
    /// Extensiones de los archivos del programa, sin punto (ej. "blend").
    pub extensions: Vec<String>,
    /// Carpetas de proyecto donde se vigilan los archivos para respaldarlos.
    pub watch_folders: Vec<String>,
}

/// Avisos del sistema (notificaciones de Windows).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Notifications {
    /// Notificar cada guardado.
    pub on_save: bool,
    /// Notificar problemas: documento sin nombre, guardado no verificado, cierres inesperados.
    pub on_problem: bool,
    /// Avisar unos segundos antes de guardar.
    pub warn_before: bool,
    pub warn_seconds: u32,
    /// Sonido breve al guardar.
    pub sound: bool,
}

impl Default for Notifications {
    fn default() -> Self {
        Self {
            on_save: false,
            on_problem: true,
            warn_before: false,
            warn_seconds: 5,
            sound: false,
        }
    }
}

pub const DEFAULT_TOGGLE_HOTKEY: &str = "Ctrl+Shift+Alt+F9";

/// Copias de seguridad con historial.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BackupConfig {
    pub enabled: bool,
    /// Carpeta de respaldos; vacía = carpeta predeterminada de la app.
    pub folder: String,
    /// Versiones que se conservan de cada archivo.
    pub keep_per_file: u32,
    /// Minutos mínimos entre dos copias del mismo archivo.
    pub min_minutes_between: u32,
    /// Espacio máximo total; se borran las copias más antiguas al superarlo.
    pub max_total_mb: u64,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            folder: String::new(),
            keep_per_file: 20,
            min_minutes_between: 5,
            max_total_mb: 2048,
        }
    }
}

impl BackupConfig {
    fn normalized(mut self) -> Self {
        self.folder = self.folder.trim().to_string();
        self.keep_per_file = self.keep_per_file.clamp(1, 500);
        self.min_minutes_between = self.min_minutes_between.min(240);
        self.max_total_mb = self.max_total_mb.clamp(50, 1_000_000);
        self
    }
}

/// Reglas para no interrumpir al usuario al guardar.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SmartSave {
    /// Esperar a que el usuario deje de usar teclado, ratón o lápiz.
    pub wait_for_idle: bool,
    pub idle_seconds: u32,
    /// Pasado este tiempo se guarda aunque no haya pausa (si no hay teclas pulsadas).
    pub max_wait_seconds: u32,
    pub skip_untitled: bool,
    pub skip_dialogs: bool,
    /// Guardar solo si el título indica cambios sin guardar.
    pub only_when_dirty: bool,
}

impl Default for SmartSave {
    fn default() -> Self {
        Self {
            wait_for_idle: true,
            idle_seconds: 2,
            max_wait_seconds: 120,
            skip_untitled: true,
            skip_dialogs: true,
            only_when_dirty: true,
        }
    }
}

impl SmartSave {
    fn normalized(mut self) -> Self {
        self.idle_seconds = self.idle_seconds.clamp(1, 30);
        self.max_wait_seconds = self.max_wait_seconds.clamp(10, 900);
        self
    }
}

pub const DEFAULT_UNTITLED_MARKERS: [&str; 4] = ["Untitled-", "Sin título-", "Unsaved", "Sin guardar"];
pub const DEFAULT_DIRTY_MARKERS: [&str; 1] = ["*"];

impl Default for Profile {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            exe: String::new(),
            enabled: true,
            interval_minutes: 5,
            shortcut: DEFAULT_SHORTCUT.into(),
            untitled_markers: DEFAULT_UNTITLED_MARKERS.map(String::from).to_vec(),
            dirty_markers: DEFAULT_DIRTY_MARKERS.map(String::from).to_vec(),
            extensions: Vec::new(),
            watch_folders: Vec::new(),
        }
    }
}

impl Profile {
    pub fn new(name: &str, exe: &str) -> Self {
        let extensions = crate::presets::find(exe)
            .map(|p| p.extensions.iter().map(|e| e.to_string()).collect())
            .unwrap_or_default();
        Self {
            name: name.into(),
            exe: exe.into(),
            extensions,
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
            smart: SmartSave::default(),
            backups: BackupConfig::default(),
            notifications: Notifications::default(),
            toggle_hotkey: DEFAULT_TOGGLE_HOTKEY.into(),
            paused_until: None,
            autostart: false,
            start_minimized: true,
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
        self.smart = self.smart.normalized();
        self.backups = self.backups.normalized();
        self.notifications.warn_seconds = self.notifications.warn_seconds.clamp(1, 60);
        self.toggle_hotkey = if self.toggle_hotkey.trim().is_empty() {
            String::new()
        } else {
            shortcut::normalize(&self.toggle_hotkey).unwrap_or_else(|| DEFAULT_TOGGLE_HOTKEY.into())
        };
        self
    }

    /// Autoguardado activado y sin pausa.
    pub fn is_running(&self) -> bool {
        self.enabled && self.paused_until.is_none()
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
        self.untitled_markers = clean_list(self.untitled_markers);
        self.dirty_markers = clean_list(self.dirty_markers);
        self.extensions = clean_list(
            self.extensions
                .iter()
                .map(|e| e.trim().trim_start_matches('.').to_lowercase())
                .collect(),
        );
        self.watch_folders = clean_list(self.watch_folders);
        self
    }

    /// El archivo pertenece a este programa (extensión y carpeta vigilada).
    pub fn owns_file(&self, path: &Path) -> bool {
        let Some(ext) = path.extension().map(|e| e.to_string_lossy().to_lowercase()) else {
            return false;
        };
        self.extensions.contains(&ext)
            && self.watch_folders.iter().any(|folder| path.starts_with(folder))
    }

    /// El título indica que el documento nunca se guardó.
    pub fn is_untitled(&self, title: &str) -> bool {
        contains_any(title, &self.untitled_markers)
    }

    /// El título indica cambios sin guardar.
    pub fn is_dirty(&self, title: &str) -> bool {
        contains_any(title, &self.dirty_markers)
    }
}

fn contains_any(title: &str, markers: &[String]) -> bool {
    let title = title.to_lowercase();
    markers.iter().any(|m| title.contains(&m.to_lowercase()))
}

/// Quita espacios sobrantes, vacíos y duplicados.
fn clean_list(items: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for item in items {
        let item = item.trim().to_string();
        if !item.is_empty() && !out.contains(&item) {
            out.push(item);
        }
    }
    out
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrates_v1_process_list() {
        let json = r#"{"enabled":true,"interval_minutes":3,"processes":["Krita.exe","aseprite"]}"#;
        let config = serde_json::from_str::<Config>(json).unwrap().normalized();
        let exes: Vec<_> = config.profiles.iter().map(|p| p.exe.as_str()).collect();
        assert_eq!(exes, ["krita.exe", "aseprite.exe"]);
        assert!(config.profiles.iter().all(|p| p.interval_minutes == 3 && !p.id.is_empty()));
    }

    #[test]
    fn detects_title_markers() {
        let profile = Profile::new("Photoshop", "photoshop.exe");
        assert!(profile.is_untitled("Untitled-1 @ 66,7% (RGB/8)"));
        assert!(!profile.is_untitled("retrato.psd @ 50%"));
        assert!(profile.is_dirty("escena.blend* - Blender"));
        assert!(!profile.is_dirty("escena.blend - Blender"));
    }
}
