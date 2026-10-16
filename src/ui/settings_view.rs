use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::sel;
use objc2_app_kit::{
    NSColor, NSControlStateValueOff, NSControlStateValueOn, NSSegmentSwitchTracking,
    NSSegmentedControl, NSStackView, NSStackViewGravity, NSSwitch, NSTextField, NSView,
};
use objc2_foundation::{MainThreadMarker, NSArray, NSString};

use crate::action_handler::HealthTasksActionHandler;
use crate::app_state::AppState;
use crate::ui::components::{create_page_header, UIComponents, WEIGHT_MEDIUM};

/// Settings row: title and help text on the left, control on the right.
fn form_row(
    title: &str,
    help: &str,
    control: &NSView,
    mtm: MainThreadMarker,
) -> Retained<NSStackView> {
    let row = UIComponents::create_h_stack(16.0, mtm);
    let text = UIComponents::create_v_stack(2.0, mtm);
    text.addView_inGravity(
        &UIComponents::create_weighted_label(title, 13.0, WEIGHT_MEDIUM, mtm),
        NSStackViewGravity::Top,
    );
    if !help.is_empty() {
        text.addView_inGravity(
            &UIComponents::create_secondary_label(help, 11.5, mtm),
            NSStackViewGravity::Top,
        );
    }
    row.addView_inGravity(&text, NSStackViewGravity::Leading);
    row.addView_inGravity(control, NSStackViewGravity::Trailing);
    row
}

fn segmented(
    labels: &[&str],
    selected: isize,
    target: &AnyObject,
    action: objc2::runtime::Sel,
    mtm: MainThreadMarker,
) -> Retained<NSSegmentedControl> {
    let labels: Vec<Retained<NSString>> = labels.iter().map(|l| NSString::from_str(l)).collect();
    let control = unsafe {
        NSSegmentedControl::segmentedControlWithLabels_trackingMode_target_action(
            &NSArray::from_retained_slice(&labels),
            NSSegmentSwitchTracking::SelectOne,
            Some(target),
            Some(action),
            mtm,
        )
    };
    control.setSelectedSegment(selected);
    control
}

pub struct SettingsView {
    pub container: Retained<NSStackView>,
    pub title_field: Retained<NSTextField>,
    pub message_field: Retained<NSTextField>,
    pub banner_checkbox: Retained<NSSwitch>,
    pub alert_checkbox: Retained<NSSwitch>,
    pub sound_checkbox: Retained<NSSwitch>,
    pub status_feedback_label: Retained<NSTextField>,
}

