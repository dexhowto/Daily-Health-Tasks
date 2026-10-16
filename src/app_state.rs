use std::sync::{Mutex, OnceLock};
use chrono::Local;

use crate::audio::SoundEffects;
use crate::models::{
    ActivitySession, ActivityStatus, AppSettings, BreakType, CoffeeEntry, CoffeeKind, DailyRecord,
    ExerciseEntry, ExerciseKind, SavedSession, TimerState, AWAY_BREAK_SECS, BREAK_GRACE_SECS,
};
use crate::storage::StorageManager;

pub struct AppState {
    pub today: DailyRecord,
    pub history: Vec<DailyRecord>,
    pub timer: TimerState,
    pub activity: ActivitySession,
    /// A break notice is waiting for a screen recording to finish (not persisted).
    pub notice_held: bool,
    pub settings: AppSettings,
    storage: StorageManager,
}

impl AppState {
    pub fn new() -> Self {
        let storage = StorageManager::new();
        let loaded = storage.load_or_init();
        let (today, history, settings) = (loaded.today, loaded.history, loaded.settings);
        let timer = TimerState::default();
        let activity = restore_session(loaded.session, &settings);

        Self {
            today,
            history,
            timer,
            activity,
            notice_held: false,
            settings,
            storage,
        }
    }

    pub fn save(&self) {
        let session = SavedSession {
            activity: self.activity.clone(),
            saved_at: Local::now().timestamp(),
        };
        self.storage.save(&self.today, &self.history, &self.settings, session);
    }

    pub fn add_water(&mut self, amount_ml: i32) {
        if amount_ml > 0 {
            self.today.water_ml = self.today.water_ml.saturating_add(amount_ml as u32);
            if self.settings.sound_enabled {
                SoundEffects::play_water_logged();
            }
        } else {
            let neg = (-amount_ml) as u32;
            self.today.water_ml = self.today.water_ml.saturating_sub(neg);
        }

        if let Some(task) = self.today.tasks.iter_mut().find(|t| t.id == "water") {
            task.current_count = self.today.water_ml;
            task.completed = self.today.water_ml >= task.target_count;
        }

        self.save();
    }

    pub fn log_coffee(&mut self, kind: CoffeeKind) {
        self.today.coffee_log.push(CoffeeEntry {
            time: Local::now().format("%H:%M").to_string(),
            kind,
        });
        if self.settings.sound_enabled {
            SoundEffects::play_named("Pop");
        }
        self.save();
    }

    pub fn log_exercise(&mut self, kind: ExerciseKind) {
        self.today.exercise_log.push(ExerciseEntry {
            time: Local::now().format("%H:%M").to_string(),
            kind,
        });
        if self.settings.sound_enabled {
            SoundEffects::play_named("Pop");
        }
        let (routine, amount) = kind.routine();
        self.increment_task(routine, amount); // also saves
    }

    pub fn undo_coffee(&mut self) {
        if self.today.coffee_log.pop().is_some() {
            self.save();
        }
    }

    pub fn toggle_task(&mut self, task_id: &str) {
        if let Some(task) = self.today.tasks.iter_mut().find(|t| t.id == task_id) {
            task.toggle();
            let was_completed = task.completed;
            if was_completed && self.settings.sound_enabled {
                SoundEffects::play_task_complete();
            }
        }
        self.save();
    }

    pub fn increment_task(&mut self, task_id: &str, amount: u32) {
        if let Some(task) = self.today.tasks.iter_mut().find(|t| t.id == task_id) {
            task.increment(amount);
            if task.completed && self.settings.sound_enabled {
                SoundEffects::play_task_complete();
            }
        }
        self.save();
    }

    pub fn select_break_type(&mut self, break_type: BreakType) {
        self.timer.select_type(break_type);
    }

    pub fn toggle_timer(&mut self) -> bool {
        self.timer.is_running = !self.timer.is_running;
        if self.settings.sound_enabled {
            SoundEffects::play_button_click();
        }
        self.timer.is_running
    }

    pub fn reset_timer(&mut self) {
        self.timer.reset();
        if self.settings.sound_enabled {
            SoundEffects::play_button_click();
        }
    }

