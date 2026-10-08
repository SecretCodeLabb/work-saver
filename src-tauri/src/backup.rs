//! Copias de seguridad con historial y verificación de guardados.
//!
//! Se vigilan las carpetas de proyecto de cada programa. Cuando un archivo con la
//! extensión del programa termina de escribirse se copia, con fecha y hora, a la
//! carpeta de respaldos. El mismo evento sirve para confirmar que un `Ctrl+S`
//! enviado por la app produjo realmente un guardado.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use chrono::{DateTime, Local};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use crate::actions;
use crate::activity::{self, Kind};
use crate::config::{BackupConfig, Config, Profile};
use crate::texts::{tr, Text};
use crate::state::{lock, Shared};

/// Tiempo sin cambios antes de considerar que el archivo terminó de escribirse.
const QUIET: Duration = Duration::from_secs(2);
/// Tiempo para que aparezca en disco el archivo de un guardado enviado.
const VERIFY_TIMEOUT: Duration = Duration::from_secs(45);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupEntry {
    pub id: String,
    /// Archivo original.
    pub original: PathBuf,
    /// Copia de seguridad.
    pub path: PathBuf,
    /// Milisegundos desde 1970.
    pub created_ms: i64,
    pub size: u64,
    /// Programa al que pertenece.
    pub app: String,
}

