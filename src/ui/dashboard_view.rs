use chrono::Local;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::sel;
use objc2_app_kit::{
    NSBox, NSButton, NSColor, NSControlStateValueOff, NSControlStateValueOn,
    NSLayoutConstraintOrientation, NSLineBreakMode, NSStackView, NSStackViewGravity, NSTextField,
};
use objc2_foundation::{MainThreadMarker, NSString};

use crate::action_handler::HealthTasksActionHandler;
use crate::app_state::AppState;
use crate::models::{
    ActivityStatus, CoffeeKind, DailyRecord, TaskCategory, CAFFEINE_CUTOFF_HOUR, CAFFEINE_LIMIT_MG,
};
use crate::ui::calendar_card::{CalendarCard, CalendarMode};
use crate::ui::reminders_card::RemindersCard;
use crate::ui::components::{
    create_page_header, ProgressBar, StatTile, UIComponents, WEIGHT_SEMIBOLD,
};

/// SF Symbol and tint for each routine category.
pub fn category_style(category: &TaskCategory) -> (&'static str, Retained<NSColor>) {
    match category {
        TaskCategory::Hydration => ("drop.fill", NSColor::systemBlueColor()),
        TaskCategory::Movement => ("figure.walk", NSColor::systemOrangeColor()),
        TaskCategory::Vision => ("eye.fill", NSColor::systemTealColor()),
        TaskCategory::Posture => ("figure.stand", NSColor::systemPurpleColor()),
        TaskCategory::Mindfulness => ("brain.head.profile", NSColor::systemPinkColor()),
        TaskCategory::Nutrition => ("leaf.fill", NSColor::systemGreenColor()),
        TaskCategory::Custom => ("sparkles", NSColor::systemYellowColor()),
    }
}

fn format_mmss(secs: u32) -> String {
    format!("{:02}:{:02}", secs / 60, secs % 60)
}

struct SessionDisplay {
    pill: &'static str,
    tint: Retained<NSColor>,
    value: String,
    detail: String,
    ratio: f64,
}

fn session_display(state: &AppState) -> SessionDisplay {
    let limit = state.settings.work_interval_minutes * 60;
    let worked = state.activity.active_work_seconds;
    let ratio = if limit > 0 { worked as f64 / limit as f64 } else { 0.0 };
    match state.activity.status {
        ActivityStatus::WaitingForInput => SessionDisplay {
            pill: "Ready",
            tint: NSColor::systemGrayColor(),
            value: "00:00".into(),
            detail: "Timer starts when you use the keyboard or mouse".into(),
            ratio: 0.0,
        },
        ActivityStatus::ActiveWorking => SessionDisplay {
            pill: "Working",
            tint: NSColor::systemBlueColor(),
            value: format_mmss(worked),
            detail: format!(
                "of {} · break in {}",
                format_mmss(limit),
                format_mmss(limit.saturating_sub(worked))
            ),
            ratio,
        },
        ActivityStatus::NoticeDue => SessionDisplay {
            pill: "Break due",
            tint: NSColor::systemOrangeColor(),
            value: format_mmss(worked),
            detail: if state.notice_held {
                "Screen recording in progress · notice will show when it ends".into()
            } else {
                "You've reached your work limit. Time to step away.".into()
            },
            ratio: 1.0,
        },
        ActivityStatus::AwayOnBreak if state.activity.break_grace_secs > 0 => SessionDisplay {
            pill: "On break",
            tint: NSColor::systemGreenColor(),
            value: format_mmss(state.activity.break_grace_secs),
            detail: "Break started · after this, any key or mouse move starts a new session".into(),
            ratio: 0.0,
        },
        ActivityStatus::AwayOnBreak => SessionDisplay {
            pill: "On break",
            tint: NSColor::systemGreenColor(),
            value: format_mmss(state.activity.last_idle_seconds as u32),
            detail: "Press any key or move the mouse to start a new session".into(),
            ratio: 0.0,
        },
    }
}

