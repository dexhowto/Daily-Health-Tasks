use chrono::NaiveDate;
use objc2::rc::Retained;
use objc2_app_kit::{NSColor, NSFont, NSStackView, NSStackViewGravity, NSTextField};
use objc2_foundation::{MainThreadMarker, NSString};

use crate::app_state::AppState;
use crate::models::{DailyRecord, CAFFEINE_LIMIT_MG};
use crate::ui::dashboard_view::caffeine_tint;
use crate::ui::components::{
    create_page_header, ProgressBar, StatTile, UIComponents, WEIGHT_SEMIBOLD,
};

struct DayRow {
    percent: Retained<NSTextField>,
    water: Retained<NSTextField>,
    coffee: Retained<NSTextField>,
    breaks: Retained<NSTextField>,
    bar: ProgressBar,
}

impl DayRow {
    fn build(label: &str, highlight: bool, mtm: MainThreadMarker) -> (Retained<NSStackView>, Self) {
        let row = UIComponents::create_h_stack(14.0, mtm);

        let day = if highlight {
            UIComponents::create_weighted_label(label, 13.0, WEIGHT_SEMIBOLD, mtm)
        } else {
            UIComponents::create_label(label, 13.0, false, mtm)
        };
        day.widthAnchor().constraintEqualToConstant(96.0).setActive(true);
        row.addView_inGravity(&day, NSStackViewGravity::Leading);

        let tint = if highlight {
            NSColor::systemGreenColor()
        } else {
            NSColor::systemGreenColor().colorWithAlphaComponent(0.6)
        };
        let bar = ProgressBar::new(0.0, &tint, 8.0, mtm);
        bar.track
            .widthAnchor()
            .constraintGreaterThanOrEqualToConstant(80.0)
            .setActive(true);
        bar.track.setContentHuggingPriority_forOrientation(
            1.0,
            objc2_app_kit::NSLayoutConstraintOrientation::Horizontal,
        );
        row.addView_inGravity(&bar.track, NSStackViewGravity::Leading);

        let metric = |width: f64| {
            let l = UIComponents::create_secondary_label("", 12.0, mtm);
            l.setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(12.0, 0.0)));
            l.setAlignment(objc2_app_kit::NSTextAlignment::Right);
            l.widthAnchor().constraintEqualToConstant(width).setActive(true);
            l
        };
        let percent = metric(44.0);
        let water = metric(64.0);
        let coffee = metric(56.0);
        let breaks = metric(72.0);
        row.addView_inGravity(&percent, NSStackViewGravity::Trailing);
        row.addView_inGravity(&water, NSStackViewGravity::Trailing);
        row.addView_inGravity(&coffee, NSStackViewGravity::Trailing);
        row.addView_inGravity(&breaks, NSStackViewGravity::Trailing);

        (row, Self { percent, water, coffee, breaks, bar })
    }

    fn update(&self, record: &DailyRecord) {
        let ratio = record.completion_ratio();
        self.bar.set_ratio(ratio);
        self.percent
            .setStringValue(&NSString::from_str(&format!("{:.0}%", ratio * 100.0)));
        self.water.setStringValue(&NSString::from_str(&format!(
            "{:.2} L",
            record.water_ml as f64 / 1000.0
        )));
        let cups = record.coffee_cups();
        self.coffee.setStringValue(&NSString::from_str(&format!(
            "{} {}",
            cups,
            if cups == 1 { "cup" } else { "cups" }
        )));
        let coffee_color = if record.caffeine_mg() > CAFFEINE_LIMIT_MG {
            NSColor::systemRedColor()
        } else {
            NSColor::secondaryLabelColor()
        };
        self.coffee.setTextColor(Some(&coffee_color));
        self.breaks.setStringValue(&NSString::from_str(&format!(
            "{} {}",
            record.breaks_taken,
            if record.breaks_taken == 1 { "break" } else { "breaks" }
        )));
    }
}

fn day_label(date: &str) -> String {
    NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map(|d| d.format("%a, %b %-d").to_string())
        .unwrap_or_else(|_| date.to_string())
}

pub struct HistoryView {
    pub container: Retained<NSStackView>,
    streak_tile: StatTile,
    water_tile: StatTile,
    coffee_tile: StatTile,
    breaks_tile: StatTile,
    today_row: DayRow,
}

