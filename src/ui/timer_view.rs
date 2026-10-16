use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::sel;
use objc2_app_kit::{
    NSButton, NSColor, NSControlSize, NSImageView, NSSegmentSwitchTracking, NSSegmentedControl,
    NSStackView, NSStackViewGravity, NSTextAlignment, NSTextField,
};
use objc2_foundation::{MainThreadMarker, NSArray, NSString};

use crate::action_handler::HealthTasksActionHandler;
use crate::app_state::AppState;
use crate::models::BreakType;
use crate::ui::components::{
    create_page_header, ProgressBar, UIComponents, WEIGHT_MEDIUM, WEIGHT_SEMIBOLD,
};

fn break_style(break_type: &BreakType) -> (&'static str, Retained<NSColor>) {
    match break_type {
        BreakType::EyeBreak20 => ("eye.fill", NSColor::systemTealColor()),
        BreakType::StandStretch => ("figure.walk", NSColor::systemOrangeColor()),
        BreakType::DeepBreathing => ("wind", NSColor::systemIndigoColor()),
        BreakType::PostureCheck => ("figure.stand", NSColor::systemPurpleColor()),
    }
}

fn segment_for(break_type: &BreakType) -> isize {
    match break_type {
        BreakType::EyeBreak20 => 0,
        BreakType::StandStretch => 1,
        BreakType::DeepBreathing => 2,
        BreakType::PostureCheck => 3,
    }
}

pub struct TimerView {
    pub container: Retained<NSStackView>,
    pub mode_selector: Retained<NSSegmentedControl>,
    timer_card: Retained<objc2_app_kit::NSBox>,
    mode_icon: Retained<NSImageView>,
    mode_title: Retained<NSTextField>,
    pub countdown_label: Retained<NSTextField>,
    pub instruction_label: Retained<NSTextField>,
    progress_bar: ProgressBar,
    pub toggle_button: Retained<NSButton>,
    pub stats_label: Retained<NSTextField>,
}

impl TimerView {
    pub fn build(
        handler: &HealthTasksActionHandler,
        state: &AppState,
        mtm: MainThreadMarker,
    ) -> Self {
        let container = UIComponents::create_v_stack(20.0, mtm);
        let target: &AnyObject = handler;

        let header = create_page_header(
            "Break Timer",
            "Short, guided micro-breaks for your eyes, body and breath.",
            mtm,
        );
        UIComponents::add_full_width(&container, &header);

        // --- Break type selector ---
        let labels = [
            NSString::from_str("20-20-20"),
            NSString::from_str("Stand & Stretch"),
            NSString::from_str("Breathe"),
            NSString::from_str("Posture"),
        ];
        let mode_selector = unsafe {
            NSSegmentedControl::segmentedControlWithLabels_trackingMode_target_action(
                &NSArray::from_retained_slice(&labels),
                NSSegmentSwitchTracking::SelectOne,
                Some(target),
                Some(sel!(onSelectBreakMode:)),
                mtm,
            )
        };
        mode_selector.setControlSize(NSControlSize::Large);
        mode_selector.setSegmentDistribution(
            objc2_app_kit::NSSegmentDistribution::FillEqually,
        );
        UIComponents::add_full_width(&container, &mode_selector);

        // --- Timer card ---
        let (timer_card, timer_stack) = UIComponents::create_card("", mtm);
        timer_stack.setAlignment(objc2_app_kit::NSLayoutAttribute::CenterX);
        timer_stack.setSpacing(10.0);
        timer_stack.setEdgeInsets(objc2_foundation::NSEdgeInsets {
            top: 32.0,
            left: 32.0,
            bottom: 28.0,
            right: 32.0,
        });

        let (symbol, tint) = break_style(&state.timer.break_type);
        let mode_icon = UIComponents::create_symbol_view(symbol, 28.0, &tint, mtm);
        let mode_title = UIComponents::create_weighted_label(
            state.timer.break_type.title(),
            17.0,
            WEIGHT_SEMIBOLD,
            mtm,
        );
        let countdown_label =
            UIComponents::create_clock_label(&state.timer.formatted_remaining(), mtm);
        let instruction_label =
            UIComponents::create_wrapping_label(state.timer.break_type.instruction(), 13.0, mtm);
        instruction_label.setAlignment(NSTextAlignment::Center);
        instruction_label
            .widthAnchor()
            .constraintLessThanOrEqualToConstant(420.0)
            .setActive(true);

        let progress_bar = ProgressBar::new(0.0, &tint, 8.0, mtm);

        let toggle_button =
            UIComponents::create_button("Start", target, sel!(onTimerToggle:), mtm);
        toggle_button
            .widthAnchor()
            .constraintGreaterThanOrEqualToConstant(120.0)
            .setActive(true);
        let reset_button = UIComponents::create_symbol_button(
            "Reset",
            "arrow.counterclockwise",
            target,
            sel!(onTimerReset:),
            mtm,
        );
        let controls_row = UIComponents::create_h_stack(12.0, mtm);
        controls_row.addView_inGravity(&reset_button, NSStackViewGravity::Center);
        controls_row.addView_inGravity(&toggle_button, NSStackViewGravity::Center);

        timer_stack.addView_inGravity(&mode_icon, NSStackViewGravity::Top);
        timer_stack.addView_inGravity(&mode_title, NSStackViewGravity::Top);
        timer_stack.addView_inGravity(&countdown_label, NSStackViewGravity::Top);
        timer_stack.addView_inGravity(&instruction_label, NSStackViewGravity::Top);
        timer_stack.setCustomSpacing_afterView(20.0, &instruction_label);
        UIComponents::add_full_width(&timer_stack, &progress_bar.track);
        timer_stack.setCustomSpacing_afterView(20.0, &progress_bar.track);
        timer_stack.addView_inGravity(&controls_row, NSStackViewGravity::Top);
        UIComponents::add_full_width(&container, &timer_card);

        // --- Impact note ---
        let (stats_card, stats_stack) = UIComponents::create_card("", mtm);
        let row = UIComponents::create_h_stack(14.0, mtm);
        row.addView_inGravity(
            &UIComponents::create_icon_badge(
                "lightbulb.fill",
                &NSColor::systemYellowColor(),
                36.0,
                mtm,
            ),
            NSStackViewGravity::Leading,
        );
        let text = UIComponents::create_v_stack(2.0, mtm);
        let stats_label = UIComponents::create_weighted_label("", 13.0, WEIGHT_MEDIUM, mtm);
        text.addView_inGravity(&stats_label, NSStackViewGravity::Top);
        text.addView_inGravity(
            &UIComponents::create_secondary_label(
                "Regular micro-breaks ease eye strain and take load off your spine.",
                12.0,
                mtm,
            ),
            NSStackViewGravity::Top,
        );
        row.addView_inGravity(&text, NSStackViewGravity::Leading);
        UIComponents::add_full_width(&stats_stack, &row);
        UIComponents::add_full_width(&container, &stats_card);

        let view = Self {
            container,
            mode_selector,
            timer_card,
            mode_icon,
            mode_title,
            countdown_label,
            instruction_label,
            progress_bar,
            toggle_button,
            stats_label,
        };
        view.refresh(state);
        view
    }

