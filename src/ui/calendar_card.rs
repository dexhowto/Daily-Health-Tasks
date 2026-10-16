//! Activity calendar: each day shows whether the water goal was met and
//! whether you exercised; clicking a day shows its details. Used as a
//! "This Week" strip on Today and as the month view on Statistics.

use std::cell::Cell;
use std::collections::HashMap;

use chrono::{Datelike, Duration, Local, NaiveDate};
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, Sel};
use objc2::sel;
use objc2_app_kit::{
    NSBox, NSButton, NSCellImagePosition, NSColor, NSFont, NSImageView, NSPopUpButton,
    NSStackView, NSStackViewGravity, NSTextField, NSView,
};
use objc2_foundation::{MainThreadMarker, NSString};

use crate::action_handler::HealthTasksActionHandler;
use crate::app_state::AppState;
use crate::models::{DailyRecord, ExerciseKind, CAFFEINE_LIMIT_MG};
use crate::ui::components::{UIComponents, WEIGHT_MEDIUM, WEIGHT_SEMIBOLD};

const WEEKS: usize = 6;
const CELL_HEIGHT: f64 = 50.0;
/// Months offered in the month picker, counting back from this month.
const PICKER_MONTHS: i32 = 12;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CalendarMode {
    /// The current week (Mon–Sun) only, no navigation.
    Week,
    /// A full month with a month picker and arrows.
    Month,
}

struct DayCell {
    view: Retained<NSBox>,
    number: Retained<NSTextField>,
    active: Retained<NSTextField>,
    water: Retained<NSImageView>,
    exercise: Retained<NSImageView>,
}

struct DetailRow {
    icon: Retained<NSImageView>,
    value: Retained<NSTextField>,
}

pub struct CalendarCard {
    pub view: Retained<NSBox>,
    mode: CalendarMode,
    month_label: Retained<NSTextField>,
    month_picker: Retained<NSPopUpButton>,
    week_rows: Vec<Retained<NSStackView>>,
    cells: Vec<DayCell>,
    detail_title: Retained<NSTextField>,
    detail_empty: Retained<NSTextField>,
    detail_rows: Retained<NSStackView>,
    water_row: DetailRow,
    exercise_row: DetailRow,
    habits_row: DetailRow,
    coffee_row: DetailRow,
    breaks_row: DetailRow,
    active_row: DetailRow,
    span_row: DetailRow,
    /// First day of the month on screen, and the selected day.
    month: Cell<NaiveDate>,
    selected: Cell<NaiveDate>,
}

fn first_of_month(date: NaiveDate) -> NaiveDate {
    date.with_day(1).unwrap()
}

/// Monday on or before the 1st: the first cell of the grid.
fn grid_start(month: NaiveDate) -> NaiveDate {
    month - Duration::days(month.weekday().num_days_from_monday() as i64)
}

fn add_months(month: NaiveDate, delta: i32) -> NaiveDate {
    let total = month.year() * 12 + month.month0() as i32 + delta;
    NaiveDate::from_ymd_opt(total.div_euclid(12), total.rem_euclid(12) as u32 + 1, 1).unwrap()
}

/// Monday of the current week.
fn this_week_start() -> NaiveDate {
    let today = Local::now().date_naive();
    today - Duration::days(today.weekday().num_days_from_monday() as i64)
}

fn months_between(from: NaiveDate, to: NaiveDate) -> i32 {
    (to.year() - from.year()) * 12 + to.month() as i32 - from.month() as i32
}

fn task_count(day: &DailyRecord, id: &str) -> u32 {
    day.tasks.iter().find(|t| t.id == id).map_or(0, |t| t.current_count)
}

pub fn water_goal_met(day: &DailyRecord) -> bool {
    day.water_goal_ml > 0 && day.water_ml >= day.water_goal_ml
}

pub fn exercised(day: &DailyRecord) -> bool {
    !day.exercise_log.is_empty() || task_count(day, "exercise") > 0 || task_count(day, "stand_stretch") > 0
}

fn exercise_summary(day: &DailyRecord) -> String {
    if !day.exercise_log.is_empty() {
        let count = |kind| day.exercise_log.iter().filter(|e| e.kind == kind).count();
        return [
            ("Plank", count(ExerciseKind::Plank)),
            ("Stretching", count(ExerciseKind::Stretching)),
            ("Walk", count(ExerciseKind::Walk)),
        ]
        .iter()
        .filter(|(_, n)| *n > 0)
        .map(|(name, n)| format!("{name} ×{n}"))
        .collect::<Vec<_>>()
        .join(" · ");
    }
    let (minutes, stretches) = (task_count(day, "exercise"), task_count(day, "stand_stretch"));
    match (minutes, stretches) {
        (0, 0) => "None".into(),
        (m, 0) => format!("{m} min movement"),
        (0, s) => format!("{s} stretch sessions"),
        (m, s) => format!("{m} min movement · {s} stretch sessions"),
    }
}

