use std::cell::RefCell;

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, Sel};
use objc2_app_kit::{
    NSAppearance, NSAppearanceNameAqua, NSAppearanceNameDarkAqua, NSBezelStyle, NSBox, NSBoxType, NSButton, NSColor, NSControlSize, NSControlStateValueOff,
    NSControlStateValueOn, NSFont, NSImage, NSImageSymbolConfiguration, NSImageView,
    NSLayoutConstraint, NSStackView, NSStackViewGravity, NSSwitch, NSTextAlignment, NSTextField,
    NSTitlePosition, NSUserInterfaceLayoutOrientation, NSView,
};
use objc2_foundation::{MainThreadMarker, NSArray, NSEdgeInsets, NSPoint, NSRect, NSSize, NSString};

/// Font weights (NSFontWeight raw values).
pub const WEIGHT_REGULAR: f64 = 0.0;
pub const WEIGHT_MEDIUM: f64 = 0.23;
pub const WEIGHT_SEMIBOLD: f64 = 0.3;
pub const WEIGHT_BOLD: f64 = 0.4;

/// Shared spacing scale.
pub const CARD_PADDING: f64 = 20.0;
pub const CARD_RADIUS: f64 = 14.0;

pub struct UIComponents;

impl UIComponents {
    pub fn create_label(
        text: &str,
        size: f64,
        bold: bool,
        mtm: MainThreadMarker,
    ) -> Retained<NSTextField> {
        let weight = if bold { WEIGHT_SEMIBOLD } else { WEIGHT_REGULAR };
        Self::create_weighted_label(text, size, weight, mtm)
    }

    pub fn create_weighted_label(
        text: &str,
        size: f64,
        weight: f64,
        mtm: MainThreadMarker,
    ) -> Retained<NSTextField> {
        let label = NSTextField::labelWithString(&NSString::from_str(text), mtm);
        label.setFont(Some(&NSFont::systemFontOfSize_weight(size, weight)));
        label
    }

    pub fn create_secondary_label(
        text: &str,
        size: f64,
        mtm: MainThreadMarker,
    ) -> Retained<NSTextField> {
        let label = Self::create_label(text, size, false, mtm);
        label.setTextColor(Some(&NSColor::secondaryLabelColor()));
        label
    }

    /// Multi-line secondary label that wraps to the width it is given.
    pub fn create_wrapping_label(
        text: &str,
        size: f64,
        mtm: MainThreadMarker,
    ) -> Retained<NSTextField> {
        let label = NSTextField::wrappingLabelWithString(&NSString::from_str(text), mtm);
        label.setFont(Some(&NSFont::systemFontOfSize(size)));
        label.setTextColor(Some(&NSColor::secondaryLabelColor()));
        label.setSelectable(false);
        label
    }

    /// Small uppercase caption used above stat values and form groups.
    pub fn create_caption(text: &str, mtm: MainThreadMarker) -> Retained<NSTextField> {
        let label = Self::create_weighted_label(&text.to_uppercase(), 10.5, WEIGHT_SEMIBOLD, mtm);
        label.setTextColor(Some(&NSColor::secondaryLabelColor()));
        label
    }