    /// `quiet` skips the finish chime (e.g. while the screen is being recorded).
    pub fn tick_timer(&mut self, quiet: bool) -> Option<BreakType> {
        if !self.timer.is_running {
            return None;
        }

        if self.timer.remaining_secs > 0 {
            self.timer.remaining_secs -= 1;
        }

        if self.timer.remaining_secs == 0 {
            self.timer.is_running = false;
            let finished_break = self.timer.break_type;
            self.timer.completed_breaks_today += 1;
            self.today.breaks_taken += 1;

            match finished_break {
                BreakType::EyeBreak20 => self.increment_task("eye_breaks", 1),
                BreakType::StandStretch => self.increment_task("stand_stretch", 1),
                BreakType::PostureCheck => self.increment_task("posture", 1),
                _ => {}
            }

            if self.settings.sound_enabled && !quiet {
                SoundEffects::play_timer_finish();
            }

            self.save();
            return Some(finished_break);
        }

        None
    }

    /// Background activity monitor: updates active work timer based on system idle seconds
    pub fn tick_activity_monitor(
        &mut self,
        idle_seconds: f64,
    ) -> Option<(String, String, String, bool, bool)> {
        self.activity.last_idle_seconds = idle_seconds;

        // New day while the app runs: archive yesterday, start fresh.
        let current_date = Local::now().format("%Y-%m-%d").to_string();
        if crate::storage::roll_over(&mut self.today, &mut self.history, &current_date) {
            self.save();
        }

        // Daily time on the computer (saved once a minute), plus the first
        // and last use of the day, taken from the time of the last input.
        if idle_seconds < 60.0 {
            let last_input = Local::now() - chrono::Duration::milliseconds((idle_seconds * 1000.0) as i64);
            // Input just before midnight belongs to yesterday.
            let hhmm = if last_input.date_naive() == Local::now().date_naive() {
                last_input.format("%H:%M").to_string()
            } else {
                "00:00".to_string()
            };
            if self.today.first_active.is_none() {
                self.today.first_active = Some(hhmm.clone());
                self.save();
            }
            self.today.last_active = Some(hhmm);
            self.today.active_seconds += 1;
            if self.today.active_seconds % 60 == 0 {
                self.save();
            }
        }

        // Just after "Take a Break": stay on break whatever the input, then
        // the next key press or mouse move starts a new session (step 2).
        if self.activity.break_grace_secs > 0 {
            self.activity.break_grace_secs -= 1;
            return None;
        }
        let idle_reset_secs = (self.settings.idle_reset_minutes * 60) as f64;
        let work_limit_secs = self.settings.work_interval_minutes * 60;

        // 1. Away longer than idle_reset_minutes: the session stops and the
        //    time away is shown instead (from `last_idle_seconds`).
        if idle_seconds >= idle_reset_secs {
            if self.activity.status != ActivityStatus::AwayOnBreak {
                self.activity.status = ActivityStatus::AwayOnBreak;
                self.activity.active_work_seconds = 0;
                self.activity.break_counted = false;
                self.activity.last_break_time = Some(Local::now().format("%H:%M:%S").to_string());
            }
        }

        // Away for 10+ minutes counts as one break.
        if self.activity.status == ActivityStatus::AwayOnBreak
            && !self.activity.break_counted
            && idle_seconds >= AWAY_BREAK_SECS as f64
        {
            self.activity.break_counted = true;
            self.today.breaks_taken += 1;
            self.save();
        }
        if idle_seconds >= idle_reset_secs {
            return None;
        }

        // 2. If user pressed key or moved mouse recently (< 2.0s):
        if idle_seconds < 2.0 {
            if self.activity.status == ActivityStatus::WaitingForInput
                || self.activity.status == ActivityStatus::AwayOnBreak
                || self.activity.status == ActivityStatus::NoticeDue
            {
                self.activity.status = ActivityStatus::ActiveWorking;
                self.activity.active_work_seconds = 0;
                self.activity.break_counted = false;
                self.activity.session_start_time = Some(Local::now().format("%H:%M:%S").to_string());
            }
        }

        // 3. If actively working, count up active work seconds
        if self.activity.status == ActivityStatus::ActiveWorking {
            if idle_seconds < 60.0 {
                self.activity.active_work_seconds += 1;
            }

            // Check if limit reached!
            if self.activity.active_work_seconds >= work_limit_secs {
                self.activity.status = ActivityStatus::NoticeDue;
                // The notice is a reminder, not a break; breaks are counted
                // when you actually step away (or use Take a Break).
                self.activity.notices_sent_today += 1;

                self.save();

                return Some((
                    self.settings.notice_title.clone(),
                    self.settings.notice_message.clone(),
                    self.settings.sound_name.clone(),
                    self.settings.show_banner_notification,
                    self.settings.show_alert_dialog,
                ));
            }
        }

        None
    }

