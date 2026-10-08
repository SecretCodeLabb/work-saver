//! Estado compartido entre la interfaz, la bandeja y el hilo de autoguardado.

use std::sync::{Mutex, MutexGuard};

use serde::Serialize;

use crate::config::Config;

pub struct Shared {
    config: Mutex<Config>,
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
    pub app: String,
}

/// Instantánea que se envía a la interfaz en cada ciclo.
#[derive(Debug, Clone, Serialize)]
pub struct Status {
    pub enabled: bool,
    /// Ejecutable de la ventana activa.
    pub foreground: Option<String>,
    /// Segundos para el próximo guardado.
    pub next_save_in: Option<u64>,
    pub last_save: Option<SaveInfo>,
}

impl Shared {
    pub fn new(config: Config) -> Self {
        Self {
            config: Mutex::new(config),
            runtime: Mutex::new(Runtime::default()),
        }
    }

    pub fn config(&self) -> Config {
        lock(&self.config).clone()
    }

    pub fn set_config(&self, config: Config) {
        *lock(&self.config) = config;
    }

    pub fn runtime(&self) -> MutexGuard<'_, Runtime> {
        lock(&self.runtime)
    }
}

/// Bloquea un mutex aunque otro hilo haya fallado mientras lo tenía.
pub fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}
