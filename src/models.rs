use chrono::Local;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskCategory {
    Hydration,
    Movement,
    Vision,
    Posture,
    Mindfulness,
    Nutrition,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthTask {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: TaskCategory,
    pub completed: bool,
    pub current_count: u32,
    pub target_count: u32,
    pub unit: String,
}

impl HealthTask {
    pub fn new(
        id: &str,
        title: &str,
        description: &str,
        category: TaskCategory,
        target_count: u32,
        unit: &str,
    ) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            category,
            completed: false,
            current_count: 0,
            target_count,
            unit: unit.to_string(),
        }
    }

    pub fn toggle(&mut self) {
        self.completed = !self.completed;
        if self.completed {
            self.current_count = self.target_count;
        } else {
            self.current_count = 0;
        }
    }

    pub fn increment(&mut self, amount: u32) {
        self.current_count = self.current_count.saturating_add(amount);
        if self.current_count >= self.target_count {
            self.completed = true;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BreakType {
    EyeBreak20,
    StandStretch,
    DeepBreathing,
    PostureCheck,
}

impl BreakType {
    #[allow(dead_code)]
    pub fn title(&self) -> &'static str {
        match self {
            BreakType::EyeBreak20 => "20-20-20 Eye Break",
            BreakType::StandStretch => "Stand & Stretch",
            BreakType::DeepBreathing => "Deep Breathing",
            BreakType::PostureCheck => "Posture Check",
        }
    }

    pub fn duration_seconds(&self) -> u32 {
        match self {
            BreakType::EyeBreak20 => 20 * 60,   // 20 minutes
            BreakType::StandStretch => 45 * 60, // 45 minutes
            BreakType::DeepBreathing => 5 * 60, // 5 minutes
            BreakType::PostureCheck => 30 * 60, // 30 minutes
        }
    }

    pub fn instruction(&self) -> &'static str {
        match self {
            BreakType::EyeBreak20 => "Look at an object 20 feet away for 20 seconds to relax ciliary eye muscles.",
            BreakType::StandStretch => "Stand up, stretch your shoulders, neck, and lower back. Walk around.",
            BreakType::DeepBreathing => "Inhale deeply for 4 seconds, hold for 4 seconds, exhale for 6 seconds.",
            BreakType::PostureCheck => "Roll shoulders back, align ears with shoulders, feet flat on the floor.",
        }
    }

    #[allow(dead_code)]
    pub fn icon(&self) -> &'static str {
        match self {
            BreakType::EyeBreak20 => "👀",
            BreakType::StandStretch => "🧍",
            BreakType::DeepBreathing => "🌬️",
            BreakType::PostureCheck => "🧘",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityStatus {
    WaitingForInput,
    ActiveWorking,
    NoticeDue,
    AwayOnBreak,
}

impl ActivityStatus {
    #[allow(dead_code)]
    pub fn display_name(&self) -> &'static str {
        match self {
            ActivityStatus::WaitingForInput => "Waiting for input (Idle)",
            ActivityStatus::ActiveWorking => "Active working",
            ActivityStatus::NoticeDue => "Break Notice Due! ⚠️",
            ActivityStatus::AwayOnBreak => "Away on break 🌿",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivitySession {
    pub status: ActivityStatus,
    pub active_work_seconds: u32,
    pub session_start_time: Option<String>,
    pub last_break_time: Option<String>,
    pub last_idle_seconds: f64,
    pub notices_sent_today: u32,
    /// Seconds left in which input is ignored after "Take a Break", so the
    /// click itself doesn't start a new session (not persisted).
    #[serde(skip)]
    pub break_grace_secs: u32,
    /// The current time away has already been counted as a break.
    #[serde(default)]
    pub break_counted: bool,
    /// On a break started with "Take a Break": input asks whether to end it
    /// instead of starting a new session.
    #[serde(default)]
    pub manual_break: bool,
    /// Input arrived during a manual break; the controller asks the user.
    #[serde(skip)]
    pub break_input_pending: bool,
}

/// Time away from the computer that counts as a break.
pub const AWAY_BREAK_SECS: u32 = 10 * 60;

/// Work session as saved to disk, with the time it was saved (Unix seconds).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedSession {
    pub activity: ActivitySession,
    pub saved_at: i64,
}

/// After the work limit, the break notice repeats this often until a break.
pub const OVERTIME_REMINDER_SECS: u32 = 30 * 60;

/// How long input is ignored after the user starts a break manually, so the
/// click on "Take a Break" itself doesn't count.
pub const BREAK_GRACE_SECS: u32 = 3;
/// How long input is ignored after "Keep Resting", to step away.
pub const KEEP_RESTING_GRACE_SECS: u32 = 30;

impl Default for ActivitySession {
    fn default() -> Self {
        Self {
            status: ActivityStatus::WaitingForInput,
            active_work_seconds: 0,
            session_start_time: None,
            last_break_time: None,
            last_idle_seconds: 0.0,
            notices_sent_today: 0,
            break_grace_secs: 0,
            break_counted: false,
            manual_break: false,
            break_input_pending: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerState {
    pub break_type: BreakType,
    pub remaining_secs: u32,
    pub total_secs: u32,
    pub is_running: bool,
    pub completed_breaks_today: u32,
}

impl Default for TimerState {
    fn default() -> Self {
        let break_type = BreakType::EyeBreak20;
        let total_secs = break_type.duration_seconds();
        Self {
            break_type,
            remaining_secs: total_secs,
            total_secs,
            is_running: false,
            completed_breaks_today: 0,
        }
    }
}

impl TimerState {
    pub fn select_type(&mut self, break_type: BreakType) {
        self.break_type = break_type;
        self.total_secs = break_type.duration_seconds();
        self.remaining_secs = self.total_secs;
        self.is_running = false;
    }

    pub fn reset(&mut self) {
        self.remaining_secs = self.total_secs;
        self.is_running = false;
    }

    pub fn formatted_remaining(&self) -> String {
        let mins = self.remaining_secs / 60;
        let secs = self.remaining_secs % 60;
        format!("{:02}:{:02}", mins, secs)
    }

    #[allow(dead_code)]
    pub fn progress(&self) -> f64 {
        if self.total_secs == 0 {
            return 1.0;
        }
        1.0 - (self.remaining_secs as f64 / self.total_secs as f64)
    }
}

/// FDA guideline for healthy adults: up to 400 mg of caffeine a day.
pub const CAFFEINE_LIMIT_MG: u32 = 400;
/// Caffeine after this hour (local time) tends to affect sleep.
pub const CAFFEINE_CUTOFF_HOUR: u32 = 14;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoffeeKind {
    Espresso,
    Coffee,
    Latte,
}

impl CoffeeKind {
    pub fn title(&self) -> &'static str {
        match self {
            CoffeeKind::Espresso => "Espresso",
            CoffeeKind::Coffee => "Coffee",
            CoffeeKind::Latte => "Latte",
        }
    }

    /// Typical caffeine content in mg (single espresso shot, 240 ml brewed
    /// coffee, latte/cappuccino with a double shot).
    pub fn caffeine_mg(&self) -> u32 {
        match self {
            CoffeeKind::Espresso => 63,
            CoffeeKind::Coffee => 95,
            CoffeeKind::Latte => 126,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoffeeEntry {
    pub time: String, // HH:MM local
    pub kind: CoffeeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExerciseKind {
    Plank,
    Stretching,
    Walk,
}

impl ExerciseKind {
    /// Daily routine this exercise counts toward, and by how much.
    pub fn routine(&self) -> (&'static str, u32) {
        match self {
            ExerciseKind::Plank => ("exercise", 1), // +1 min of movement
            ExerciseKind::Stretching => ("stand_stretch", 1), // +1 session
            ExerciseKind::Walk => ("exercise", 10), // +10 min of movement
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExerciseEntry {
    pub time: String, // HH:MM local
    pub kind: ExerciseKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyRecord {
    pub date: String, // YYYY-MM-DD
    pub water_ml: u32,
    pub water_goal_ml: u32,
    pub tasks: Vec<HealthTask>,
    pub breaks_taken: u32,
    #[serde(default)]
    pub coffee_log: Vec<CoffeeEntry>,
    #[serde(default)]
    pub exercise_log: Vec<ExerciseEntry>,
    /// Seconds with keyboard or mouse use in the last minute.
    #[serde(default)]
    pub active_seconds: u32,
    /// First and last keyboard/mouse use that day, "HH:MM" local.
    #[serde(default)]
    pub first_active: Option<String>,
    #[serde(default)]
    pub last_active: Option<String>,
}

impl DailyRecord {
    pub fn today() -> Self {
        let date = Local::now().format("%Y-%m-%d").to_string();
        Self::with_date(&date)
    }

    pub fn with_date(date: &str) -> Self {
        let default_tasks = vec![
            HealthTask::new(
                "water",
                "Hydration Target",
                "Drink at least 2,000 ml of water throughout the day",
                TaskCategory::Hydration,
                2000,
                "ml",
            ),
            HealthTask::new(
                "eye_breaks",
                "20-20-20 Eye Breaks",
                "Rest your eyes 6 times throughout work sessions",
                TaskCategory::Vision,
                6,
                "times",
            ),
            HealthTask::new(
                "stand_stretch",
                "Stand & Stretch Sessions",
                "Get up and stretch for at least 5 minutes every hour",
                TaskCategory::Movement,
                5,
                "times",
            ),
            HealthTask::new(
                "posture",
                "Ergonomic Posture Check",
                "Adjust screen height, align spine, drop shoulders",
                TaskCategory::Posture,
                4,
                "checks",
            ),
            HealthTask::new(
                "exercise",
                "Daily Movement / Exercise",
                "At least 30 minutes of walking, workout, or cardio",
                TaskCategory::Movement,
                30,
                "min",
            ),
            HealthTask::new(
                "mindful_lunch",
                "Screen-Free Meal",
                "Take a deliberate 20-minute break away from screens to eat",
                TaskCategory::Mindfulness,
                1,
                "meal",
            ),
            HealthTask::new(
                "vitamins",
                "Vitamins & Supplements",
                "Take daily multivitamins, Vitamin D, or prescribed supplements",
                TaskCategory::Nutrition,
                1,
                "dose",
            ),
        ];

        Self {
            date: date.to_string(),
            water_ml: 0,
            water_goal_ml: 2000,
            tasks: default_tasks,
            breaks_taken: 0,
            coffee_log: Vec::new(),
            exercise_log: Vec::new(),
            active_seconds: 0,
            first_active: None,
            last_active: None,
        }
    }

    pub fn coffee_cups(&self) -> u32 {
        self.coffee_log.len() as u32
    }

    pub fn caffeine_mg(&self) -> u32 {
        self.coffee_log.iter().map(|c| c.kind.caffeine_mg()).sum()
    }

    pub fn last_coffee_time(&self) -> Option<&str> {
        self.coffee_log.last().map(|c| c.time.as_str())
    }

    pub fn completion_ratio(&self) -> f64 {
        if self.tasks.is_empty() {
            return 0.0;
        }
        let completed = self.tasks.iter().filter(|t| t.completed).count();
        completed as f64 / self.tasks.len() as f64
    }

    pub fn completed_count(&self) -> usize {
        self.tasks.iter().filter(|t| t.completed).count()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub sound_enabled: bool,
    pub water_step_ml: u32,
    pub selected_tab: usize,
    pub work_interval_minutes: u32,
    pub idle_reset_minutes: u32,
    pub notice_title: String,
    pub notice_message: String,
    pub show_banner_notification: bool,
    pub show_alert_dialog: bool,
    pub sound_name: String,
    /// Floating control center: visible, collapsed to a tile, and its top-left
    /// corner in screen coordinates (None = default spot).
    #[serde(default = "default_true")]
    pub control_center_visible: bool,
    #[serde(default)]
    pub control_center_collapsed: bool,
    #[serde(default)]
    pub control_center_top_left: Option<(f64, f64)>,
}

fn default_true() -> bool {
    true
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            sound_enabled: true,
            water_step_ml: 250,
            selected_tab: 0,
            work_interval_minutes: 60,
            idle_reset_minutes: 5,
            notice_title: "Time for a Health Break! 🧘".to_string(),
            notice_message: "You've been using your keyboard & mouse for 60 minutes. Stand up, stretch, and rest your eyes!".to_string(),
            show_banner_notification: true,
            show_alert_dialog: true,
            sound_name: "Glass".to_string(),
            control_center_visible: true,
            control_center_collapsed: false,
            control_center_top_left: None,
        }
    }
}
