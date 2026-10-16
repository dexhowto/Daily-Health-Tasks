use chrono::{Datelike, Local, NaiveDate};
use objc2::rc::Retained;
use objc2_app_kit::{NSColor, NSStackView};
use objc2_foundation::MainThreadMarker;

use crate::action_handler::HealthTasksActionHandler;
use crate::app_state::AppState;
use crate::models::{DailyRecord, CAFFEINE_LIMIT_MG};
use crate::ui::calendar_card::{
    exercised, format_active, format_active_short, water_goal_met, CalendarCard, CalendarMode,
};
use crate::ui::components::{create_page_header, StatTile, UIComponents};
use crate::ui::dashboard_view::caffeine_tint;

/// Statistics page: monthly summary tiles and the month calendar.
pub struct StatisticsView {
    pub container: Retained<NSStackView>,
    pub calendar: CalendarCard,
    water_tile: StatTile,
    exercise_tile: StatTile,
    active_tile: StatTile,
    coffee_tile: StatTile,
}

impl StatisticsView {
    pub fn build(handler: &HealthTasksActionHandler, state: &AppState, mtm: MainThreadMarker) -> Self {
        let container = UIComponents::create_v_stack(20.0, mtm);
        UIComponents::add_full_width(
            &container,
            &create_page_header("Statistics", "Pick a month to see your water, exercise and screen time.", mtm),
        );

        let water_tile = StatTile::new("Water goal", "drop.fill", &NSColor::systemBlueColor(), mtm);
        let exercise_tile = StatTile::new("Exercise", "figure.run", &NSColor::systemOrangeColor(), mtm);
        let active_tile = StatTile::new("Active time", "desktopcomputer", &NSColor::systemIndigoColor(), mtm);
        let coffee_tile = StatTile::new("Caffeine", "cup.and.saucer.fill", &NSColor::systemBrownColor(), mtm);
        // Screen time has no goal, so no bar.
        active_tile.bar.track.setHidden(true);
        let tiles = UIComponents::create_equal_row(
            &[&water_tile.view, &exercise_tile.view, &active_tile.view, &coffee_tile.view],
            14.0,
            mtm,
        );
        UIComponents::add_full_width(&container, &tiles);

        let calendar = CalendarCard::build(handler, CalendarMode::Month, mtm);
        UIComponents::add_full_width(&container, &calendar.view);

        let view = Self { container, calendar, water_tile, exercise_tile, active_tile, coffee_tile };
        view.refresh(state);
        view
    }

    pub fn refresh(&self, state: &AppState) {
        self.calendar.refresh(state);

        let month = self.calendar.month();
        let today = Local::now().date_naive();
        let days: Vec<&DailyRecord> = state
            .history
            .iter()
            .chain(std::iter::once(&state.today))
            .filter(|d| {
                NaiveDate::parse_from_str(&d.date, "%Y-%m-%d")
                    .is_ok_and(|date| date.year() == month.year() && date.month() == month.month())
            })
            .collect();

        // Days so far in this month (all days for past months).
        let elapsed = if month.year() == today.year() && month.month() == today.month() {
            today.day()
        } else {
            let next = if month.month() == 12 {
                NaiveDate::from_ymd_opt(month.year() + 1, 1, 1)
            } else {
                NaiveDate::from_ymd_opt(month.year(), month.month() + 1, 1)
            };
            next.map_or(30, |n| n.pred_opt().map_or(30, |d| d.day()))
        }
        .max(1);
        let plural = |n: usize| if n == 1 { "day" } else { "days" };

        let water_days = days.iter().filter(|d| water_goal_met(d)).count();
        self.water_tile.update(
            &format!("{water_days} {}", plural(water_days)),
            &format!("goal met, of {elapsed} days"),
            water_days as f64 / elapsed as f64,
        );

        let exercise_days = days.iter().filter(|d| exercised(d)).count();
        self.exercise_tile.update(
            &format!("{exercise_days} {}", plural(exercise_days)),
            &format!("with exercise, of {elapsed} days"),
            exercise_days as f64 / elapsed as f64,
        );

        let active: u32 = days.iter().map(|d| d.active_seconds).sum();
        let tracked = days.iter().filter(|d| d.active_seconds > 0).count().max(1) as u32;
        self.active_tile.update(
            &format_active_short(active),
            &format!("avg {} per day", format_active(active / tracked)),
            0.0,
        );

        let caffeine: u32 = days.iter().map(|d| d.caffeine_mg()).sum();
        let avg_mg = caffeine / days.len().max(1) as u32;
        let over = days.iter().filter(|d| d.caffeine_mg() > CAFFEINE_LIMIT_MG).count();
        let detail = if over > 0 {
            format!("over {CAFFEINE_LIMIT_MG} mg on {over} {}", plural(over))
        } else {
            "avg per day".to_string()
        };
        self.coffee_tile.update(&format!("{avg_mg} mg"), &detail, avg_mg as f64 / CAFFEINE_LIMIT_MG as f64);
        self.coffee_tile.bar.set_tint(&caffeine_tint(avg_mg));
    }
}
