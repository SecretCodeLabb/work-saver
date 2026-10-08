//! Estado compartido entre la interfaz, la bandeja y el hilo de autoguardado.

use std::io;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use serde::Serialize;

use crate::config::{self, Config};
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
}

#[derive(Debug, Clone, Serialize)]
pub struct SaveInfo {
    /// Hora local "HH:MM:SS".
    pub time: String,
    /// Nombre del programa.
    pub app: String,
    pub profile_id: String,
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
}

/// Bloquea un mutex aunque otro hilo haya fallado mientras lo tenía.
pub fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}