/// Versiones de un mismo archivo, de la más nueva a la más antigua.
#[derive(Debug, Clone, Serialize)]
pub struct BackupGroup {
    pub original: PathBuf,
    pub name: String,
    pub app: String,
    pub total_size: u64,
    pub entries: Vec<BackupEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SaveCheck {
    pub profile_id: String,
    pub app: String,
    /// Archivo guardado (si se verificó).
    pub file: Option<String>,
}

struct Expected {
    profile_id: String,
    app: String,
    since: SystemTime,
    deadline: Instant,
    /// El título indicaba cambios, así que debería haberse escrito el archivo.
    dirty: bool,
}

pub struct Backups {
    app: AppHandle,
    shared: Arc<Shared>,
    default_root: PathBuf,
    index_path: PathBuf,
    index: Mutex<Vec<BackupEntry>>,
    /// Archivos modificados recientemente → momento del último evento.
    pending: Mutex<HashMap<PathBuf, Instant>>,
    /// Guardados enviados pendientes de confirmar.
    expected: Mutex<Vec<Expected>>,
    watcher: Mutex<Option<(Vec<PathBuf>, RecommendedWatcher)>>,
}

impl Backups {
    pub fn start(app: AppHandle, shared: Arc<Shared>, data_dir: PathBuf) -> Arc<Self> {
        let index_path = data_dir.join("backups.json");
        let index = fs::read_to_string(&index_path)
            .ok()
            .and_then(|text| serde_json::from_str::<Vec<BackupEntry>>(&text).ok())
            .unwrap_or_default();

        let backups = Arc::new(Self {
            app,
            shared,
            default_root: data_dir.join("backups"),
            index_path,
            index: Mutex::new(index),
            pending: Mutex::new(HashMap::new()),
            expected: Mutex::new(Vec::new()),
            watcher: Mutex::new(None),
        });
        backups.refresh_watches();

        let worker = backups.clone();
        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(1));
            worker.process_pending();
            worker.expire_checks();
        });
        backups
    }

    /// Carpeta donde se guardan las copias.
    pub fn root(&self, config: &BackupConfig) -> PathBuf {
        if config.folder.is_empty() {
            self.default_root.clone()
        } else {
            PathBuf::from(&config.folder)
        }
    }

    /// Vuelve a vigilar las carpetas tras un cambio de configuración.
    pub fn refresh_watches(self: &Arc<Self>) {
        let config = self.shared.config();
        let mut folders: Vec<PathBuf> = config
            .profiles
            .iter()
            .filter(|p| p.enabled && !p.extensions.is_empty())
            .flat_map(|p| p.watch_folders.iter().map(PathBuf::from))
            .filter(|f| f.is_dir())
            .collect();
        folders.sort();
        folders.dedup();
        // Una carpeta dentro de otra ya vigilada no hace falta.
        let roots: Vec<PathBuf> = folders
            .iter()
            .filter(|f| !folders.iter().any(|other| other != *f && f.starts_with(other)))
            .cloned()
            .collect();

        let mut current = lock(&self.watcher);
        if current.as_ref().is_some_and(|(watched, _)| *watched == roots) {
            return;
        }
        *current = None;
        if roots.is_empty() {
            return;
        }

        let this = Arc::downgrade(self);
        let watcher = notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
            let (Ok(event), Some(this)) = (result, this.upgrade()) else {
                return;
            };
            if matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_)) {
                let now = Instant::now();
                let mut pending = lock(&this.pending);
                for path in event.paths {
                    pending.insert(path, now);
                }
            }
        });
        match watcher {
            Ok(mut watcher) => {
                for root in &roots {
                    if let Err(error) = watcher.watch(root, RecursiveMode::Recursive) {
                        eprintln!("No se puede vigilar {}: {error}", root.display());
                    }
                }
                *current = Some((roots, watcher));
            }
            Err(error) => eprintln!("No se pudo crear el vigilante de archivos: {error}"),
        }
    }

    /// Registra un guardado enviado para comprobar que el archivo se escribe.
    pub fn expect_save(&self, profile: &Profile, dirty: bool) {
        if profile.watch_folders.is_empty() || profile.extensions.is_empty() {
            return;
        }
        lock(&self.expected).push(Expected {
            profile_id: profile.id.clone(),
            app: profile.name.clone(),
            since: SystemTime::now() - Duration::from_secs(2),
            deadline: Instant::now() + VERIFY_TIMEOUT,
            dirty,
        });
    }

    fn process_pending(&self) {
        let ready: Vec<PathBuf> = {
            let mut pending = lock(&self.pending);
            let ready: Vec<PathBuf> = pending
                .iter()
                .filter(|(_, at)| at.elapsed() >= QUIET)
                .map(|(path, _)| path.clone())
                .collect();
            for path in &ready {
                pending.remove(path);
            }
            ready
        };
        if ready.is_empty() {
            return;
        }

        let config = self.shared.config();
        let root = self.root(&config.backups);
        for path in ready {
            if path.starts_with(&root) || is_temp_file(&path) {
                continue;
            }
            let Ok(meta) = fs::metadata(&path) else { continue };
            if !meta.is_file() || meta.len() == 0 {
                continue;
            }
            let Some(profile) = config.profiles.iter().find(|p| p.enabled && p.owns_file(&path)) else {
                continue;
            };
            self.confirm_save(profile, &path, meta.modified().ok());
            if config.backups.enabled {
                if let Err(error) = self.backup(&config, profile, &path) {
                    eprintln!("No se pudo respaldar {}: {error}", path.display());
                }
            }
        }
    }

    fn confirm_save(&self, profile: &Profile, path: &Path, modified: Option<SystemTime>) {
        let modified = modified.unwrap_or_else(SystemTime::now);
        let mut expected = lock(&self.expected);
        let before = expected.len();
        expected.retain(|e| !(e.profile_id == profile.id && e.since <= modified));
        if expected.len() == before {
            return;
        }
        drop(expected);

        let file = path.file_name().map(|n| n.to_string_lossy().to_string());
        if let Some(last) = self.shared.runtime().last_save.as_mut() {
            if last.profile_id == profile.id {
                last.verified = Some(true);
                last.file = file.clone();
            }
        }
        let _ = self.app.emit(
            "save_verified",
            SaveCheck {
                profile_id: profile.id.clone(),
                app: profile.name.clone(),
                file,
            },
        );
    }

    fn expire_checks(&self) {
        let now = Instant::now();
        let mut expired = Vec::new();
        lock(&self.expected).retain(|e| {
            let alive = e.deadline > now;
            if !alive {
                expired.push((e.profile_id.clone(), e.app.clone(), e.dirty));
            }
            alive
        });
        for (profile_id, app, dirty) in expired {
            if let Some(last) = self.shared.runtime().last_save.as_mut() {
                if last.profile_id == profile_id && last.verified.is_none() {
                    last.verified = Some(false);
                }
            }
            // Solo es un problema si había cambios: sin cambios muchos programas no escriben nada.
            if dirty {
                activity::record(&self.app, Kind::Unverified, &app, "");
                if self.shared.config().notifications.on_problem {
                    actions::notify(
                        &self.app,
                        &tr(Text::NotifyUnverifiedTitle, &[("app", &app)]),
                        &tr(Text::NotifyUnverifiedBody, &[]),
                    );
                }
                let _ = self.app.emit("save_unverified", SaveCheck { profile_id, app, file: None });
            }
        }
    }

    fn backup(&self, config: &Config, profile: &Profile, original: &Path) -> io::Result<()> {
        let now = Local::now();
        let min_gap = i64::from(config.backups.min_minutes_between) * 60_000;
        {
            let index = lock(&self.index);
            let last = index.iter().filter(|e| e.original == original).map(|e| e.created_ms).max();
            if last.is_some_and(|ms| now.timestamp_millis() - ms < min_gap) {
                return Ok(());
            }
        }

        let stem = original.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        let ext = original.extension().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        let dir = self.root(&config.backups).join(format!(
            "{}_{:08x}",
            sanitize(&stem),
            fnv1a(&original.to_string_lossy().to_lowercase())
        ));
        fs::create_dir_all(&dir)?;
        let dest = dir.join(format!("{}_{}.{}", sanitize(&stem), now.format("%Y-%m-%d_%H-%M-%S"), ext));
        let size = fs::copy(original, &dest)?;

        let entry = BackupEntry {
            id: format!("{:x}{:08x}", now.timestamp_millis(), fnv1a(&dest.to_string_lossy())),
            original: original.to_path_buf(),
            path: dest,
            created_ms: now.timestamp_millis(),
            size,
            app: profile.name.clone(),
        };
        {
            let mut index = lock(&self.index);
            index.push(entry.clone());
            prune(&mut index, &config.backups);
            self.save_index(&index);
        }
        activity::record(&self.app, Kind::Backup, &entry.app, &entry.original.to_string_lossy());
        let _ = self.app.emit("backup_created", entry);
        Ok(())
    }

    pub fn groups(&self) -> Vec<BackupGroup> {
        let mut index = lock(&self.index);
        let before = index.len();
        index.retain(|e| e.path.is_file());
        if index.len() != before {
            self.save_index(&index);
        }

        let mut groups: Vec<BackupGroup> = Vec::new();
        for entry in index.iter() {
            match groups.iter_mut().find(|g| g.original == entry.original) {
                Some(group) => {
                    group.total_size += entry.size;
                    group.entries.push(entry.clone());
                }
                None => groups.push(BackupGroup {
                    original: entry.original.clone(),
                    name: entry
                        .original
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    app: entry.app.clone(),
                    total_size: entry.size,
                    entries: vec![entry.clone()],
                }),
            }
        }
        for group in &mut groups {
            group.entries.sort_by_key(|e| std::cmp::Reverse(e.created_ms));
        }
        groups.sort_by_key(|g| std::cmp::Reverse(g.entries[0].created_ms));
        groups
    }

    /// Respaldo más reciente de un programa.
    pub fn latest_for(&self, app: &str) -> Option<BackupEntry> {
        lock(&self.index)
            .iter()
            .filter(|e| e.app == app && e.path.is_file())
            .max_by_key(|e| e.created_ms)
            .cloned()
    }

    pub fn find(&self, id: &str) -> Option<BackupEntry> {
        lock(&self.index).iter().find(|e| e.id == id).cloned()
    }

    /// Copia la versión junto al original sin sobrescribirlo. Devuelve la ruta creada.
    pub fn restore(&self, id: &str) -> io::Result<PathBuf> {
        let entry = self.find(id).ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "respaldo no encontrado"))?;
        let dir = entry
            .original
            .parent()
            .filter(|d| d.is_dir())
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "la carpeta original ya no existe"))?;
        let stem = entry.original.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        let ext = entry.original.extension().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        let stamp = DateTime::from_timestamp_millis(entry.created_ms)
            .map(|d| d.with_timezone(&Local).format("%Y-%m-%d %H-%M").to_string())
            .unwrap_or_default();

        let mut dest = dir.join(format!("{stem} (restaurado {stamp}).{ext}"));
        let mut n = 2;
        while dest.exists() {
            dest = dir.join(format!("{stem} (restaurado {stamp}) {n}.{ext}"));
            n += 1;
        }
        fs::copy(&entry.path, &dest)?;
        Ok(dest)
    }

    pub fn delete(&self, id: &str) -> io::Result<()> {
        let mut index = lock(&self.index);
        if let Some(pos) = index.iter().position(|e| e.id == id) {
            let entry = index.remove(pos);
            remove_backup_file(&entry.path);
            self.save_index(&index);
        }
        Ok(())
    }

    /// Borra todas las versiones de un archivo.
    pub fn delete_group(&self, original: &Path) {
        let mut index = lock(&self.index);
        index.retain(|e| {
            let keep = e.original != original;
            if !keep {
                remove_backup_file(&e.path);
            }
            keep
        });
        self.save_index(&index);
    }

    /// Aplica los límites actuales (tras cambiar la configuración).
    pub fn apply_limits(&self) {
        let config = self.shared.config();
        let mut index = lock(&self.index);
        prune(&mut index, &config.backups);
        self.save_index(&index);
    }

    fn save_index(&self, index: &[BackupEntry]) {
        let result = serde_json::to_string(index)
            .map_err(io::Error::other)
            .and_then(|json| {
                if let Some(dir) = self.index_path.parent() {
                    fs::create_dir_all(dir)?;
                }
                let tmp = self.index_path.with_extension("json.tmp");
                fs::write(&tmp, json)?;
                fs::rename(&tmp, &self.index_path)
            });
        if let Err(error) = result {
            eprintln!("No se pudo guardar el índice de respaldos: {error}");
        }
    }
}

