use std::cell::Cell;

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, Sel};
use objc2::{sel, Message};
use objc2_app_kit::{
    NSBox, NSButton, NSColor, NSFont, NSLayoutConstraintOrientation, NSStackView,
    NSStackViewGravity, NSTextAlignment, NSTextField, NSView,
};
use objc2_foundation::{MainThreadMarker, NSString};

use crate::action_handler::HealthTasksActionHandler;
use crate::guides::{Guide, GuideAction, GuideColor, GuideGroup, GUIDES};
use crate::ui::components::{create_page_header, UIComponents, WEIGHT_BOLD, WEIGHT_MEDIUM, WEIGHT_SEMIBOLD};

/// Width the detail text wraps at (the page is at most 760 pt wide).
const TEXT_WIDTH: f64 = 600.0;

pub fn guide_color(color: GuideColor) -> Retained<NSColor> {
    match color {
        GuideColor::Orange => NSColor::systemOrangeColor(),
        GuideColor::Purple => NSColor::systemPurpleColor(),
        GuideColor::Teal => NSColor::systemTealColor(),
        GuideColor::Indigo => NSColor::systemIndigoColor(),
        GuideColor::Cyan => NSColor::systemCyanColor(),
        GuideColor::Blue => NSColor::systemBlueColor(),
        GuideColor::Pink => NSColor::systemPinkColor(),
        GuideColor::Yellow => NSColor::systemYellowColor(),
    }
}

fn action_selector(action: GuideAction) -> Sel {
    match action {
        GuideAction::Plank => sel!(onLogPlank:),
        GuideAction::Stretching => sel!(onLogStretching:),
        GuideAction::Walk => sel!(onLogWalk:),
        GuideAction::Water => sel!(onAddWater250:),
    }
}

struct Chip {
    view: Retained<NSBox>,
    icon: Retained<objc2_app_kit::NSImageView>,
    label: Retained<NSTextField>,
}

/// Guides page: a picker of all exercises and routines, and the selected
/// guide's step-by-step instructions.
pub struct GuidesView {
    pub container: Retained<NSStackView>,
    handler: Retained<HealthTasksActionHandler>,
    chips: Vec<Chip>,
    detail: Retained<NSStackView>,
    selected: Cell<usize>,
    mtm: MainThreadMarker,
}

impl GuidesView {
    pub fn build(handler: &HealthTasksActionHandler, mtm: MainThreadMarker) -> Self {
        let container = UIComponents::create_v_stack(20.0, mtm);
        UIComponents::add_full_width(
            &container,
            &create_page_header("Guides", "How to do each exercise and daily routine, step by step.", mtm),
        );

        // Picker.
        let (picker_card, picker) = UIComponents::create_card("", mtm);
        picker.setSpacing(8.0);
        let mut chips = Vec::new();
        for (group, caption) in [(GuideGroup::Exercise, "Exercises"), (GuideGroup::Routine, "Daily routines")] {
            if group == GuideGroup::Routine {
                picker.setCustomSpacing_afterView(16.0, &picker.arrangedSubviews().lastObject().unwrap());
            }
            picker.addView_inGravity(&UIComponents::create_caption(caption, mtm), NSStackViewGravity::Top);
            let indices: Vec<usize> = GUIDES.iter().enumerate().filter(|(_, g)| g.group == group).map(|(i, _)| i).collect();
            for row in indices.chunks(3) {
                let mut views: Vec<Retained<NSView>> = Vec::new();
                for &idx in row {
                    let chip = Self::build_chip(idx, &GUIDES[idx], handler, mtm);
                    views.push(Retained::into_super(chip.view.clone()));
                    chips.push((idx, chip));
                }
                // Keep chip widths equal when a row is short.
                while views.len() < 3 {
                    let spacer = NSView::new(mtm);
                    spacer.setTranslatesAutoresizingMaskIntoConstraints(false);
                    views.push(spacer);
                }
                let refs: Vec<&NSView> = views.iter().map(|v| &**v).collect();
                UIComponents::add_full_width(&picker, &UIComponents::create_equal_row(&refs, 8.0, mtm));
            }
        }
        chips.sort_by_key(|(idx, _)| *idx);
        let chips = chips.into_iter().map(|(_, c)| c).collect();
        UIComponents::add_full_width(&container, &picker_card);

        // Detail.
        let (detail_card, detail) = UIComponents::create_card("", mtm);
        detail.setSpacing(10.0);
        UIComponents::add_full_width(&container, &detail_card);

        let view = Self {
            container,
            handler: handler.retain(),
            chips,
            detail,
            selected: Cell::new(0),
            mtm,
        };
        view.show(0);
        view
    }