    pub fn refresh(&self, state: &AppState) {
        let timer = &state.timer;
        let (symbol, tint) = break_style(&timer.break_type);
        if let Some(img) = UIComponents::symbol(symbol, 28.0, WEIGHT_SEMIBOLD) {
            self.mode_icon.setImage(Some(&img));
        }
        self.mode_icon.setContentTintColor(Some(&tint));
        // The timer card takes a faint wash of the break type's color.
        self.timer_card.setFillColor(&tint.colorWithAlphaComponent(0.06));
        self.timer_card.setBorderColor(&tint.colorWithAlphaComponent(0.25));
        self.mode_title
            .setStringValue(&NSString::from_str(timer.break_type.title()));
        self.countdown_label
            .setStringValue(&NSString::from_str(&timer.formatted_remaining()));
        self.instruction_label
            .setStringValue(&NSString::from_str(timer.break_type.instruction()));

        let elapsed = timer.total_secs.saturating_sub(timer.remaining_secs);
        self.progress_bar.set_tint(&tint);
        self.progress_bar
            .set_ratio(elapsed as f64 / timer.total_secs.max(1) as f64);

        let (title, symbol, color) = if timer.is_running {
            ("Pause", "pause.fill", NSColor::systemOrangeColor())
        } else if elapsed > 0 {
            ("Resume", "play.fill", NSColor::controlAccentColor())
        } else {
            ("Start", "play.fill", NSColor::controlAccentColor())
        };
        self.toggle_button.setTitle(&NSString::from_str(title));
        if let Some(img) = UIComponents::symbol(symbol, 13.0, WEIGHT_MEDIUM) {
            self.toggle_button.setImage(Some(&img));
            self.toggle_button
                .setImagePosition(objc2_app_kit::NSCellImagePosition::ImageLeading);
        }
        self.toggle_button.setBezelColor(Some(&color));

        let breaks = state.today.breaks_taken;
        self.stats_label.setStringValue(&NSString::from_str(&format!(
            "{} {} taken today",
            breaks,
            if breaks == 1 { "break" } else { "breaks" }
        )));

        self.mode_selector
            .setSelectedSegment(segment_for(&timer.break_type));
    }
}