/// Conserva `keep_per_file` versiones por archivo y respeta el espacio máximo total.
fn prune(index: &mut Vec<BackupEntry>, config: &BackupConfig) {
    index.sort_by_key(|e| std::cmp::Reverse(e.created_ms));

    let mut counts: HashMap<PathBuf, u32> = HashMap::new();
    let mut total: u64 = 0;
    let max_total = config.max_total_mb * 1024 * 1024;
    index.retain(|e| {
        let count = counts.entry(e.original.clone()).or_default();
        *count += 1;
        // La versión más reciente de cada archivo se conserva siempre.
        let keep = *count == 1 || (*count <= config.keep_per_file && total + e.size <= max_total);
        if keep {
            total += e.size;
        } else {
            remove_backup_file(&e.path);
        }
        keep
    });
}

fn remove_backup_file(path: &Path) {
    let _ = fs::remove_file(path);
    if let Some(dir) = path.parent() {
        let _ = fs::remove_dir(dir); // Solo se borra si quedó vacía.
    }
}

/// Archivos temporales que algunos programas escriben al guardar.
fn is_temp_file(path: &Path) -> bool {
    let name = path.file_name().map(|n| n.to_string_lossy().to_lowercase()).unwrap_or_default();
    name.starts_with('~') || name.starts_with(".~") || name.ends_with(".tmp")
}

