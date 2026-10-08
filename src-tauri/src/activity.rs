//! Estadísticas diarias y registro de actividad (en la interfaz y en un archivo de texto).

use std::collections::{BTreeMap, VecDeque};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

use crate::state::lock;

const MAX_EVENTS: usize = 300;
const MAX_DAYS: usize = 90;
const MAX_LOG_BYTES: u64 = 1024 * 1024;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Save,
    Skip,
    Backup,
    Unverified,
    Problem,
    Crash,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct DayStats {
    pub saves: u32,
    pub skipped: u32,
    pub backups: u32,
    pub crashes: u32,
    /// Segundos con un programa vigilado en primer plano y el autoguardado activo.
    pub protected_seconds: u64,
}

impl DayStats {
    fn add(&mut self, other: &DayStats) {
        self.saves += other.saves;
        self.skipped += other.skipped;
        self.backups += other.backups;
        self.crashes += other.crashes;
        self.protected_seconds += other.protected_seconds;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub time_ms: i64,
    pub kind: Kind,
    pub app: String,
    pub detail: String,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct Data {
    days: BTreeMap<String, DayStats>,
    events: VecDeque<Event>,
}

#[derive(Serialize)]
pub struct Summary {
    pub today: DayStats,
    pub week: DayStats,
    pub total: DayStats,
    pub events: Vec<Event>,
}

pub struct Activity {
    path: PathBuf,
    log_dir: PathBuf,
    data: Mutex<Data>,
    dirty: AtomicBool,
}

impl Activity {
    pub fn start(data_dir: PathBuf, log_dir: PathBuf) -> Arc<Self> {
        let path = data_dir.join("activity.json");
        let data = fs::read_to_string(&path)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default();
        let activity = Arc::new(Self {
            path,
            log_dir,
            data: Mutex::new(data),
            dirty: AtomicBool::new(false),
        });
        let saver = activity.clone();
        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(30));
            saver.flush();
        });
        activity
    }

    pub fn record(&self, kind: Kind, app: &str, detail: &str) {
        let now = Local::now();
        {
            let mut data = lock(&self.data);
            let day = data.days.entry(today_key()).or_default();
            match kind {
                Kind::Save => day.saves += 1,
                Kind::Skip => day.skipped += 1,
                Kind::Backup => day.backups += 1,
                Kind::Crash => day.crashes += 1,
                Kind::Unverified | Kind::Problem => {}
            }
            data.events.push_front(Event {
                time_ms: now.timestamp_millis(),
                kind,
                app: app.into(),
                detail: detail.into(),
            });
            data.events.truncate(MAX_EVENTS);
            while data.days.len() > MAX_DAYS {
                data.days.pop_first();
            }
        }
        self.dirty.store(true, Ordering::Relaxed);
        self.append_log(&format!("{} [{kind:?}] {app} {detail}", now.format("%Y-%m-%d %H:%M:%S")));
        if matches!(kind, Kind::Crash | Kind::Backup) {
            self.flush();
        }
    }

    pub fn add_protected_seconds(&self, seconds: u64) {
        lock(&self.data).days.entry(today_key()).or_default().protected_seconds += seconds;
        self.dirty.store(true, Ordering::Relaxed);
    }

    pub fn summary(&self) -> Summary {
        let data = lock(&self.data);
        let today = Local::now().date_naive();
        let mut summary = Summary {
            today: data.days.get(&today_key()).cloned().unwrap_or_default(),
            week: DayStats::default(),
            total: DayStats::default(),
            events: data.events.iter().cloned().collect(),
        };
        for (key, stats) in &data.days {
            summary.total.add(stats);
            let in_week = NaiveDate::parse_from_str(key, "%Y-%m-%d")
                .is_ok_and(|date| (today - date).num_days() < 7);
            if in_week {
                summary.week.add(stats);
            }
        }
        summary
    }

    pub fn clear(&self) {
        *lock(&self.data) = Data::default();
        self.dirty.store(true, Ordering::Relaxed);
        self.flush();
    }

    pub fn log_dir(&self) -> &PathBuf {
        &self.log_dir
    }

    pub fn flush(&self) {
        if !self.dirty.swap(false, Ordering::Relaxed) {
            return;
        }
        let json = match serde_json::to_string(&*lock(&self.data)) {
            Ok(json) => json,
            Err(_) => return,
        };
        let tmp = self.path.with_extension("json.tmp");
        let result = self
            .path
            .parent()
            .map_or(Ok(()), fs::create_dir_all)
            .and_then(|_| fs::write(&tmp, json))
            .and_then(|_| fs::rename(&tmp, &self.path));
        if let Err(error) = result {
            eprintln!("No se pudo guardar la actividad: {error}");
        }
    }

    fn append_log(&self, line: &str) {
        let path = self.log_dir.join("activity.log");
        if fs::metadata(&path).is_ok_and(|m| m.len() > MAX_LOG_BYTES) {
            let _ = fs::rename(&path, self.log_dir.join("activity.old.log"));
        }
        let _ = fs::create_dir_all(&self.log_dir);
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
            let _ = writeln!(file, "{line}");
        }
    }
}

fn today_key() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

/// Registra un evento desde cualquier módulo y avisa a la interfaz.
pub fn record(app: &AppHandle, kind: Kind, name: &str, detail: &str) {
    if let Some(activity) = app.try_state::<Arc<Activity>>() {
        activity.record(kind, name, detail);
        let _ = app.emit("activity", ());
    }
}