    /// Large rounded monospaced-digit value label.
    pub fn create_value_label(
        text: &str,
        size: f64,
        mtm: MainThreadMarker,
    ) -> Retained<NSTextField> {
        let label = NSTextField::labelWithString(&NSString::from_str(text), mtm);
        label.setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(size, WEIGHT_BOLD)));
        label.setTextColor(Some(&NSColor::labelColor()));
        label
    }

    pub fn create_clock_label(text: &str, mtm: MainThreadMarker) -> Retained<NSTextField> {
        let label = NSTextField::labelWithString(&NSString::from_str(text), mtm);
        label.setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(72.0, WEIGHT_MEDIUM)));
        label.setAlignment(NSTextAlignment::Center);
        label.setTextColor(Some(&NSColor::labelColor()));
        label
    }

    pub fn create_button(
        title: &str,
        target: &AnyObject,
        action: Sel,
        mtm: MainThreadMarker,
    ) -> Retained<NSButton> {
        let ns_str = NSString::from_str(title);
        let btn = unsafe {
            NSButton::buttonWithTitle_target_action(&ns_str, Some(target), Some(action), mtm)
        };
        btn.setBezelStyle(NSBezelStyle::Push);
        btn.setControlSize(NSControlSize::Large);
        btn
    }


    /// Push button with a leading SF Symbol.
    pub fn create_symbol_button(
        title: &str,
        symbol: &str,
        target: &AnyObject,
        action: Sel,
        mtm: MainThreadMarker,
    ) -> Retained<NSButton> {
        let btn = Self::create_button(title, target, action, mtm);
        if let Some(img) = Self::symbol(symbol, 13.0, WEIGHT_MEDIUM) {
            btn.setImage(Some(&img));
            btn.setImagePosition(objc2_app_kit::NSCellImagePosition::ImageLeading);
        }
        btn
    }

    pub fn create_checkbox(
        title: &str,
        checked: bool,
        tag: usize,
        target: &AnyObject,
        action: Sel,
        mtm: MainThreadMarker,
    ) -> Retained<NSButton> {
        let ns_str = NSString::from_str(title);
        let btn = unsafe {
            NSButton::checkboxWithTitle_target_action(&ns_str, Some(target), Some(action), mtm)
        };
        btn.setTag(tag as isize);
        btn.setState(if checked { NSControlStateValueOn } else { NSControlStateValueOff });
        btn
    }

    pub fn create_switch(
        on: bool,
        target: &AnyObject,
        action: Sel,
        mtm: MainThreadMarker,
    ) -> Retained<NSSwitch> {
        let sw = NSSwitch::new(mtm);
        unsafe {
            sw.setTarget(Some(target));
            sw.setAction(Some(action));
        }
        sw.setControlSize(NSControlSize::Small);
        sw.setState(if on { NSControlStateValueOn } else { NSControlStateValueOff });
        sw
    }

    /// App icon (assets/icon_1024.png), embedded so `cargo run` shows it too.
    pub fn app_icon(mtm: MainThreadMarker) -> Option<Retained<NSImage>> {
        let data = objc2_foundation::NSData::with_bytes(include_bytes!("../../assets/icon_1024.png"));
        NSImage::initWithData(mtm.alloc(), &data)
    }

    /// SF Symbol image at a given point size, or None on systems without the symbol.
    pub fn symbol(name: &str, size: f64, weight: f64) -> Option<Retained<NSImage>> {
        let img = NSImage::imageWithSystemSymbolName_accessibilityDescription(
            &NSString::from_str(name),
            None,
        )?;
        let config = NSImageSymbolConfiguration::configurationWithPointSize_weight(size, weight);
        img.imageWithSymbolConfiguration(&config)
    }

    pub fn create_symbol_view(
        name: &str,
        size: f64,
        tint: &NSColor,
        mtm: MainThreadMarker,
    ) -> Retained<NSImageView> {
        let view = match Self::symbol(name, size, WEIGHT_SEMIBOLD) {
            Some(img) => NSImageView::imageViewWithImage(&img, mtm),
            None => NSImageView::new(mtm),
        };
        view.setContentTintColor(Some(tint));
        view
    }

    /// Rounded square tile with a tinted symbol, used as a row/icon badge.
    pub fn create_icon_badge(
        symbol: &str,
        tint: &NSColor,
        side: f64,
        mtm: MainThreadMarker,
    ) -> Retained<NSBox> {
        let badge = Self::create_rounded_box(side * 0.28, mtm);
        badge.setFillColor(&tint.colorWithAlphaComponent(0.16));
        badge.setBorderWidth(0.0);
        Self::pin_size(&badge, side, side);

        let icon = Self::create_symbol_view(symbol, side * 0.45, tint, mtm);
        icon.setTranslatesAutoresizingMaskIntoConstraints(false);
        badge.addSubview(&icon);
        Self::activate(&[
            icon.centerXAnchor().constraintEqualToAnchor(&badge.centerXAnchor()),
            icon.centerYAnchor().constraintEqualToAnchor(&badge.centerYAnchor()),
        ]);
        badge
    }

    pub fn create_rounded_box(radius: f64, mtm: MainThreadMarker) -> Retained<NSBox> {
        let frame = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(100.0, 100.0));
        let box_view = NSBox::initWithFrame(mtm.alloc(), frame);
        box_view.setBoxType(NSBoxType::Custom);
        box_view.setTitlePosition(NSTitlePosition::NoTitle);
        box_view.setCornerRadius(radius);
        box_view.setContentViewMargins(NSSize::new(0.0, 0.0));
        box_view.setTranslatesAutoresizingMaskIntoConstraints(false);
        box_view
    }

    /// Card surface: rounded, hairline border, content stack with generous padding.
    /// An empty title produces a card without a header.
    pub fn create_card(
        title: &str,
        mtm: MainThreadMarker,
    ) -> (Retained<NSBox>, Retained<NSStackView>) {
        Self::create_card_with_symbol(title, None, mtm)
    }

    pub fn create_card_with_symbol(
        title: &str,
        symbol: Option<(&str, &NSColor)>,
        mtm: MainThreadMarker,
    ) -> (Retained<NSBox>, Retained<NSStackView>) {
        let box_view = Self::create_rounded_box(CARD_RADIUS, mtm);
        Self::style_surface(&box_view);

        let stack = Self::create_v_stack(14.0, mtm);
        stack.setEdgeInsets(NSEdgeInsets {
            top: CARD_PADDING,
            left: CARD_PADDING,
            bottom: CARD_PADDING,
            right: CARD_PADDING,
        });

        if !title.is_empty() {
            let header = Self::create_h_stack(10.0, mtm);
            if let Some((name, tint)) = symbol {
                let icon = Self::create_icon_badge(name, tint, 28.0, mtm);
                header.addView_inGravity(&icon, NSStackViewGravity::Leading);
            }
            let label = Self::create_weighted_label(title, 15.0, WEIGHT_SEMIBOLD, mtm);
            header.addView_inGravity(&label, NSStackViewGravity::Leading);
            Self::add_full_width(&stack, &header);
        }

        Self::set_box_content(&box_view, &stack);
        (box_view, stack)
    }

    /// Card surface: a shade off the page, with a faint edge.
    pub fn style_surface(box_view: &NSBox) {
        box_view.setFillColor(&Self::label_wash(0.04));
        box_view.setBorderColor(&Self::label_wash(0.08));
        box_view.setBorderWidth(1.0);
    }

    /// Text color at `alpha` (black in light mode, white in dark mode) that
    /// follows appearance changes while the app runs. `labelColor()` with an
    /// alpha keeps the appearance it was made in.
    pub fn label_wash(alpha: f64) -> Retained<NSColor> {
        let light = NSColor::colorWithWhite_alpha(0.0, alpha);
        let dark = NSColor::colorWithWhite_alpha(1.0, alpha);
        let provider = block2::RcBlock::new(move |appearance: std::ptr::NonNull<NSAppearance>| {
            let names = NSArray::from_slice(&[unsafe { NSAppearanceNameAqua }, unsafe { NSAppearanceNameDarkAqua }]);
            let best = unsafe { appearance.as_ref() }.bestMatchFromAppearancesWithNames(&names);
            let is_dark = best.is_some_and(|name| &*name == unsafe { NSAppearanceNameDarkAqua });
            // The block owns both colors, so the pointers stay valid.
            std::ptr::NonNull::from(if is_dark { &*dark } else { &*light })
        });
        unsafe { NSColor::colorWithName_dynamicProvider(None, &provider) }
    }

    pub fn create_v_stack(spacing: f64, mtm: MainThreadMarker) -> Retained<NSStackView> {
        let stack = NSStackView::new(mtm);
        stack.setOrientation(NSUserInterfaceLayoutOrientation::Vertical);
        stack.setAlignment(objc2_app_kit::NSLayoutAttribute::Leading);
        stack.setSpacing(spacing);
        stack.setTranslatesAutoresizingMaskIntoConstraints(false);
        stack
    }

    pub fn create_h_stack(spacing: f64, mtm: MainThreadMarker) -> Retained<NSStackView> {
        let stack = NSStackView::new(mtm);
        stack.setOrientation(NSUserInterfaceLayoutOrientation::Horizontal);
        stack.setAlignment(objc2_app_kit::NSLayoutAttribute::CenterY);
        stack.setSpacing(spacing);
        stack.setTranslatesAutoresizingMaskIntoConstraints(false);
        stack
    }


    pub fn create_separator(mtm: MainThreadMarker) -> Retained<NSBox> {
        let frame = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(100.0, 1.0));
        let sep = NSBox::initWithFrame(mtm.alloc(), frame);
        sep.setBoxType(NSBoxType::Separator);
        sep.setTranslatesAutoresizingMaskIntoConstraints(false);
        sep
    }

    pub fn create_text_input(initial_value: &str, mtm: MainThreadMarker) -> Retained<NSTextField> {
        let field = NSTextField::textFieldWithString(&NSString::from_str(initial_value), mtm);
        field.setFont(Some(&NSFont::systemFontOfSize(13.0)));
        field.setControlSize(NSControlSize::Large);
        field.setTranslatesAutoresizingMaskIntoConstraints(false);
        field
    }

    pub fn create_pill_badge(
        text: &str,
        tint: &NSColor,
        mtm: MainThreadMarker,
    ) -> (Retained<NSBox>, Retained<NSTextField>) {
        let box_view = Self::create_rounded_box(11.0, mtm);
        box_view.setBorderWidth(0.0);
        box_view.setFillColor(&tint.colorWithAlphaComponent(0.15));

        let label = Self::create_weighted_label(text, 11.5, WEIGHT_SEMIBOLD, mtm);
        label.setTextColor(Some(tint));

        let stack = Self::create_h_stack(0.0, mtm);
        stack.setEdgeInsets(NSEdgeInsets { top: 4.0, left: 10.0, bottom: 4.0, right: 10.0 });
        stack.addView_inGravity(&label, NSStackViewGravity::Center);

        Self::set_box_content(&box_view, &stack);
        // Hug the label so the pill never stretches inside a full-width row.
        stack.setHuggingPriority_forOrientation(
            750.0,
            objc2_app_kit::NSLayoutConstraintOrientation::Horizontal,
        );
        (box_view, label)
    }

    /// Adds `child` to a vertical stack and stretches it to the stack's inner width.
    pub fn add_full_width(stack: &NSStackView, child: &NSView) {
        stack.addView_inGravity(child, NSStackViewGravity::Top);
        let insets = stack.edgeInsets();
        Self::activate(&[child
            .widthAnchor()
            .constraintEqualToAnchor_constant(&stack.widthAnchor(), -(insets.left + insets.right))]);
    }

    /// Lays out `views` side by side with equal widths, filling the row.
    pub fn create_equal_row(
        views: &[&NSView],
        spacing: f64,
        mtm: MainThreadMarker,
    ) -> Retained<NSStackView> {
        let row = Self::create_h_stack(spacing, mtm);
        row.setDistribution(objc2_app_kit::NSStackViewDistribution::FillEqually);
        row.setAlignment(objc2_app_kit::NSLayoutAttribute::Top);
        for v in views {
            row.addView_inGravity(v, NSStackViewGravity::Leading);
        }
        row
    }

    /// Installs `content` in a custom box and pins it to all four edges, so the
    /// box sizes itself from its content under Auto Layout.
    pub fn set_box_content(box_view: &NSBox, content: &NSView) {
        box_view.setContentView(Some(content));
        content.setTranslatesAutoresizingMaskIntoConstraints(false);
        Self::activate(&[
            content.leadingAnchor().constraintEqualToAnchor(&box_view.leadingAnchor()),
            content.trailingAnchor().constraintEqualToAnchor(&box_view.trailingAnchor()),
            content.topAnchor().constraintEqualToAnchor(&box_view.topAnchor()),
            content.bottomAnchor().constraintEqualToAnchor(&box_view.bottomAnchor()),
        ]);
    }

    pub fn pin_size(view: &NSView, width: f64, height: f64) {
        Self::activate(&[
            view.widthAnchor().constraintEqualToConstant(width),
            view.heightAnchor().constraintEqualToConstant(height),
        ]);
    }

    pub fn activate(constraints: &[Retained<NSLayoutConstraint>]) {
        for c in constraints {
            c.setActive(true);
        }
    }
}