impl SettingsView {
    pub fn build(
        handler: &HealthTasksActionHandler,
        state: &AppState,
        mtm: MainThreadMarker,
    ) -> Self {
        let container = UIComponents::create_v_stack(20.0, mtm);
        let target: &AnyObject = handler;

        let header = create_page_header(
            "Settings",
            "DexHowTo HealthTask watches keyboard and mouse activity in the background.",
            mtm,
        );
        UIComponents::add_full_width(&container, &header);

        // --- Activity monitor ---
        let (timer_card, timer_stack) = UIComponents::create_card_with_symbol(
            "Activity Monitor",
            Some(("keyboard", &NSColor::systemBlueColor())),
            mtm,
        );
        let work_sel = match state.settings.work_interval_minutes {
            30 => 0,
            45 => 1,
            90 => 3,
            _ => 2,
        };
        let work_selector = segmented(
            &["30m", "45m", "60m", "90m"],
            work_sel,
            target,
            sel!(onWorkIntervalChanged:),
            mtm,
        );
        UIComponents::add_full_width(
            &timer_stack,
            &form_row(
                "Remind me to take a break after",
                "Continuous keyboard or mouse activity",
                &work_selector,
                mtm,
            ),
        );
        UIComponents::add_full_width(&timer_stack, &UIComponents::create_separator(mtm));

        let idle_sel = match state.settings.idle_reset_minutes {
            3 => 0,
            10 => 2,
            15 => 3,
            _ => 1,
        };
        let idle_selector = segmented(
            &["3m", "5m", "10m", "15m"],
            idle_sel,
            target,
            sel!(onIdleIntervalChanged:),
            mtm,
        );
        UIComponents::add_full_width(
            &timer_stack,
            &form_row(
                "Count it as a break when away for",
                "The work session restarts after this much idle time",
                &idle_selector,
                mtm,
            ),
        );
        UIComponents::add_full_width(&container, &timer_card);

        // --- Notice content ---
        let (notice_card, notice_stack) = UIComponents::create_card_with_symbol(
            "Break Notice",
            Some(("text.bubble.fill", &NSColor::systemPurpleColor())),
            mtm,
        );
        notice_stack.setSpacing(8.0);
        let title_field = UIComponents::create_text_input(&state.settings.notice_title, mtm);
        let message_field = UIComponents::create_text_input(&state.settings.notice_message, mtm);
        let title_caption = UIComponents::create_caption("Title", mtm);
        notice_stack.addView_inGravity(&title_caption, NSStackViewGravity::Top);
        UIComponents::add_full_width(&notice_stack, &title_field);
        notice_stack.setCustomSpacing_afterView(16.0, &title_field);
        notice_stack.addView_inGravity(&UIComponents::create_caption("Message", mtm), NSStackViewGravity::Top);
        UIComponents::add_full_width(&notice_stack, &message_field);
        UIComponents::add_full_width(&container, &notice_card);

        // --- Delivery ---
        let (delivery_card, delivery_stack) = UIComponents::create_card_with_symbol(
            "Delivery",
            Some(("bell.badge.fill", &NSColor::systemRedColor())),
            mtm,
        );
        let banner_checkbox = UIComponents::create_switch(
            state.settings.show_banner_notification,
            target,
            sel!(onToggleBannerSetting:),
            mtm,
        );
        let alert_checkbox = UIComponents::create_switch(
            state.settings.show_alert_dialog,
            target,
            sel!(onToggleAlertSetting:),
            mtm,
        );
        let sound_checkbox = UIComponents::create_switch(
            state.settings.sound_enabled,
            target,
            sel!(onToggleSound:),
            mtm,
        );
        let rows: [(&str, &str, &NSView); 3] = [
            ("Notification banner", "Shows in Notification Center", &banner_checkbox),
            ("On-screen alert", "A dialog you have to dismiss", &alert_checkbox),
            ("Chime", "Plays the Glass sound", &sound_checkbox),
        ];
        for (idx, (title, help, control)) in rows.iter().enumerate() {
            if idx > 0 {
                UIComponents::add_full_width(&delivery_stack, &UIComponents::create_separator(mtm));
            }
            UIComponents::add_full_width(&delivery_stack, &form_row(title, help, control, mtm));
        }
        UIComponents::add_full_width(&container, &delivery_card);

        // --- Actions ---
        let action_row = UIComponents::create_h_stack(10.0, mtm);
        let status_feedback_label = UIComponents::create_secondary_label(
            "Saved in ~/Library/Application Support/HealthTasks",
            11.5,
            mtm,
        );
        status_feedback_label.setLineBreakMode(objc2_app_kit::NSLineBreakMode::ByTruncatingMiddle);
        status_feedback_label.setContentCompressionResistancePriority_forOrientation(
            250.0,
            objc2_app_kit::NSLayoutConstraintOrientation::Horizontal,
        );
        let test_btn = UIComponents::create_symbol_button(
            "Send Test Notice",
            "bell",
            target,
            sel!(onTestNotice:),
            mtm,
        );
        let save_btn = UIComponents::create_button("Save", target, sel!(onSaveSettings:), mtm);
        save_btn.setKeyEquivalent(&NSString::from_str("\r"));
        action_row.addView_inGravity(&status_feedback_label, NSStackViewGravity::Leading);
        action_row.addView_inGravity(&test_btn, NSStackViewGravity::Trailing);
        action_row.addView_inGravity(&save_btn, NSStackViewGravity::Trailing);
        UIComponents::add_full_width(&container, &action_row);

        Self {
            container,
            title_field,
            message_field,
            banner_checkbox,
            alert_checkbox,
            sound_checkbox,
            status_feedback_label,
        }
    }

    pub fn refresh(&self, state: &AppState) {
        let on = |v: bool| if v { NSControlStateValueOn } else { NSControlStateValueOff };
        self.banner_checkbox.setState(on(state.settings.show_banner_notification));
        self.alert_checkbox.setState(on(state.settings.show_alert_dialog));
        self.sound_checkbox.setState(on(state.settings.sound_enabled));
    }
}