    /// Picker chip: tinted icon and plain label, left-aligned, with an
    /// invisible button over it for the click.
    fn build_chip(idx: usize, guide: &Guide, handler: &HealthTasksActionHandler, mtm: MainThreadMarker) -> Chip {
        let target: &AnyObject = handler;
        let view = UIComponents::create_rounded_box(9.0, mtm);
        view.setBorderWidth(0.0);
        view.heightAnchor().constraintEqualToConstant(36.0).setActive(true);

        let row = UIComponents::create_h_stack(9.0, mtm);
        let icon = UIComponents::create_symbol_view(guide.symbol, 13.0, &guide_color(guide.color), mtm);
        icon.setTranslatesAutoresizingMaskIntoConstraints(false);
        UIComponents::pin_size(&icon, 20.0, 20.0);
        row.addView_inGravity(&icon, NSStackViewGravity::Leading);
        let label = UIComponents::create_weighted_label(guide.title, 12.5, WEIGHT_MEDIUM, mtm);
        label.setLineBreakMode(objc2_app_kit::NSLineBreakMode::ByTruncatingTail);
        label.setContentCompressionResistancePriority_forOrientation(250.0, NSLayoutConstraintOrientation::Horizontal);
        row.addView_inGravity(&label, NSStackViewGravity::Leading);
        view.addSubview(&row);

        let button = unsafe {
            NSButton::buttonWithTitle_target_action(&NSString::from_str(""), Some(target), Some(sel!(onSelectGuide:)), mtm)
        };
        button.setBordered(false);
        button.setTransparent(true);
        button.setTag(idx as isize);
        button.setTranslatesAutoresizingMaskIntoConstraints(false);
        view.addSubview(&button);
        UIComponents::activate(&[
            row.leadingAnchor().constraintEqualToAnchor_constant(&view.leadingAnchor(), 10.0),
            row.trailingAnchor().constraintLessThanOrEqualToAnchor_constant(&view.trailingAnchor(), -10.0),
            row.centerYAnchor().constraintEqualToAnchor(&view.centerYAnchor()),
            button.leadingAnchor().constraintEqualToAnchor(&view.leadingAnchor()),
            button.trailingAnchor().constraintEqualToAnchor(&view.trailingAnchor()),
            button.topAnchor().constraintEqualToAnchor(&view.topAnchor()),
            button.bottomAnchor().constraintEqualToAnchor(&view.bottomAnchor()),
        ]);
        Chip { view, icon, label }
    }

    /// Shows the guide with this id (e.g. from a routine's info button).
    pub fn show_id(&self, id: &str) {
        if let Some(idx) = GUIDES.iter().position(|g| g.id == id) {
            self.show(idx);
        }
    }

    pub fn show(&self, idx: usize) {
        let Some(guide) = GUIDES.get(idx) else {
            return;
        };
        self.selected.set(idx);
        for (i, chip) in self.chips.iter().enumerate() {
            let tint = guide_color(GUIDES[i].color);
            if i == idx {
                chip.view.setFillColor(&tint.colorWithAlphaComponent(0.2));
                chip.view.setBorderColor(&tint.colorWithAlphaComponent(0.6));
                chip.view.setBorderWidth(1.0);
                chip.icon.setContentTintColor(Some(&tint));
                chip.label.setTextColor(Some(&NSColor::labelColor()));
                chip.label.setFont(Some(&NSFont::systemFontOfSize_weight(12.5, WEIGHT_SEMIBOLD)));
            } else {
                chip.view.setFillColor(&UIComponents::label_wash(0.05));
                chip.view.setBorderWidth(0.0);
                chip.icon.setContentTintColor(Some(&tint));
                chip.label.setTextColor(Some(&NSColor::secondaryLabelColor()));
                chip.label.setFont(Some(&NSFont::systemFontOfSize_weight(12.5, WEIGHT_MEDIUM)));
            }
        }
        self.render_detail(guide);
    }