/// Compact duration for tight spots: "3h 12m", "45m".
pub fn format_active_short(secs: u32) -> String {
    let (hours, minutes) = (secs / 3600, secs / 60 % 60);
    if hours > 0 { format!("{hours}h {minutes}m") } else { format!("{minutes}m") }
}

pub fn format_active(secs: u32) -> String {
    let minutes = secs / 60;
    match (minutes / 60, minutes % 60) {
        (0, 0) if secs == 0 => "Not tracked".into(),
        (0, 0) => "Less than a minute".into(),
        (0, m) => format!("{m} min"),
        (h, m) => format!("{h} h {m} min"),
    }
}

impl CalendarCard {
    pub fn build(handler: &HealthTasksActionHandler, mode: CalendarMode, mtm: MainThreadMarker) -> Self {
        let target: &AnyObject = handler;
        let select_day = match mode {
            CalendarMode::Week => sel!(onSelectWeekDay:),
            CalendarMode::Month => sel!(onSelectCalendarDay:),
        };
        let (view, stack) = UIComponents::create_card("", mtm);
        stack.setSpacing(10.0);

        // Header: month name and navigation.
        let header = UIComponents::create_h_stack(8.0, mtm);
        header.addView_inGravity(
            &UIComponents::create_symbol_view("calendar", 14.0, &NSColor::systemRedColor(), mtm),
            NSStackViewGravity::Leading,
        );
        let month_label = UIComponents::create_weighted_label("", 15.0, WEIGHT_SEMIBOLD, mtm);
        header.addView_inGravity(&month_label, NSStackViewGravity::Leading);
        let nav = |symbol: &str, title: &str, action: Sel| {
            let btn = unsafe {
                NSButton::buttonWithTitle_target_action(&NSString::from_str(title), Some(target), Some(action), mtm)
            };
            btn.setBordered(false);
            btn.setContentTintColor(Some(&NSColor::secondaryLabelColor()));
            btn.setFont(Some(&NSFont::systemFontOfSize_weight(12.0, WEIGHT_MEDIUM)));
            if !symbol.is_empty() {
                if let Some(img) = UIComponents::symbol(symbol, 13.0, WEIGHT_SEMIBOLD) {
                    btn.setImage(Some(&img));
                    btn.setImagePosition(NSCellImagePosition::ImageOnly);
                }
            }
            btn
        };
        let month_picker = NSPopUpButton::initWithFrame_pullsDown(
            mtm.alloc(),
            objc2_foundation::NSRect::new(objc2_foundation::NSPoint::new(0.0, 0.0), objc2_foundation::NSSize::new(160.0, 24.0)),
            false,
        );
        let this_month = first_of_month(Local::now().date_naive());
        for back in 0..PICKER_MONTHS {
            let month = add_months(this_month, -back);
            month_picker.addItemWithTitle(&NSString::from_str(&month.format("%B %Y").to_string()));
        }
        unsafe {
            month_picker.setTarget(Some(target));
            month_picker.setAction(Some(sel!(onCalendarMonthPicked:)));
        }
        if mode == CalendarMode::Month {
            header.addView_inGravity(&nav("chevron.left", "", sel!(onCalendarPrev:)), NSStackViewGravity::Trailing);
            header.addView_inGravity(&month_picker, NSStackViewGravity::Trailing);
            header.addView_inGravity(&nav("chevron.right", "", sel!(onCalendarNext:)), NSStackViewGravity::Trailing);
            header.addView_inGravity(&nav("", "Today", sel!(onCalendarToday:)), NSStackViewGravity::Trailing);
        }
        UIComponents::add_full_width(&stack, &header);

        // Weekday names.
        let names: Vec<Retained<NSView>> = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
            .iter()
            .map(|d| {
                let l = UIComponents::create_caption(d, mtm);
                l.setAlignment(objc2_app_kit::NSTextAlignment::Center);
                Retained::into_super(Retained::into_super(l))
            })
            .collect();
        let refs: Vec<&NSView> = names.iter().map(|v| &**v).collect();
        UIComponents::add_full_width(&stack, &UIComponents::create_equal_row(&refs, 6.0, mtm));

        // 6 × 7 day cells, reused for every month.
        let grid = UIComponents::create_v_stack(6.0, mtm);
        let mut cells = Vec::new();
        let mut week_rows = Vec::new();
        for week in 0..WEEKS {
            let views: Vec<Retained<NSView>> = (0..7)
                .map(|day| {
                    let cell = Self::build_cell(week * 7 + day, target, select_day, mtm);
                    let v = Retained::into_super(cell.view.clone());
                    cells.push(cell);
                    v
                })
                .collect();
            let refs: Vec<&NSView> = views.iter().map(|v| &**v).collect();
            let row = UIComponents::create_equal_row(&refs, 6.0, mtm);
            UIComponents::add_full_width(&grid, &row);
            week_rows.push(row);
        }
        UIComponents::add_full_width(&stack, &grid);

        // Legend.
        let legend = UIComponents::create_h_stack(12.0, mtm);
        for (symbol, tint, text) in [
            ("drop.fill", NSColor::systemBlueColor(), "Water goal met"),
            ("figure.run", NSColor::systemOrangeColor(), "Exercised"),
            ("desktopcomputer", NSColor::systemIndigoColor(), "Time on computer"),
        ] {
            legend.addView_inGravity(&UIComponents::create_symbol_view(symbol, 10.0, &tint, mtm), NSStackViewGravity::Leading);
            legend.addView_inGravity(&UIComponents::create_secondary_label(text, 11.0, mtm), NSStackViewGravity::Leading);
        }
        stack.addView_inGravity(&legend, NSStackViewGravity::Top);

        // Details of the selected day.
        UIComponents::add_full_width(&stack, &UIComponents::create_separator(mtm));
        let detail_title = UIComponents::create_weighted_label("", 14.0, WEIGHT_SEMIBOLD, mtm);
        stack.addView_inGravity(&detail_title, NSStackViewGravity::Top);
        let detail_empty = UIComponents::create_secondary_label("No activity recorded for this day.", 12.0, mtm);
        stack.addView_inGravity(&detail_empty, NSStackViewGravity::Top);
        let detail_rows = UIComponents::create_v_stack(8.0, mtm);
        let row = |label: &str| {
            let (view, detail) = Self::build_detail_row(label, mtm);
            UIComponents::add_full_width(&detail_rows, &view);
            detail
        };
        let water_row = row("Water");
        let exercise_row = row("Exercise");
        let habits_row = row("Habits");
        let coffee_row = row("Coffee");
        let breaks_row = row("Breaks");
        let active_row = row("Active on computer");
        let span_row = row("Computer use");
        UIComponents::add_full_width(&stack, &detail_rows);

        let today = Local::now().date_naive();
        Self {
            view,
            mode,
            month_label,
            month_picker,
            week_rows,
            cells,
            detail_title,
            detail_empty,
            detail_rows,
            water_row,
            exercise_row,
            habits_row,
            coffee_row,
            breaks_row,
            active_row,
            span_row,
            month: Cell::new(first_of_month(today)),
            selected: Cell::new(today),
        }
    }

