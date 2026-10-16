use std::fs;
use std::path::PathBuf;
use chrono::Local;
use serde::{Deserialize, Serialize};

use crate::models::{AppSettings, DailyRecord, SavedSession};

#[derive(Debug, Serialize, Deserialize)]
pub struct PersistentData {
    pub today: DailyRecord,
    pub history: Vec<DailyRecord>,
    pub settings: AppSettings,
    /// Work session at the last save, so a restart can continue it.
    #[serde(default)]
    pub session: Option<SavedSession>,
}

/// Everything loaded at startup.
pub struct Loaded {
    pub today: DailyRecord,
    pub history: Vec<DailyRecord>,
    pub settings: AppSettings,
    pub session: Option<SavedSession>,
}

pub struct StorageManager {
    file_path: PathBuf,
}

impl StorageManager {
    pub fn new() -> Self {
        let app_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("HealthTasks");

        if !app_dir.exists() {
            let _ = fs::create_dir_all(&app_dir);
        }

        let file_path = app_dir.join("health_tasks_data.json");
        Self { file_path }
    }

    pub fn load_or_init(&self) -> Loaded {
        let current_date = Local::now().format("%Y-%m-%d").to_string();

        if self.file_path.exists() {
            if let Ok(content) = fs::read_to_string(&self.file_path) {
                if let Ok(mut data) = serde_json::from_str::<PersistentData>(&content) {
                    roll_over(&mut data.today, &mut data.history, &current_date);
                    return Loaded {
                        today: data.today,
                        history: data.history,
                        settings: data.settings,
                        session: data.session,
                    };
                }
            }
        }

        // Default initial state
        Loaded {
            today: DailyRecord::today(),
            history: Vec::new(),
            settings: AppSettings::default(),
            session: None,
        }
    }

    pub fn save(
        &self,
        today: &DailyRecord,
        history: &[DailyRecord],
        settings: &AppSettings,
        session: SavedSession,
    ) {
        let data = PersistentData {
            today: today.clone(),
            history: history.to_vec(),
            settings: settings.clone(),
            session: Some(session),
        };

        if let Ok(json) = serde_json::to_string_pretty(&data) {
            let _ = fs::write(&self.file_path, json);
        }
    }
}

/// Days of history kept: enough to show this month and last month in the calendar.
pub const HISTORY_DAYS: usize = 62;

/// If `today` is from an earlier date, moves it into history (trimmed to
/// [`HISTORY_DAYS`]) and starts a fresh record for `current_date`.
/// Returns true when it rolled over.
pub fn roll_over(today: &mut DailyRecord, history: &mut Vec<DailyRecord>, current_date: &str) -> bool {
    if today.date == current_date {
        return false;
    }
    let finished = std::mem::replace(today, DailyRecord::with_date(current_date));
    if !history.iter().any(|h| h.date == finished.date) {
        history.push(finished);
    }
    if history.len() > HISTORY_DAYS {
        let excess = history.len() - HISTORY_DAYS;
        history.drain(0..excess);
    }
    true
}