/// Bar color for caffeine intake: brown, amber near the limit, red over it.
pub fn caffeine_tint(mg: u32) -> Retained<NSColor> {
    if mg > CAFFEINE_LIMIT_MG {
        NSColor::systemRedColor()
    } else if mg * 4 >= CAFFEINE_LIMIT_MG * 3 {
        NSColor::systemOrangeColor()
    } else {
        NSColor::systemBrownColor()
    }
}

fn coffee_tip(day: &DailyRecord) -> String {
    let mg = day.caffeine_mg();
    let late = day
        .last_coffee_time()
        .and_then(|t| t.split(':').next()?.parse::<u32>().ok())
        .is_some_and(|hour| hour >= CAFFEINE_CUTOFF_HOUR);
    if day.coffee_log.is_empty() {
        "No coffee yet today.".into()
    } else if mg > CAFFEINE_LIMIT_MG {
        format!("Over the {CAFFEINE_LIMIT_MG} mg daily guideline. Switch to water for the rest of the day.")
    } else if late {
        format!("Caffeine after {CAFFEINE_CUTOFF_HOUR}:00 can make it harder to sleep tonight.")
    } else {
        format!("{} mg left before the daily guideline.", CAFFEINE_LIMIT_MG - mg)
    }
}

struct TaskRow {
    checkbox: Retained<NSButton>,
    title: Retained<NSTextField>,
    count: Retained<NSTextField>,
}

pub struct DashboardView {
    pub container: Retained<NSStackView>,
    pub reminders: RemindersCard,
    pub week: CalendarCard,
    session_pill: Retained<NSBox>,
    session_pill_label: Retained<NSTextField>,
    session_value: Retained<NSTextField>,
    session_detail: Retained<NSTextField>,
    session_bar: ProgressBar,
    habits_tile: StatTile,
    water_tile: StatTile,
    coffee_tile: StatTile,
    streak_tile: StatTile,
    caffeine_value: Retained<NSTextField>,
    caffeine_detail: Retained<NSTextField>,
    caffeine_bar: ProgressBar,
    coffee_tip: Retained<NSTextField>,
    task_rows: Vec<TaskRow>,
}