impl HistoryView {
    pub fn build(state: &AppState, mtm: MainThreadMarker) -> Self {
        let container = UIComponents::create_v_stack(20.0, mtm);

        let header = create_page_header(
            "Insights",
            "Your habits, hydration and breaks over the last 7 days.",
            mtm,
        );
        UIComponents::add_full_width(&container, &header);

        // --- Totals ---
        let streak_tile =
            StatTile::new("Streak", "flame.fill", &NSColor::systemOrangeColor(), mtm);
        let water_tile =
            StatTile::new("Hydration", "drop.fill", &NSColor::systemBlueColor(), mtm);
        let coffee_tile =
            StatTile::new("Coffee", "cup.and.saucer.fill", &NSColor::systemBrownColor(), mtm);
        let breaks_tile =
            StatTile::new("Breaks", "figure.walk", &NSColor::systemGreenColor(), mtm);
        let tiles = UIComponents::create_equal_row(
            &[&streak_tile.view, &water_tile.view, &coffee_tile.view, &breaks_tile.view],
            14.0,
            mtm,
        );
        UIComponents::add_full_width(&container, &tiles);

        // --- Last 7 days ---
        let (days_card, days_stack) = UIComponents::create_card_with_symbol(
            "Last 7 Days",
            Some(("calendar", &NSColor::systemRedColor())),
            mtm,
        );
        days_stack.setSpacing(12.0);

        let (row, today_row) = DayRow::build("Today", true, mtm);
        UIComponents::add_full_width(&days_stack, &row);

        for record in state.history.iter().rev().take(6) {
            UIComponents::add_full_width(&days_stack, &UIComponents::create_separator(mtm));
            let (row, day) = DayRow::build(&day_label(&record.date), false, mtm);
            day.update(record);
            UIComponents::add_full_width(&days_stack, &row);
        }

        if state.history.is_empty() {
            UIComponents::add_full_width(&days_stack, &UIComponents::create_separator(mtm));
            days_stack.addView_inGravity(
                &UIComponents::create_secondary_label(
                    "Previous days will appear here as your history builds up.",
                    12.0,
                    mtm,
                ),
                NSStackViewGravity::Top,
            );
        }
        UIComponents::add_full_width(&container, &days_card);

        // --- Ergonomic guide ---
        let (tips_card, tips_stack) = UIComponents::create_card_with_symbol(
            "Ergonomic Guide",
            Some(("figure.mind.and.body", &NSColor::systemPurpleColor())),
            mtm,
        );
        tips_stack.setSpacing(14.0);
        let tips: [(&str, Retained<NSColor>, &str, &str); 4] = [
            (
                "display",
                NSColor::systemBlueColor(),
                "Monitor alignment",
                "Top of the screen at eye level, about an arm's length away.",
            ),
            (
                "eye.fill",
                NSColor::systemTealColor(),
                "20-20-20 rule",
                "Every 20 minutes, look 20 feet away for 20 seconds.",
            ),
            (
                "figure.stand",
                NSColor::systemPurpleColor(),
                "Spine & shoulders",
                "Elbows and knees near 90°, shoulders relaxed and down.",
            ),
            (
                "drop.fill",
                NSColor::systemCyanColor(),
                "Steady hydration",
                "Sip regularly. Around 2 L a day helps focus and energy.",
            ),
        ];
        for (symbol, tint, title, body) in tips.iter() {
            let row = UIComponents::create_h_stack(12.0, mtm);
            row.setAlignment(objc2_app_kit::NSLayoutAttribute::Top);
            row.addView_inGravity(
                &UIComponents::create_icon_badge(symbol, tint, 28.0, mtm),
                NSStackViewGravity::Leading,
            );
            let text = UIComponents::create_v_stack(2.0, mtm);
            text.addView_inGravity(
                &UIComponents::create_weighted_label(title, 13.0, WEIGHT_SEMIBOLD, mtm),
                NSStackViewGravity::Top,
            );
            text.addView_inGravity(
                &UIComponents::create_secondary_label(body, 12.0, mtm),
                NSStackViewGravity::Top,
            );
            row.addView_inGravity(&text, NSStackViewGravity::Leading);
            UIComponents::add_full_width(&tips_stack, &row);
        }
        UIComponents::add_full_width(&container, &tips_card);

        let view = Self {
            container,
            streak_tile,
            water_tile,
            coffee_tile,
            breaks_tile,
            today_row,
        };
        view.refresh(state, mtm);
        view
    }

    pub fn refresh(&self, state: &AppState, _mtm: MainThreadMarker) {
        let recent: Vec<&DailyRecord> = std::iter::once(&state.today)
            .chain(state.history.iter().rev().take(6))
            .collect();
        let days = recent.len().max(1);

        let streak = state.calculate_streak();
        self.streak_tile.update(
            &format!("{} {}", streak, if streak == 1 { "day" } else { "days" }),
            "In a row at 50%+",
            streak as f64 / 7.0,
        );

        let total_water: u32 = recent.iter().map(|d| d.water_ml).sum();
        let total_goal: u32 = recent.iter().map(|d| d.water_goal_ml).sum();
        self.water_tile.update(
            &format!("{:.1} L", total_water as f64 / 1000.0),
            &format!("Avg {:.2} L per day", total_water as f64 / 1000.0 / days as f64),
            total_water as f64 / total_goal.max(1) as f64,
        );

        let total_cups: u32 = recent.iter().map(|d| d.coffee_cups()).sum();
        let avg_mg = recent.iter().map(|d| d.caffeine_mg()).sum::<u32>() / days as u32;
        let over_days = recent.iter().filter(|d| d.caffeine_mg() > CAFFEINE_LIMIT_MG).count();
        let mut coffee_detail = format!("Avg {avg_mg} mg per day");
        if over_days > 0 {
            coffee_detail = format!(
                "Over {CAFFEINE_LIMIT_MG} mg on {over_days} {}",
                if over_days == 1 { "day" } else { "days" }
            );
        }
        self.coffee_tile.update(
            &format!("{} {}", total_cups, if total_cups == 1 { "cup" } else { "cups" }),
            &coffee_detail,
            avg_mg as f64 / CAFFEINE_LIMIT_MG as f64,
        );
        self.coffee_tile.bar.set_tint(&caffeine_tint(avg_mg));

        let total_breaks: u32 = recent.iter().map(|d| d.breaks_taken).sum();
        let days_with_breaks = recent.iter().filter(|d| d.breaks_taken > 0).count();
        self.breaks_tile.update(
            &total_breaks.to_string(),
            &format!(
                "Taken on {} of {} {}",
                days_with_breaks,
                days,
                if days == 1 { "day" } else { "days" }
            ),
            days_with_breaks as f64 / days as f64,
        );

        self.today_row.update(&state.today);
    }
}