/// Rounded, tinted progress bar built from two boxes, so it can take any color
/// (NSProgressIndicator only follows the system accent).
pub struct ProgressBar {
    pub track: Retained<NSBox>,
    fill: Retained<NSBox>,
    width_constraint: RefCell<Retained<NSLayoutConstraint>>,
}

impl ProgressBar {
    pub fn new(ratio: f64, tint: &NSColor, height: f64, mtm: MainThreadMarker) -> Self {
        let track = UIComponents::create_rounded_box(height / 2.0, mtm);
        track.setBorderWidth(0.0);
        track.setFillColor(&NSColor::quaternaryLabelColor());
        track.heightAnchor().constraintEqualToConstant(height).setActive(true);

        let fill = UIComponents::create_rounded_box(height / 2.0, mtm);
        fill.setBorderWidth(0.0);
        fill.setFillColor(tint);
        track.addSubview(&fill);
        UIComponents::activate(&[
            fill.leadingAnchor().constraintEqualToAnchor(&track.leadingAnchor()),
            fill.topAnchor().constraintEqualToAnchor(&track.topAnchor()),
            fill.bottomAnchor().constraintEqualToAnchor(&track.bottomAnchor()),
        ]);

        let width_constraint = Self::width_constraint(&fill, &track, ratio);
        width_constraint.setActive(true);

        Self { track, fill, width_constraint: RefCell::new(width_constraint) }
    }