impl DashboardView {
    pub fn build(
        handler: &HealthTasksActionHandler,
        state: &AppState,
        mtm: MainThreadMarker,
    ) -> Self {
        let container = UIComponents::create_v_stack(20.0, mtm);
        let target: &AnyObject = handler;

        let date_str = Local::now().format("%A, %B %-d").to_string();
        let header = create_page_header("Today", &date_str, mtm);
        UIComponents::add_full_width(&container, &header);

        // --- Live work session ---
        let (session_card, session_stack) = UIComponents::create_card("", mtm);
        let top_row = UIComponents::create_h_stack(10.0, mtm);
        top_row.addView_inGravity(
            &UIComponents::create_caption("Current work session", mtm),
            NSStackViewGravity::Leading,
        );
        let (session_pill, session_pill_label) =
            UIComponents::create_pill_badge("Ready", &NSColor::systemGrayColor(), mtm);
        top_row.addView_inGravity(&session_pill, NSStackViewGravity::Trailing);
        UIComponents::add_full_width(&session_stack, &top_row);

        let value_row = UIComponents::create_h_stack(12.0, mtm);
        value_row.setAlignment(objc2_app_kit::NSLayoutAttribute::LastBaseline);
        let session_value = UIComponents::create_value_label("00:00", 40.0, mtm);
        let session_detail = UIComponents::create_secondary_label("", 13.0, mtm);
        value_row.addView_inGravity(&session_value, NSStackViewGravity::Leading);
        value_row.addView_inGravity(&session_detail, NSStackViewGravity::Leading);
        let break_btn = UIComponents::create_symbol_button(
            "Take a Break",
            "figure.walk",
            target,
            sel!(onManualTakeBreak:),
            mtm,
        );
        value_row.addView_inGravity(&break_btn, NSStackViewGravity::Trailing);
        UIComponents::add_full_width(&session_stack, &value_row);

        let session_bar = ProgressBar::new(0.0, &NSColor::systemBlueColor(), 8.0, mtm);
        UIComponents::add_full_width(&session_stack, &session_bar.track);
        UIComponents::add_full_width(&container, &session_card);

        // --- Stat tiles ---
        let habits_tile =
            StatTile::new("Habits", "checkmark.circle.fill", &NSColor::systemGreenColor(), mtm);
        let water_tile = StatTile::new("Water", "drop.fill", &NSColor::systemBlueColor(), mtm);
        let coffee_tile =
            StatTile::new("Coffee", "cup.and.saucer.fill", &NSColor::systemBrownColor(), mtm);
        let streak_tile = StatTile::new("Streak", "flame.fill", &NSColor::systemOrangeColor(), mtm);
        let tiles = UIComponents::create_equal_row(
            &[&habits_tile.view, &water_tile.view, &coffee_tile.view, &streak_tile.view],
            14.0,
            mtm,
        );
        UIComponents::add_full_width(&container, &tiles);

        // --- Reminders (Apple Reminders app) ---
        let reminders = RemindersCard::build(handler, mtm);
        UIComponents::add_full_width(&container, &reminders.view);

        // --- This week's daily activity (full months live on Statistics) ---
        let week = CalendarCard::build(handler, CalendarMode::Week, mtm);
        UIComponents::add_full_width(&container, &week.view);

        // --- Hydration quick actions ---
        let (water_card, water_stack) = UIComponents::create_card_with_symbol(
            "Hydration",
            Some(("drop.fill", &NSColor::systemBlueColor())),
            mtm,
        );
        water_stack.addView_inGravity(
            &UIComponents::create_secondary_label(
                "Log a drink as you go. Small, steady sips beat catching up later.",
                12.0,
                mtm,
            ),
            NSStackViewGravity::Top,
        );
        let button_row = UIComponents::create_h_stack(10.0, mtm);
        let plus_250_btn = UIComponents::create_symbol_button(
            "Glass · 250 ml",
            "plus",
            target,
            sel!(onAddWater250:),
            mtm,
        );
        plus_250_btn.setBezelColor(Some(&NSColor::systemBlueColor()));
        let plus_500_btn = UIComponents::create_symbol_button(
            "Bottle · 500 ml",
            "plus",
            target,
            sel!(onAddWater500:),
            mtm,
        );
        let minus_btn = UIComponents::create_symbol_button(
            "Undo 250 ml",
            "arrow.uturn.backward",
            target,
            sel!(onMinusWater250:),
            mtm,
        );
        button_row.addView_inGravity(&plus_250_btn, NSStackViewGravity::Leading);
        button_row.addView_inGravity(&plus_500_btn, NSStackViewGravity::Leading);
        button_row.addView_inGravity(&minus_btn, NSStackViewGravity::Trailing);
        UIComponents::add_full_width(&water_stack, &button_row);
        UIComponents::add_full_width(&container, &water_card);

        // --- Coffee & caffeine ---
        let (coffee_card, coffee_stack) = UIComponents::create_card_with_symbol(
            "Coffee",
            Some(("cup.and.saucer.fill", &NSColor::systemBrownColor())),
            mtm,
        );
        let caffeine_row = UIComponents::create_h_stack(10.0, mtm);
        caffeine_row.setAlignment(objc2_app_kit::NSLayoutAttribute::LastBaseline);
        let caffeine_value = UIComponents::create_value_label("0 mg", 26.0, mtm);
        let caffeine_detail = UIComponents::create_secondary_label("", 13.0, mtm);
        caffeine_row.addView_inGravity(&caffeine_value, NSStackViewGravity::Leading);
        caffeine_row.addView_inGravity(&caffeine_detail, NSStackViewGravity::Leading);
        UIComponents::add_full_width(&coffee_stack, &caffeine_row);

        let caffeine_bar = ProgressBar::new(0.0, &NSColor::systemBrownColor(), 8.0, mtm);
        UIComponents::add_full_width(&coffee_stack, &caffeine_bar.track);
        let coffee_tip = UIComponents::create_secondary_label("", 12.0, mtm);
        coffee_stack.addView_inGravity(&coffee_tip, NSStackViewGravity::Top);

        let coffee_buttons = UIComponents::create_h_stack(10.0, mtm);
        for (kind, action) in [
            (CoffeeKind::Espresso, sel!(onAddEspresso:)),
            (CoffeeKind::Coffee, sel!(onAddCoffee:)),
            (CoffeeKind::Latte, sel!(onAddLatte:)),
        ] {
            let btn = UIComponents::create_symbol_button(
                &format!("{} · {} mg", kind.title(), kind.caffeine_mg()),
                "plus",
                target,
                action,
                mtm,
            );
            coffee_buttons.addView_inGravity(&btn, NSStackViewGravity::Leading);
        }
        let undo_coffee = UIComponents::create_symbol_button(
            "Undo",
            "arrow.uturn.backward",
            target,
            sel!(onUndoCoffee:),
            mtm,
        );
        coffee_buttons.addView_inGravity(&undo_coffee, NSStackViewGravity::Trailing);
        UIComponents::add_full_width(&coffee_stack, &coffee_buttons);
        UIComponents::add_full_width(&container, &coffee_card);

        // --- Daily routines checklist ---
        let (tasks_card, tasks_stack) = UIComponents::create_card_with_symbol(
            "Daily Routines",
            Some(("checklist", &NSColor::systemGreenColor())),
            mtm,
        );
        tasks_stack.setSpacing(12.0);

        let mut task_rows = Vec::new();
        for (idx, task) in state.today.tasks.iter().enumerate() {
            if idx > 0 {
                UIComponents::add_full_width(&tasks_stack, &UIComponents::create_separator(mtm));
            }
            let (symbol, tint) = category_style(&task.category);
            let row = UIComponents::create_h_stack(12.0, mtm);
            row.addView_inGravity(
                &UIComponents::create_icon_badge(symbol, &tint, 32.0, mtm),
                NSStackViewGravity::Leading,
            );

            let text = UIComponents::create_v_stack(2.0, mtm);
            let title = UIComponents::create_weighted_label(&task.title, 13.0, WEIGHT_SEMIBOLD, mtm);
            let desc = UIComponents::create_secondary_label(&task.description, 11.5, mtm);
            desc.setLineBreakMode(NSLineBreakMode::ByTruncatingTail);
            desc.setContentCompressionResistancePriority_forOrientation(
                250.0,
                NSLayoutConstraintOrientation::Horizontal,
            );
            text.addView_inGravity(&title, NSStackViewGravity::Top);
            text.addView_inGravity(&desc, NSStackViewGravity::Top);
            row.addView_inGravity(&text, NSStackViewGravity::Leading);

            let count = UIComponents::create_secondary_label("", 12.0, mtm);
            count.setFont(Some(&objc2_app_kit::NSFont::monospacedDigitSystemFontOfSize_weight(
                12.0, 0.0,
            )));
            let checkbox = UIComponents::create_checkbox(
                "",
                task.completed,
                idx,
                target,
                sel!(onToggleTask:),
                mtm,
            );
            checkbox.setControlSize(objc2_app_kit::NSControlSize::Large);
            // Opens this routine's how-to guide.
            let info = unsafe {
                objc2_app_kit::NSButton::buttonWithTitle_target_action(
                    &NSString::from_str(""),
                    Some(target),
                    Some(sel!(onShowRoutineGuide:)),
                    mtm,
                )
            };
            info.setBordered(false);
            info.setTag(idx as isize);
            if let Some(img) = UIComponents::symbol("info.circle", 14.0, 0.23) {
                info.setImage(Some(&img));
                info.setImagePosition(objc2_app_kit::NSCellImagePosition::ImageOnly);
            }
            info.setContentTintColor(Some(&NSColor::secondaryLabelColor()));
            info.setToolTip(Some(&NSString::from_str("How to do it")));
            row.addView_inGravity(&count, NSStackViewGravity::Trailing);
            row.addView_inGravity(&info, NSStackViewGravity::Trailing);
            row.addView_inGravity(&checkbox, NSStackViewGravity::Trailing);

            UIComponents::add_full_width(&tasks_stack, &row);
            task_rows.push(TaskRow { checkbox, title, count });
        }
        UIComponents::add_full_width(&container, &tasks_card);

        let view = Self {
            container,
            reminders,
            week,
            session_pill,
            session_pill_label,
            session_value,
            session_detail,
            session_bar,
            habits_tile,
            water_tile,
            coffee_tile,
            streak_tile,
            caffeine_value,
            caffeine_detail,
            caffeine_bar,
            coffee_tip,
            task_rows,
        };
        view.refresh(state);
        view
    }

