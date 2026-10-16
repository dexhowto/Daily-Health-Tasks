use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{sel, Message};
use objc2_app_kit::{
    NSBox, NSButton, NSColor, NSControlSize, NSLayoutConstraintOrientation, NSLineBreakMode,
    NSStackView, NSStackViewGravity, NSTextField, NSView,
};
use objc2_foundation::{MainThreadMarker, NSString};

use crate::action_handler::HealthTasksActionHandler;
use crate::reminders::{relative_day, Access, ReminderItem, Snapshot};
use crate::ui::components::{UIComponents, WEIGHT_MEDIUM, WEIGHT_SEMIBOLD};

/// "Reminders" card on the Today page: open reminders due today or overdue,
/// with checkboxes that complete them in the Reminders app.
pub struct RemindersCard {
    pub view: Retained<NSBox>,
    handler: Retained<HealthTasksActionHandler>,
    count_pill: Retained<NSBox>,
    count_label: Retained<NSTextField>,
    message: Retained<NSTextField>,
    connect_button: Retained<NSButton>,
    rows: Retained<NSStackView>,
    mtm: MainThreadMarker,
}

impl RemindersCard {
    pub fn build(handler: &HealthTasksActionHandler, mtm: MainThreadMarker) -> Self {
        let target: &AnyObject = handler;
        let (view, stack) = UIComponents::create_card("", mtm);
        stack.setSpacing(12.0);

        // Header: icon + title, progress pill and "Open Reminders" on the right.
        let header = UIComponents::create_h_stack(8.0, mtm);
        header.addView_inGravity(
            &UIComponents::create_symbol_view("checklist.checked", 14.0, &NSColor::systemOrangeColor(), mtm),
            NSStackViewGravity::Leading,
        );
        header.addView_inGravity(
            &UIComponents::create_weighted_label("Reminders", 15.0, WEIGHT_SEMIBOLD, mtm),
            NSStackViewGravity::Leading,
        );
        let (count_pill, count_label) =
            UIComponents::create_pill_badge("", &NSColor::systemOrangeColor(), mtm);
        header.addView_inGravity(&count_pill, NSStackViewGravity::Leading);
        let open_button = unsafe {
            NSButton::buttonWithTitle_target_action(
                &NSString::from_str("Open Reminders"),
                Some(target),
                Some(sel!(onOpenReminders:)),
                mtm,
            )
        };
        open_button.setBordered(false);
        open_button.setContentTintColor(Some(&NSColor::controlAccentColor()));
        open_button.setFont(Some(&objc2_app_kit::NSFont::systemFontOfSize_weight(12.0, WEIGHT_MEDIUM)));
        if let Some(img) = UIComponents::symbol("arrow.up.forward.app", 12.0, WEIGHT_MEDIUM) {
            open_button.setImage(Some(&img));
            open_button.setImagePosition(objc2_app_kit::NSCellImagePosition::ImageTrailing);
        }
        header.addView_inGravity(&open_button, NSStackViewGravity::Trailing);
        UIComponents::add_full_width(&stack, &header);

        let message = UIComponents::create_secondary_label("", 12.0, mtm);
        stack.addView_inGravity(&message, NSStackViewGravity::Top);
        let connect_button =
            UIComponents::create_symbol_button("Connect Reminders", "link", target, sel!(onConnectReminders:), mtm);
        stack.addView_inGravity(&connect_button, NSStackViewGravity::Top);

        let rows = UIComponents::create_v_stack(10.0, mtm);
        UIComponents::add_full_width(&stack, &rows);

        let card = Self {
            view,
            handler: handler.retain(),
            count_pill,
            count_label,
            message,
            connect_button,
            rows,
            mtm,
        };
        card.update(&Snapshot::Loading);
        card
    }

    pub fn update(&self, snapshot: &Snapshot) {
        self.clear_rows();
        let (message, button, items): (&str, Option<&str>, &[ReminderItem]) = match snapshot {
            Snapshot::Access(Access::NotRequested) => (
                "Show today's reminders here and tick them off without leaving the app.",
                Some("Connect Reminders"),
                &[],
            ),
            Snapshot::Access(Access::Denied) => (
                "Access to Reminders is turned off for this app.",
                Some("Open Privacy Settings"),
                &[],
            ),
            Snapshot::Access(Access::Granted) | Snapshot::Loading => ("Loading reminders…", None, &[]),
            Snapshot::Items(items) if items.is_empty() => ("Nothing due today.", None, &[]),
            Snapshot::Items(items) => ("", None, items.as_slice()),
        };

        self.message.setStringValue(&NSString::from_str(message));
        self.message.setHidden(message.is_empty());
        self.connect_button.setHidden(button.is_none());
        if let Some(title) = button {
            self.connect_button.setTitle(&NSString::from_str(title));
        }

        let overdue = items.iter().filter(|r| r.is_overdue()).count();
        let mut count = format!("{} to do", items.len());
        if overdue > 0 {
            count.push_str(&format!(" · {overdue} overdue"));
        }
        self.count_pill.setHidden(items.is_empty());
        self.count_label.setStringValue(&NSString::from_str(&count));

        for (idx, item) in items.iter().enumerate() {
            if idx > 0 {
                UIComponents::add_full_width(&self.rows, &UIComponents::create_separator(self.mtm));
            }
            UIComponents::add_full_width(&self.rows, &self.build_row(idx, item));
        }
    }

    fn clear_rows(&self) {
        for view in self.rows.arrangedSubviews().iter() {
            self.rows.removeArrangedSubview(&view);
            view.removeFromSuperview();
        }
    }

    fn build_row(&self, idx: usize, item: &ReminderItem) -> Retained<NSView> {
        let mtm = self.mtm;
        let target: &AnyObject = &self.handler;
        let row = UIComponents::create_h_stack(10.0, mtm);

        let checkbox = UIComponents::create_checkbox(
            "",
            false,
            idx,
            target,
            sel!(onToggleReminder:),
            mtm,
        );
        checkbox.setControlSize(NSControlSize::Large);
        row.addView_inGravity(&checkbox, NSStackViewGravity::Leading);

        let text = UIComponents::create_v_stack(2.0, mtm);
        let title = UIComponents::create_weighted_label(&item.title, 13.0, WEIGHT_MEDIUM, mtm);
        title.setLineBreakMode(NSLineBreakMode::ByTruncatingTail);
        title.setContentCompressionResistancePriority_forOrientation(
            250.0,
            NSLayoutConstraintOrientation::Horizontal,
        );
        text.addView_inGravity(&title, NSStackViewGravity::Top);
        if !item.list.is_empty() {
            text.addView_inGravity(
                &UIComponents::create_secondary_label(&item.list, 11.5, mtm),
                NSStackViewGravity::Top,
            );
        }
        row.addView_inGravity(&text, NSStackViewGravity::Leading);

        let due_text = match (&item.due_date, &item.due_time) {
            _ if item.is_overdue() => {
                let day = item.due_date.map(relative_day).unwrap_or_default();
                format!("Overdue · {day}")
            }
            (_, Some(time)) => time.clone(),
            (Some(_), None) => "Today".into(),
            (None, None) => String::new(),
        };
        let due = UIComponents::create_secondary_label(&due_text, 12.0, mtm);
        if item.is_overdue() {
            due.setTextColor(Some(&NSColor::systemRedColor()));
        }
        row.addView_inGravity(&due, NSStackViewGravity::Trailing);

        Retained::into_super(row)
    }
}