    fn width_constraint(fill: &NSBox, track: &NSBox, ratio: f64) -> Retained<NSLayoutConstraint> {
        // A zero-width rounded box still draws its corner caps, so hide it instead.
        fill.setHidden(ratio <= 0.0);
        fill.widthAnchor()
            .constraintEqualToAnchor_multiplier(&track.widthAnchor(), ratio.clamp(0.0, 1.0))
    }

    pub fn set_ratio(&self, ratio: f64) {
        let mut current = self.width_constraint.borrow_mut();
        current.setActive(false);
        let next = Self::width_constraint(&self.fill, &self.track, ratio);
        next.setActive(true);
        *current = next;
    }

    pub fn set_tint(&self, tint: &NSColor) {
        self.fill.setFillColor(tint);
    }
}

/// Stat tile: caption, big value, secondary detail and a progress bar.
pub struct StatTile {
    pub view: Retained<NSBox>,
    pub value: Retained<NSTextField>,
    pub detail: Retained<NSTextField>,
    pub bar: ProgressBar,
}

impl StatTile {
    pub fn new(
        caption: &str,
        symbol: &str,
        tint: &NSColor,
        mtm: MainThreadMarker,
    ) -> Self {
        let (card, stack) = UIComponents::create_card("", mtm);
        stack.setSpacing(6.0);
        stack.setEdgeInsets(NSEdgeInsets { top: 16.0, left: 16.0, bottom: 16.0, right: 16.0 });

        let header = UIComponents::create_h_stack(8.0, mtm);
        let icon = UIComponents::create_icon_badge(symbol, tint, 22.0, mtm);
        header.addView_inGravity(&icon, NSStackViewGravity::Leading);
        header.addView_inGravity(&UIComponents::create_caption(caption, mtm), NSStackViewGravity::Leading);

        let value = UIComponents::create_value_label("", 28.0, mtm);
        let detail = UIComponents::create_secondary_label("", 11.5, mtm);
        // Let long details truncate so equal-width tiles stay equal.
        detail.setLineBreakMode(objc2_app_kit::NSLineBreakMode::ByTruncatingTail);
        detail.setContentCompressionResistancePriority_forOrientation(
            250.0,
            objc2_app_kit::NSLayoutConstraintOrientation::Horizontal,
        );
        let bar = ProgressBar::new(0.0, tint, 6.0, mtm);

        stack.addView_inGravity(&header, NSStackViewGravity::Top);
        stack.setCustomSpacing_afterView(10.0, &header);
        stack.addView_inGravity(&value, NSStackViewGravity::Top);
        stack.addView_inGravity(&detail, NSStackViewGravity::Top);
        stack.setCustomSpacing_afterView(10.0, &detail);
        UIComponents::add_full_width(&stack, &bar.track);

        Self { view: card, value, detail, bar }
    }

    pub fn update(&self, value: &str, detail: &str, ratio: f64) {
        self.value.setStringValue(&NSString::from_str(value));
        self.detail.setStringValue(&NSString::from_str(detail));
        self.bar.set_ratio(ratio);
    }
}

/// Page header: large title + secondary subtitle.
pub fn create_page_header(
    title: &str,
    subtitle: &str,
    mtm: MainThreadMarker,
) -> Retained<NSStackView> {
    let stack = UIComponents::create_v_stack(4.0, mtm);
    stack.addView_inGravity(
        &UIComponents::create_weighted_label(title, 28.0, WEIGHT_BOLD, mtm),
        NSStackViewGravity::Top,
    );
    stack.addView_inGravity(
        &UIComponents::create_secondary_label(subtitle, 13.0, mtm),
        NSStackViewGravity::Top,
    );
    stack
}