fn sanitize(name: &str) -> String {
    let clean: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() || " -_.()".contains(c) { c } else { '_' })
        .take(60)
        .collect();
    if clean.trim().is_empty() {
        "archivo".into()
    } else {
        clean
    }
}

/// Hash FNV-1a de 32 bits (estable entre versiones, a diferencia de `DefaultHasher`).
fn fnv1a(text: &str) -> u32 {
    text.bytes()
        .fold(0x811c9dc5u32, |hash, byte| (hash ^ u32::from(byte)).wrapping_mul(0x0100_0193))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(original: &str, created_ms: i64, size: u64) -> BackupEntry {
        BackupEntry {
            id: format!("{original}{created_ms}"),
            original: PathBuf::from(original),
            path: PathBuf::from(format!("Z:/no-existe/{original}{created_ms}")),
            created_ms,
            size,
            app: "Test".into(),
        }
    }

    #[test]
    fn prune_keeps_newest_per_file() {
        let mut index = vec![entry("a", 1, 10), entry("a", 2, 10), entry("a", 3, 10), entry("b", 1, 10)];
        let config = BackupConfig { keep_per_file: 2, ..Default::default() };
        prune(&mut index, &config);
        let kept: Vec<_> = index.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(kept, ["a3", "a2", "b1"]);
    }

    #[test]
    fn prune_respects_total_size_but_keeps_latest() {
        let mb = 1024 * 1024;
        let mut index = vec![entry("a", 1, 40 * mb), entry("a", 2, 40 * mb), entry("b", 3, 40 * mb)];
        let config = BackupConfig { max_total_mb: 50, ..Default::default() };
        prune(&mut index, &config);
        let kept: Vec<_> = index.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(kept, ["b3", "a2"]);
    }

    #[test]
    fn ignores_temp_files() {
        assert!(is_temp_file(Path::new("C:/p/~escena.psd")));
        assert!(is_temp_file(Path::new("C:/p/escena.kra.tmp")));
        assert!(!is_temp_file(Path::new("C:/p/escena.kra")));
    }
}