    fn build_cell(index: usize, target: &AnyObject, action: Sel, mtm: MainThreadMarker) -> DayCell {
        let view = UIComponents::create_rounded_box(8.0, mtm);
        view.heightAnchor().constraintEqualToConstant(CELL_HEIGHT).setActive(true);

        let number = UIComponents::create_weighted_label("", 12.0, WEIGHT_SEMIBOLD, mtm);
        number.setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(12.0, WEIGHT_SEMIBOLD)));
        let water = UIComponents::create_symbol_view("drop.fill", 10.0, &NSColor::systemBlueColor(), mtm);
        let exercise = UIComponents::create_symbol_view("figure.run", 10.0, &NSColor::systemOrangeColor(), mtm);
        let icons = UIComponents::create_h_stack(3.0, mtm);
        icons.addView_inGravity(&water, NSStackViewGravity::Leading);
        icons.addView_inGravity(&exercise, NSStackViewGravity::Leading);
        // Time active on the computer that day.
        let active = UIComponents::create_secondary_label("", 10.0, mtm);
        active.setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(10.0, WEIGHT_MEDIUM)));
        active.setTextColor(Some(&NSColor::systemIndigoColor()));

        for sub in [&*number as &NSView, &icons, &active] {
            sub.setTranslatesAutoresizingMaskIntoConstraints(false);
            view.addSubview(sub);
        }
        // Invisible button over the whole cell to select the day.
        let button = unsafe {
            NSButton::buttonWithTitle_target_action(&NSString::from_str(""), Some(target), Some(action), mtm)
        };
        button.setTransparent(true);
        button.setTag(index as isize);
        button.setTranslatesAutoresizingMaskIntoConstraints(false);
        view.addSubview(&button);
        UIComponents::activate(&[
            number.topAnchor().constraintEqualToAnchor_constant(&view.topAnchor(), 6.0),
            number.leadingAnchor().constraintEqualToAnchor_constant(&view.leadingAnchor(), 8.0),
            icons.bottomAnchor().constraintEqualToAnchor_constant(&view.bottomAnchor(), -6.0),
            icons.leadingAnchor().constraintEqualToAnchor_constant(&view.leadingAnchor(), 8.0),
            active.trailingAnchor().constraintEqualToAnchor_constant(&view.trailingAnchor(), -6.0),
            active.centerYAnchor().constraintEqualToAnchor(&icons.centerYAnchor()),
            button.leadingAnchor().constraintEqualToAnchor(&view.leadingAnchor()),
            button.trailingAnchor().constraintEqualToAnchor(&view.trailingAnchor()),
            button.topAnchor().constraintEqualToAnchor(&view.topAnchor()),
            button.bottomAnchor().constraintEqualToAnchor(&view.bottomAnchor()),
        ]);
        DayCell { view, number, active, water, exercise }
    }

    fn build_detail_row(label: &str, mtm: MainThreadMarker) -> (Retained<NSStackView>, DetailRow) {
        let row = UIComponents::create_h_stack(8.0, mtm);
        let icon = UIComponents::create_symbol_view("circle", 13.0, &NSColor::tertiaryLabelColor(), mtm);
        row.addView_inGravity(&icon, NSStackViewGravity::Leading);
        row.addView_inGravity(&UIComponents::create_label(label, 13.0, false, mtm), NSStackViewGravity::Leading);
        let value = UIComponents::create_secondary_label("", 13.0, mtm);
        row.addView_inGravity(&value, NSStackViewGravity::Trailing);
        (row, DetailRow { icon, value })
    }

    // --- Navigation ---

    pub fn show_previous_month(&self) {
        self.month.set(add_months(self.month.get(), -1));
    }

    pub fn show_next_month(&self) {
        self.month.set(add_months(self.month.get(), 1));
    }

    pub fn show_today(&self) {
        let today = Local::now().date_naive();
        self.month.set(first_of_month(today));
        self.selected.set(today);
    }

    /// Shows the month at `index` in the picker (0 = this month).
    pub fn show_picked_month(&self, index: isize) {
        let this_month = first_of_month(Local::now().date_naive());
        self.month.set(add_months(this_month, -(index.max(0) as i32)));
    }

    /// First day of the month on screen.
    pub fn month(&self) -> NaiveDate {
        self.month.get()
    }

    /// Date shown in the first cell.
    fn range_start(&self) -> NaiveDate {
        match self.mode {
            CalendarMode::Week => this_week_start(),
            CalendarMode::Month => grid_start(self.month.get()),
        }
    }

    /// Selects the day in grid cell `index`, switching month if it's outside.
    pub fn select_cell(&self, index: usize) {
        let date = self.range_start() + Duration::days(index as i64);
        self.selected.set(date);
        if self.mode == CalendarMode::Month && first_of_month(date) != self.month.get() {
            self.month.set(first_of_month(date));
        }
    }

    // --- Rendering ---

    pub fn refresh(&self, state: &AppState) {
        let records: HashMap<&str, &DailyRecord> = state
            .history
            .iter()
            .chain(std::iter::once(&state.today))
            .map(|d| (d.date.as_str(), d))
            .collect();
        let record = |date: NaiveDate| records.get(date.format("%Y-%m-%d").to_string().as_str()).copied();

        let month = self.month.get();
        let today = Local::now().date_naive();
        let selected = self.selected.get();
        let start = self.range_start();
        let title = match self.mode {
            CalendarMode::Week => format!(
                "This Week · {} – {}",
                start.format("%b %-d"),
                (start + Duration::days(6)).format("%b %-d")
            ),
            // The month picker already shows the month name.
            CalendarMode::Month => "Calendar".to_string(),
        };
        self.month_label.setStringValue(&NSString::from_str(&title));
        let back = months_between(month, first_of_month(today));
        self.month_picker.selectItemAtIndex(if (0..PICKER_MONTHS).contains(&back) { back as isize } else { -1 });

        for (idx, cell) in self.cells.iter().enumerate() {
            let date = start + Duration::days(idx as i64);
            let in_month = self.mode == CalendarMode::Week || date.month() == month.month();
            let day = record(date).filter(|_| in_month);

            cell.number.setStringValue(&NSString::from_str(&date.day().to_string()));
            let number_color = if !in_month || date > today {
                NSColor::tertiaryLabelColor()
            } else {
                NSColor::labelColor()
            };
            cell.number.setTextColor(Some(&number_color));
            cell.water.setHidden(!day.is_some_and(water_goal_met));
            cell.exercise.setHidden(!day.is_some_and(exercised));
            let active = day.map_or(0, |d| d.active_seconds);
            cell.active.setStringValue(&NSString::from_str(&if active >= 60 {
                format_active_short(active)
            } else {
                String::new()
            }));

            let fill = if date == selected {
                NSColor::controlAccentColor().colorWithAlphaComponent(0.22)
            } else if in_month {
                NSColor::labelColor().colorWithAlphaComponent(0.04)
            } else {
                NSColor::clearColor()
            };
            cell.view.setFillColor(&fill);
            cell.view.setBorderWidth(if date == today { 1.5 } else { 0.0 });
            cell.view.setBorderColor(&NSColor::controlAccentColor());
        }
        // Week mode shows one row; month mode hides the 6th week when the
        // month fits in 5.
        let last = start + Duration::days((WEEKS * 7 - 7) as i64);
        for (week, row) in self.week_rows.iter().enumerate() {
            let hidden = match self.mode {
                CalendarMode::Week => week > 0,
                CalendarMode::Month => week == WEEKS - 1 && last.month() != month.month(),
            };
            row.setHidden(hidden);
        }

        self.refresh_details(selected, record(selected), today);
    }

    fn refresh_details(&self, date: NaiveDate, day: Option<&DailyRecord>, today: NaiveDate) {
        let title = if date == today {
            format!("Today · {}", date.format("%A, %B %-d"))
        } else {
            date.format("%A, %B %-d, %Y").to_string()
        };
        self.detail_title.setStringValue(&NSString::from_str(&title));
        self.detail_empty.setHidden(day.is_some());
        self.detail_rows.setHidden(day.is_none());
        let Some(day) = day else {
            return;
        };

        let check = |row: &DetailRow, done: bool, value: String| {
            let (symbol, tint) = if done {
                ("checkmark.circle.fill", NSColor::systemGreenColor())
            } else {
                ("circle", NSColor::tertiaryLabelColor())
            };
            Self::set_icon(row, symbol, &tint);
            row.value.setStringValue(&NSString::from_str(&value));
        };
        let info = |row: &DetailRow, symbol: &str, tint: &NSColor, value: String| {
            Self::set_icon(row, symbol, tint);
            row.value.setStringValue(&NSString::from_str(&value));
        };

        check(
            &self.water_row,
            water_goal_met(day),
            format!("{:.2} / {:.1} L", day.water_ml as f64 / 1000.0, day.water_goal_ml as f64 / 1000.0),
        );
        check(&self.exercise_row, exercised(day), exercise_summary(day));
        check(
            &self.habits_row,
            day.completion_ratio() >= 0.5,
            format!("{} of {} routines", day.completed_count(), day.tasks.len()),
        );
        let cups = day.coffee_cups();
        let mg = day.caffeine_mg();
        let coffee_tint = if mg > CAFFEINE_LIMIT_MG { NSColor::systemRedColor() } else { NSColor::systemBrownColor() };
        info(
            &self.coffee_row,
            "cup.and.saucer.fill",
            &coffee_tint,
            format!("{cups} {} · {mg} mg", if cups == 1 { "cup" } else { "cups" }),
        );
        info(
            &self.breaks_row,
            "pause.circle.fill",
            &NSColor::systemTealColor(),
            format!("{} {}", day.breaks_taken, if day.breaks_taken == 1 { "break" } else { "breaks" }),
        );
        info(&self.active_row, "desktopcomputer", &NSColor::systemIndigoColor(), format_active(day.active_seconds));
        let span = match (&day.first_active, &day.last_active) {
            (Some(first), Some(last)) => format!("{first} – {last}"),
            _ => "Not tracked".to_string(),
        };
        info(&self.span_row, "clock.fill", &NSColor::systemIndigoColor(), span);
    }

    fn set_icon(row: &DetailRow, symbol: &str, tint: &NSColor) {
        if let Some(img) = UIComponents::symbol(symbol, 13.0, WEIGHT_SEMIBOLD) {
            row.icon.setImage(Some(&img));
        }
        row.icon.setContentTintColor(Some(tint));
    }
}
