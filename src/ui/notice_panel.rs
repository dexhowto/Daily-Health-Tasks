//! Floating notice card for the break notice and the "You're on a break"
//! question. Unlike a modal alert it shows above every app and Space
//! (including full-screen apps) without activating this app, never ends up
//! behind other windows, and doesn't pause the app's timer while it is open.

use std::cell::Cell;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, Sel};
use objc2::Message;
use objc2_app_kit::{
    NSAnimatablePropertyContainer, NSAnimationContext, NSBox, NSButton, NSCellImagePosition, NSColor,
    NSEvent, NSFont, NSLayoutConstraintOrientation, NSScreen, NSStackViewGravity,
    NSTextField, NSView,
};
use objc2_foundation::{MainThreadMarker, NSEdgeInsets, NSPoint, NSRect, NSString};

use crate::action_handler::HealthTasksActionHandler;
use crate::ui::components::{UIComponents, WEIGHT_MEDIUM, WEIGHT_SEMIBOLD};
use crate::ui::control_center::{ControlCenter, ControlCenterPanel};

const WIDTH: f64 = 380.0;
const PADDING: f64 = 22.0;
const CORNER: f64 = 18.0;

/// What a notice shows. Buttons send their action to the app's action handler.
pub struct NoticeContent<'a> {
    pub symbol: &'a str,
    pub tint: Retained<NSColor>,
    pub caption: &'a str,
    pub title: &'a str,
    pub message: &'a str,
    /// Small facts under the message, as (SF Symbol, text).
    pub facts: Vec<(&'a str, String)>,
    pub secondary: (&'a str, Sel),
    /// (title, SF Symbol, action); drawn filled in `tint`.
    pub primary: (&'a str, &'a str, Sel),
}

pub struct NoticePanel {
    panel: Retained<ControlCenterPanel>,
    handler: Retained<HealthTasksActionHandler>,
    /// Shown, possibly hidden for a screen recording.
    shown: Cell<bool>,
    suspended: Cell<bool>,
    mtm: MainThreadMarker,
}

impl NoticePanel {
    pub fn new(handler: &HealthTasksActionHandler, mtm: MainThreadMarker) -> Self {
        Self {
            panel: ControlCenter::make_panel(mtm),
            handler: handler.retain(),
            shown: Cell::new(false),
            suspended: Cell::new(false),
            mtm,
        }
    }

    pub fn is_shown(&self) -> bool {
        self.shown.get()
    }

    /// Shows `content`, replacing what is shown, on the screen with the pointer.
    pub fn show(&self, content: &NoticeContent) {
        let root = self.build(content);
        self.panel.setContentView(Some(&root));
        root.layoutSubtreeIfNeeded();
        let size = root.fittingSize();

        let screen = Self::screen_with_pointer(self.mtm);
        let v = screen.map(|s| s.visibleFrame()).unwrap_or(NSRect::new(
            NSPoint::new(0.0, 0.0),
            objc2_foundation::NSSize::new(1440.0, 900.0),
        ));
        // Centered, a little above the middle.
        let origin = NSPoint::new(
            (v.origin.x + (v.size.width - size.width) / 2.0).round(),
            (v.origin.y + v.size.height * 0.58 - size.height / 2.0).round(),
        );
        self.panel.setFrame_display(NSRect::new(origin, size), true);
        self.panel.invalidateShadow();

        let was_shown = self.shown.replace(true);
        if self.suspended.get() {
            return;
        }
        if was_shown {
            self.panel.orderFrontRegardless();
        } else {
            self.fade_in();
        }
    }

    #[cfg(debug_assertions)]
    pub fn content_view(&self) -> Option<Retained<NSView>> {
        self.panel.contentView()
    }

    pub fn hide(&self) {
        self.shown.set(false);
        self.panel.orderOut(None);
    }

    /// Hides the notice during a screen recording and brings it back after.
    pub fn set_suspended(&self, suspended: bool) {
        if self.suspended.replace(suspended) == suspended {
            return;
        }
        if suspended {
            self.panel.orderOut(None);
        } else if self.shown.get() {
            self.fade_in();
        }
    }

    fn fade_in(&self) {
        self.panel.setAlphaValue(0.0);
        self.panel.orderFrontRegardless();
        let panel = self.panel.clone();
        let changes = RcBlock::new(move |ctx: std::ptr::NonNull<NSAnimationContext>| {
            unsafe { ctx.as_ref() }.setDuration(0.22);
            panel.animator().setAlphaValue(1.0);
        });
        NSAnimationContext::runAnimationGroup(&changes);
    }

    fn screen_with_pointer(mtm: MainThreadMarker) -> Option<Retained<NSScreen>> {
        let mouse = NSEvent::mouseLocation();
        NSScreen::screens(mtm)
            .iter()
            .find(|s| {
                let f = s.frame();
                mouse.x >= f.origin.x
                    && mouse.x <= f.origin.x + f.size.width
                    && mouse.y >= f.origin.y
                    && mouse.y <= f.origin.y + f.size.height
            })
            .or_else(|| NSScreen::mainScreen(mtm))
    }

    fn build(&self, c: &NoticeContent) -> Retained<NSBox> {
        let mtm = self.mtm;
        let target: &AnyObject = &self.handler;
        let inner = WIDTH - 2.0 * PADDING;

        // Same near-black surface as the control center, with a tinted edge.
        let root = UIComponents::create_rounded_box(CORNER, mtm);
        root.setTranslatesAutoresizingMaskIntoConstraints(true);
        root.setFillColor(&NSColor::colorWithWhite_alpha(0.07, 0.96));
        root.setBorderColor(&c.tint.colorWithAlphaComponent(0.35));
        root.setBorderWidth(1.0);

        let stack = UIComponents::create_v_stack(16.0, mtm);
        stack.setEdgeInsets(NSEdgeInsets { top: PADDING, left: PADDING, bottom: PADDING, right: PADDING });
        stack.widthAnchor().constraintEqualToConstant(WIDTH).setActive(true);

        // Header: icon, caption and title.
        let header = UIComponents::create_h_stack(14.0, mtm);
        header.setAlignment(objc2_app_kit::NSLayoutAttribute::CenterY);
        header.addView_inGravity(&UIComponents::create_icon_badge(c.symbol, &c.tint, 48.0, mtm), NSStackViewGravity::Leading);
        let titles = UIComponents::create_v_stack(3.0, mtm);
        let caption = UIComponents::create_caption(c.caption, mtm);
        caption.setTextColor(Some(&c.tint));
        titles.addView_inGravity(&caption, NSStackViewGravity::Top);
        let title = NSTextField::wrappingLabelWithString(&NSString::from_str(c.title), mtm);
        title.setFont(Some(&NSFont::systemFontOfSize_weight(17.0, WEIGHT_SEMIBOLD)));
        title.setTextColor(Some(&NSColor::labelColor()));
        title.setSelectable(false);
        title.setPreferredMaxLayoutWidth(inner - 48.0 - 14.0);
        titles.addView_inGravity(&title, NSStackViewGravity::Top);
        header.addView_inGravity(&titles, NSStackViewGravity::Leading);
        UIComponents::add_full_width(&stack, &header);

        if !c.message.is_empty() {
            let message = UIComponents::create_wrapping_label(c.message, 13.0, mtm);
            message.setPreferredMaxLayoutWidth(inner);
            UIComponents::add_full_width(&stack, &message);
        }

        if !c.facts.is_empty() {
            let facts = UIComponents::create_h_stack(8.0, mtm);
            for (symbol, text) in &c.facts {
                facts.addView_inGravity(&Self::fact_chip(symbol, text, mtm), NSStackViewGravity::Leading);
            }
            stack.addView_inGravity(&facts, NSStackViewGravity::Top);
        }
        stack.setCustomSpacing_afterView(20.0, &stack.views().lastObject().unwrap());

        // Buttons: secondary, then the filled primary action.
        let secondary = Self::button(
            c.secondary.0,
            None,
            &NSColor::colorWithWhite_alpha(1.0, 0.09),
            &NSColor::labelColor(),
            target,
            c.secondary.1,
            mtm,
        );
        // A touch darker than the tint, so white text stays readable.
        let fill = c.tint.blendedColorWithFraction_ofColor(0.2, &NSColor::blackColor()).unwrap_or(c.tint.clone());
        let primary = Self::button(
            c.primary.0,
            Some(c.primary.1),
            &fill,
            &NSColor::whiteColor(),
            target,
            c.primary.2,
            mtm,
        );
        let secondary_view: &NSView = &secondary;
        let primary_view: &NSView = &primary;
        let buttons = UIComponents::create_equal_row(&[secondary_view, primary_view], 10.0, mtm);
        UIComponents::add_full_width(&stack, &buttons);

        UIComponents::set_box_content(&root, &stack);
        root
    }

    /// Rounded filled button. Drawn by hand: native push buttons turn grey
    /// in a panel that isn't key, which this one usually isn't.
    fn button(
        title: &str,
        symbol: Option<&str>,
        fill: &NSColor,
        text: &NSColor,
        target: &AnyObject,
        action: Sel,
        mtm: MainThreadMarker,
    ) -> Retained<NSBox> {
        let surface = UIComponents::create_rounded_box(10.0, mtm);
        surface.setBorderWidth(0.0);
        surface.setFillColor(fill);
        let btn = unsafe {
            NSButton::buttonWithTitle_target_action(&NSString::from_str(title), Some(target), Some(action), mtm)
        };
        btn.setBordered(false);
        btn.setFont(Some(&NSFont::systemFontOfSize_weight(13.5, WEIGHT_SEMIBOLD)));
        btn.setContentTintColor(Some(text));
        if let Some(img) = symbol.and_then(|s| UIComponents::symbol(s, 13.0, WEIGHT_SEMIBOLD)) {
            btn.setImage(Some(&img));
            btn.setImagePosition(NSCellImagePosition::ImageLeading);
            btn.setImageHugsTitle(true);
        }
        UIComponents::set_box_content(&surface, &btn);
        surface.heightAnchor().constraintEqualToConstant(38.0).setActive(true);
        surface
    }

    fn fact_chip(symbol: &str, text: &str, mtm: MainThreadMarker) -> Retained<NSBox> {
        let chip = UIComponents::create_rounded_box(10.0, mtm);
        chip.setBorderWidth(0.0);
        chip.setFillColor(&NSColor::colorWithWhite_alpha(1.0, 0.07));
        let row = UIComponents::create_h_stack(6.0, mtm);
        row.setEdgeInsets(NSEdgeInsets { top: 5.0, left: 9.0, bottom: 5.0, right: 10.0 });
        row.addView_inGravity(
            &UIComponents::create_symbol_view(symbol, 10.5, &NSColor::secondaryLabelColor(), mtm),
            NSStackViewGravity::Leading,
        );
        let label = UIComponents::create_weighted_label(text, 11.5, WEIGHT_MEDIUM, mtm);
        label.setTextColor(Some(&NSColor::secondaryLabelColor()));
        row.addView_inGravity(&label, NSStackViewGravity::Leading);
        row.setHuggingPriority_forOrientation(750.0, NSLayoutConstraintOrientation::Horizontal);
        UIComponents::set_box_content(&chip, &row);
        chip
    }
}