    pub fn refresh(&self, state: &AppState) {
        self.week.refresh(state);
        let session = session_display(state);
        self.session_pill_label.setStringValue(&NSString::from_str(session.pill));
        self.session_pill_label.setTextColor(Some(&session.tint));
        self.session_pill.setFillColor(&session.tint.colorWithAlphaComponent(0.15));
        self.session_value.setStringValue(&NSString::from_str(&session.value));
        self.session_detail.setStringValue(&NSString::from_str(&session.detail));
        self.session_bar.set_tint(&session.tint);
        self.session_bar.set_ratio(session.ratio);

        let today = &state.today;
        self.habits_tile.update(
            &format!("{}/{}", today.completed_count(), today.tasks.len()),
            &format!("{:.0}% of today's routines", today.completion_ratio() * 100.0),
            today.completion_ratio(),
        );
        let water_ratio = today.water_ml as f64 / today.water_goal_ml.max(1) as f64;
        self.water_tile.update(
            &format!("{:.2} L", today.water_ml as f64 / 1000.0),
            &format!("of {:.1} L daily goal", today.water_goal_ml as f64 / 1000.0),
            water_ratio,
        );
        let mg = today.caffeine_mg();
        let cups = today.coffee_cups();
        let cups_text = format!("{} {}", cups, if cups == 1 { "cup" } else { "cups" });
        let caffeine_ratio = mg as f64 / CAFFEINE_LIMIT_MG as f64;
        let tint = caffeine_tint(mg);
        self.coffee_tile.update(&cups_text, &format!("{mg} mg caffeine"), caffeine_ratio);
        self.coffee_tile.bar.set_tint(&tint);

        self.caffeine_value
            .setStringValue(&NSString::from_str(&format!("{mg} mg")));
        let mut detail = format!("of {CAFFEINE_LIMIT_MG} mg caffeine · {cups_text}");
        if let Some(last) = today.last_coffee_time() {
            detail.push_str(&format!(" · last at {last}"));
        }
        self.caffeine_detail.setStringValue(&NSString::from_str(&detail));
        self.caffeine_bar.set_tint(&tint);
        self.caffeine_bar.set_ratio(caffeine_ratio);
        self.coffee_tip
            .setStringValue(&NSString::from_str(&coffee_tip(today)));

        let streak = state.calculate_streak();
        self.streak_tile.update(
            &format!("{} {}", streak, if streak == 1 { "day" } else { "days" }),
            "Days at 50%+ routines",
            streak as f64 / 7.0,
        );

        for (task, row) in today.tasks.iter().zip(&self.task_rows) {
            row.checkbox.setState(if task.completed {
                NSControlStateValueOn
            } else {
                NSControlStateValueOff
            });
            let title_color = if task.completed {
                NSColor::secondaryLabelColor()
            } else {
                NSColor::labelColor()
            };
            row.title.setTextColor(Some(&title_color));
            row.count.setStringValue(&NSString::from_str(&format!(
                "{}/{} {}",
                task.current_count, task.target_count, task.unit
            )));
        }
    }
}
