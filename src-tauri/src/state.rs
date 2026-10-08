//! Estado compartido entre la interfaz, la bandeja y el hilo de autoguardado.

use std::io;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use serde::Serialize;

use crate::config::{self, Config};
use crate::crash::CrashAlert;
use crate::scheduler::Reason;

pub struct Shared {
    config: Mutex<Config>,
    config_path: PathBuf,
    runtime: Mutex<Runtime>,
}

/// Estado en memoria que no se guarda en disco.
#[derive(Default)]
pub struct Runtime {
    pub last_save: Option<SaveInfo>,
    /// Último cierre inesperado, hasta que el usuario lo descarte.
    pub crash_alert: Option<CrashAlert>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SaveInfo {
    /// Hora local "HH:MM:SS".
    pub time: String,
    /// Nombre del programa.
    pub app: String,
    pub profile_id: String,
    /// `Some(true)` si se detectó el archivo escrito; `Some(false)` si no apareció.
    pub verified: Option<bool>,
    /// Nombre del archivo guardado, si se verificó.
    pub file: Option<String>,
}

/// Instantánea que se envía a la interfaz en cada ciclo.
#[derive(Debug, Clone, Serialize)]
pub struct Status {
    pub enabled: bool,
    pub foreground: Option<ForegroundInfo>,
    /// Perfil del programa en primer plano, si está vigilado.
    pub active_profile: Option<String>,
    pub profiles: Vec<ProfileStatus>,
    pub last_save: Option<SaveInfo>,
    pub crash_alert: Option<CrashAlert>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ForegroundInfo {
    pub exe: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProfileStatus {
    pub id: String,
    /// Segundos para el próximo guardado (0 = pendiente).
    pub next_save_in: u64,
    /// Motivo por el que el guardado pendiente espera o se omitió.
    pub reason: Option<Reason>,
}

impl Shared {
    /// Carga la configuración guardada en `config_path`.
    pub fn load(config_path: PathBuf) -> Self {
        Self {
            config: Mutex::new(config::load(&config_path)),
            config_path,
            runtime: Mutex::new(Runtime::default()),
        }
    }

    pub fn config(&self) -> Config {
        lock(&self.config).clone()
    }

    /// Reemplaza la configuración y la guarda en disco.
    /// La configuración en memoria se actualiza aunque falle la escritura.
    pub fn set_config(&self, config: Config) -> io::Result<()> {
        let mut current = lock(&self.config);
        let result = config::save(&self.config_path, &config);
        *current = config;
        result
    }

    pub fn runtime(&self) -> MutexGuard<'_, Runtime> {
        lock(&self.runtime)
    }

    /// Último guardado y cierre inesperado, leídos con un único bloqueo.
    ///
    /// Llamar a `runtime()` dos veces en la misma expresión bloquea el hilo para siempre:
    /// el primer `MutexGuard` temporal sigue vivo hasta el final de la instrucción.
    pub fn runtime_snapshot(&self) -> (Option<SaveInfo>, Option<CrashAlert>) {
        let runtime = self.runtime();
        (runtime.last_save.clone(), runtime.crash_alert.clone())
    }
}

/// Bloquea un mutex aunque otro hilo haya fallado mientras lo tenía.
pub fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    /// Regresión: leer el estado en cada ciclo no debe bloquear el hilo (deadlock de la v2.0.0).
    #[test]
    fn runtime_snapshot_does_not_deadlock() {
        let shared = std::sync::Arc::new(Shared::load(std::env::temp_dir().join("dcn-test-no-existe.json")));
        shared.runtime().last_save = Some(SaveInfo {
            time: "12:00:00".into(),
            app: "Krita".into(),
            profile_id: "p1".into(),
            verified: None,
            file: None,
        });
        let (tx, rx) = mpsc::channel();
        let worker = shared.clone();
        std::thread::spawn(move || {
            for _ in 0..3 {
                let _ = worker.runtime_snapshot();
            }
            tx.send(worker.runtime_snapshot()).unwrap();
        });
        let (last_save, crash) = rx.recv_timeout(Duration::from_secs(2)).expect("el hilo quedó bloqueado");
        assert_eq!(last_save.map(|s| s.app).as_deref(), Some("Krita"));
        assert!(crash.is_none());
        // El candado quedó libre.
        assert!(shared.runtime.try_lock().is_ok());
    }
}