    pub fn manual_take_break(&mut self) {
        self.activity.status = ActivityStatus::AwayOnBreak;
        self.activity.active_work_seconds = 0;
        self.activity.break_grace_secs = BREAK_GRACE_SECS;
        self.activity.break_counted = true; // counted now, not again after 10 min away
        self.activity.last_break_time = Some(Local::now().format("%H:%M:%S").to_string());
        self.today.breaks_taken += 1;
        if self.settings.sound_enabled {
            SoundEffects::play_button_click();
        }
        self.save();
    }

    pub fn calculate_streak(&self) -> u32 {
        let mut streak = 0;
        if self.today.completion_ratio() >= 0.5 {
            streak += 1;
        }

        for day in self.history.iter().rev() {
            if day.completion_ratio() >= 0.5 {
                streak += 1;
            } else {
                break;
            }
        }

        streak
    }

    pub fn menu_bar_title(&self) -> String {
        let mins = self.activity.active_work_seconds / 60;
        let secs = self.activity.active_work_seconds % 60;
        let limit = self.settings.work_interval_minutes;

        match self.activity.status {
            ActivityStatus::ActiveWorking => {
                format!("⏱️ {:02}:{:02} / {:02}:00", mins, secs, limit)
            }
            ActivityStatus::NoticeDue => {
                format!("⚠️ {:02}:{:02} - Break Due!", mins, secs)
            }
            ActivityStatus::AwayOnBreak if self.activity.break_grace_secs > 0 => {
                format!("🌿 Break · input paused {:02}", self.activity.break_grace_secs)
            }
            ActivityStatus::AwayOnBreak => {
                let away_mins = (self.activity.last_idle_seconds / 60.0).floor() as u32;
                let away_secs = (self.activity.last_idle_seconds % 60.0).floor() as u32;
                format!("🌿 Away ({:02}:{:02})", away_mins, away_secs)
            }
            ActivityStatus::WaitingForInput => {
                "⏱️ 00:00 (Ready)".to_string()
            }
        }
    }
}

/// Continues the work session from the last run if the app was closed for
/// less than the idle-reset time (otherwise that gap was a break) and it's
/// still the same day.
fn restore_session(saved: Option<SavedSession>, settings: &AppSettings) -> ActivitySession {
    let Some(saved) = saved else {
        return ActivitySession::default();
    };
    let gap = Local::now().timestamp() - saved.saved_at;
    let same_day = chrono::DateTime::from_timestamp(saved.saved_at, 0)
        .map(|t| t.with_timezone(&Local).date_naive() == Local::now().date_naive())
        .unwrap_or(false);
    let working = matches!(
        saved.activity.status,
        ActivityStatus::ActiveWorking | ActivityStatus::NoticeDue
    );
    if working && same_day && (0..(settings.idle_reset_minutes as i64 * 60)).contains(&gap) {
        ActivitySession {
            break_grace_secs: 0,
            notices_sent_today: saved.activity.notices_sent_today,
            ..saved.activity
        }
    } else {
        ActivitySession {
            notices_sent_today: if same_day { saved.activity.notices_sent_today } else { 0 },
            ..ActivitySession::default()
        }
    }
}

static GLOBAL_STATE: OnceLock<Mutex<AppState>> = OnceLock::new();

pub fn state() -> &'static Mutex<AppState> {
    GLOBAL_STATE.get_or_init(|| Mutex::new(AppState::new()))
}