    fn render_detail(&self, guide: &Guide) {
        let mtm = self.mtm;
        let detail = &self.detail;
        for view in detail.arrangedSubviews().iter() {
            detail.removeArrangedSubview(&view);
            view.removeFromSuperview();
        }
        let tint = guide_color(guide.color);

        // Title.
        let header = UIComponents::create_h_stack(14.0, mtm);
        header.addView_inGravity(&UIComponents::create_icon_badge(guide.symbol, &tint, 48.0, mtm), NSStackViewGravity::Leading);
        let titles = UIComponents::create_v_stack(3.0, mtm);
        titles.addView_inGravity(&UIComponents::create_weighted_label(guide.title, 20.0, WEIGHT_BOLD, mtm), NSStackViewGravity::Top);
        titles.addView_inGravity(&Self::text(guide.summary, 13.0, TEXT_WIDTH - 62.0, mtm), NSStackViewGravity::Top);
        header.addView_inGravity(&titles, NSStackViewGravity::Leading);
        UIComponents::add_full_width(detail, &header);

        Self::section(detail, "Why it helps", mtm);
        detail.addView_inGravity(&Self::text(guide.why, 13.0, TEXT_WIDTH, mtm), NSStackViewGravity::Top);

        Self::section(detail, "How to do it", mtm);
        for (n, step) in guide.steps.iter().enumerate() {
            let row = UIComponents::create_h_stack(10.0, mtm);
            row.setAlignment(objc2_app_kit::NSLayoutAttribute::Top);
            let badge = UIComponents::create_rounded_box(11.0, mtm);
            badge.setBorderWidth(0.0);
            badge.setFillColor(&tint.colorWithAlphaComponent(0.18));
            UIComponents::pin_size(&badge, 22.0, 22.0);
            let number = UIComponents::create_weighted_label(&(n + 1).to_string(), 11.5, WEIGHT_BOLD, mtm);
            number.setTextColor(Some(&tint));
            number.setAlignment(NSTextAlignment::Center);
            number.setTranslatesAutoresizingMaskIntoConstraints(false);
            badge.addSubview(&number);
            UIComponents::activate(&[
                number.centerXAnchor().constraintEqualToAnchor(&badge.centerXAnchor()),
                number.centerYAnchor().constraintEqualToAnchor(&badge.centerYAnchor()),
            ]);
            row.addView_inGravity(&badge, NSStackViewGravity::Leading);
            let text = Self::text(step, 13.0, TEXT_WIDTH - 32.0, mtm);
            text.setTextColor(Some(&NSColor::labelColor()));
            row.addView_inGravity(&text, NSStackViewGravity::Leading);
            UIComponents::add_full_width(detail, &row);
        }

        Self::section(detail, "How long", mtm);
        detail.addView_inGravity(&Self::icon_line("clock", &tint, guide.how_long, mtm), NSStackViewGravity::Top);

        if !guide.tips.is_empty() {
            Self::section(detail, "Tips", mtm);
            for tip in guide.tips {
                detail.addView_inGravity(&Self::icon_line("lightbulb.fill", &NSColor::systemYellowColor(), tip, mtm), NSStackViewGravity::Top);
            }
        }

        if let Some((action, title)) = guide.action {
            let target: &AnyObject = &self.handler;
            let button = UIComponents::create_symbol_button(title, "plus.circle.fill", target, action_selector(action), mtm);
            button.setBezelColor(Some(&tint));
            detail.setCustomSpacing_afterView(18.0, &detail.arrangedSubviews().lastObject().unwrap());
            detail.addView_inGravity(&button, NSStackViewGravity::Top);
        }

        let note = UIComponents::create_secondary_label(
            "General guidance, not medical advice. Stop if anything hurts.",
            11.0,
            mtm,
        );
        note.setTextColor(Some(&NSColor::tertiaryLabelColor()));
        detail.setCustomSpacing_afterView(16.0, &detail.arrangedSubviews().lastObject().unwrap());
        detail.addView_inGravity(&note, NSStackViewGravity::Top);
    }

    fn section(detail: &NSStackView, title: &str, mtm: MainThreadMarker) {
        if let Some(last) = detail.arrangedSubviews().lastObject() {
            detail.setCustomSpacing_afterView(18.0, &last);
        }
        detail.addView_inGravity(&UIComponents::create_caption(title, mtm), NSStackViewGravity::Top);
    }

    /// Wrapping secondary text at a fixed width.
    fn text(text: &str, size: f64, width: f64, mtm: MainThreadMarker) -> Retained<NSTextField> {
        let label = UIComponents::create_wrapping_label(text, size, mtm);
        label.setPreferredMaxLayoutWidth(width);
        label.setContentCompressionResistancePriority_forOrientation(250.0, NSLayoutConstraintOrientation::Horizontal);
        label
    }

    fn icon_line(symbol: &str, tint: &NSColor, text: &str, mtm: MainThreadMarker) -> Retained<NSStackView> {
        let row = UIComponents::create_h_stack(8.0, mtm);
        row.setAlignment(objc2_app_kit::NSLayoutAttribute::Top);
        row.addView_inGravity(&UIComponents::create_symbol_view(symbol, 12.0, tint, mtm), NSStackViewGravity::Leading);
        let label = Self::text(text, 13.0, TEXT_WIDTH - 24.0, mtm);
        label.setTextColor(Some(&NSColor::labelColor()));
        row.addView_inGravity(&label, NSStackViewGravity::Leading);
        row
    }
}
