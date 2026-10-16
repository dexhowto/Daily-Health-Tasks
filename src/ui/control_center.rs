//! Floating "control center": a small vertical panel that stays above other
//! windows on every Space (including full-screen apps), shows the session
//! timer, quick-log buttons, today's stats and open tasks, and collapses to a
//! draggable tile.

use std::cell::Cell;

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, Sel};
use objc2::{define_class, msg_send, sel, DefinedClass, MainThreadOnly, Message};
use objc2_app_kit::{
    NSBackingStoreType, NSButton, NSCellImagePosition, NSColor, NSControlSize, NSEvent, NSFont,
    NSImageView, NSLayoutConstraintOrientation, NSLineBreakMode, NSPanel, NSScreen, NSStackView,
    NSStackViewGravity, NSTextField, NSView, NSVisualEffectBlendingMode, NSVisualEffectMaterial,
    NSVisualEffectState, NSVisualEffectView, NSWindowCollectionBehavior, NSWindowStyleMask,
};
use objc2_foundation::{MainThreadMarker, NSPoint, NSRect, NSSize, NSString};

use crate::action_handler::HealthTasksActionHandler;
use crate::app_state::AppState;
use crate::controller;
use crate::models::{ActivityStatus, CAFFEINE_LIMIT_MG};
use crate::reminders::{relative_day, Access, Snapshot};
use crate::ui::components::{ProgressBar, UIComponents, WEIGHT_MEDIUM, WEIGHT_SEMIBOLD};
use crate::ui::dashboard_view::caffeine_tint;

const WIDTH: f64 = 272.0;
const TILE: f64 = 60.0;
const CORNER: f64 = 16.0;
const MAX_TASKS: usize = 8;
const EDGE_MARGIN: f64 = 16.0;

pub struct ControlCenter {
    pub panel: Retained<NSPanel>,
    root: Retained<NSVisualEffectView>,
    expanded: Retained<NSStackView>,
    tile: Retained<CollapsedTile>,
    handler: Retained<HealthTasksActionHandler>,
    timer_label: Retained<NSTextField>,
    water_value: Retained<NSTextField>,
    water_bar: ProgressBar,
    coffee_value: Retained<NSTextField>,
    coffee_bar: ProgressBar,
    habits_value: Retained<NSTextField>,
    habits_bar: ProgressBar,
    tasks_caption: Retained<NSTextField>,
    task_rows: Retained<NSStackView>,
    collapsed: Cell<bool>,
    /// Collapsed automatically for a screen recording; restore when it ends.
    collapsed_for_recording: Cell<bool>,
    mtm: MainThreadMarker,
}

impl ControlCenter {
    pub fn build(handler: &HealthTasksActionHandler, state: &AppState, mtm: MainThreadMarker) -> Self {
        let target: &AnyObject = handler;
        let panel = Self::make_panel(mtm);

        let root = NSVisualEffectView::new(mtm);
        root.setMaterial(NSVisualEffectMaterial::HUDWindow);
        root.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);
        root.setState(NSVisualEffectState::Active);
        root.setWantsLayer(true);
        if let Some(layer) = root.layer() {
            layer.setCornerRadius(CORNER);
            layer.setMasksToBounds(true);
        }
        panel.setContentView(Some(&root));

        // --- Expanded layout ---
        let expanded = UIComponents::create_v_stack(12.0, mtm);
        expanded.setEdgeInsets(objc2_foundation::NSEdgeInsets { top: 12.0, left: 14.0, bottom: 14.0, right: 14.0 });
        expanded.widthAnchor().constraintEqualToConstant(WIDTH).setActive(true);

        let header = UIComponents::create_h_stack(8.0, mtm);
        if let Some(icon) = UIComponents::app_icon(mtm) {
            let logo = NSImageView::imageViewWithImage(&icon, mtm);
            logo.setImageScaling(objc2_app_kit::NSImageScaling::ScaleProportionallyUpOrDown);
            logo.setTranslatesAutoresizingMaskIntoConstraints(false);
            UIComponents::pin_size(&logo, 22.0, 22.0);
            header.addView_inGravity(&logo, NSStackViewGravity::Leading);
        }
        header.addView_inGravity(
            &UIComponents::create_weighted_label("HealthTask", 13.0, WEIGHT_SEMIBOLD, mtm),
            NSStackViewGravity::Leading,
        );
        let timer_label = UIComponents::create_secondary_label("", 12.0, mtm);
        timer_label.setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(12.0, WEIGHT_MEDIUM)));
        header.addView_inGravity(&timer_label, NSStackViewGravity::Trailing);
        let collapse = Self::icon_button("chevron.down.circle.fill", "Minimize", target, sel!(onToggleControlCenterCollapsed:), mtm);
        header.addView_inGravity(&collapse, NSStackViewGravity::Trailing);
        UIComponents::add_full_width(&expanded, &header);

        // Quick actions.
        let actions: [(&str, &str, Retained<NSColor>, Sel); 4] = [
            ("Water", "drop.fill", NSColor::systemBlueColor(), sel!(onAddWater250:)),
            ("Coffee", "cup.and.saucer.fill", NSColor::systemBrownColor(), sel!(onAddCoffee:)),
            ("Break", "figure.walk", NSColor::systemGreenColor(), sel!(onManualTakeBreak:)),
            ("Open", "macwindow", NSColor::labelColor(), sel!(onShowDashboard:)),
        ];
        let action_views: Vec<Retained<NSView>> = actions
            .iter()
            .map(|(title, symbol, tint, action)| Self::action_tile(title, symbol, tint, target, *action, mtm))
            .collect();
        let action_refs: Vec<&NSView> = action_views.iter().map(|v| &**v).collect();
        let action_row = UIComponents::create_equal_row(&action_refs, 8.0, mtm);
        UIComponents::add_full_width(&expanded, &action_row);

        // Stats.
        let (water_row, water_value, water_bar) =
            Self::stat_row("drop.fill", "Water", &NSColor::systemBlueColor(), mtm);
        let (coffee_row, coffee_value, coffee_bar) =
            Self::stat_row("cup.and.saucer.fill", "Caffeine", &NSColor::systemBrownColor(), mtm);
        let (habits_row, habits_value, habits_bar) =
            Self::stat_row("checkmark.circle.fill", "Habits", &NSColor::systemGreenColor(), mtm);
        let stats = UIComponents::create_v_stack(7.0, mtm);
        for row in [&water_row, &coffee_row, &habits_row] {
            UIComponents::add_full_width(&stats, row);
        }
        UIComponents::add_full_width(&expanded, &stats);

        // Tasks (open Reminders).
        UIComponents::add_full_width(&expanded, &UIComponents::create_separator(mtm));
        let tasks_caption = UIComponents::create_caption("Tasks", mtm);
        expanded.addView_inGravity(&tasks_caption, NSStackViewGravity::Top);
        expanded.setCustomSpacing_afterView(6.0, &tasks_caption);
        let task_rows = UIComponents::create_v_stack(6.0, mtm);
        UIComponents::add_full_width(&expanded, &task_rows);

        // --- Collapsed tile ---
        let tile = CollapsedTile::new(mtm);

        let cc = Self {
            panel,
            root,
            expanded,
            tile,
            handler: handler.retain(),
            timer_label,
            water_value,
            water_bar,
            coffee_value,
            coffee_bar,
            habits_value,
            habits_bar,
            tasks_caption,
            task_rows,
            collapsed: Cell::new(state.settings.control_center_collapsed),
            collapsed_for_recording: Cell::new(false),
            mtm,
        };
        cc.refresh(state);
        cc.update_tasks(&Snapshot::Loading);
        cc.install_current_view();
        cc.place(state.settings.control_center_top_left);
        if state.settings.control_center_visible {
            cc.panel.orderFrontRegardless();
        }
        cc
    }

    fn make_panel(mtm: MainThreadMarker) -> Retained<NSPanel> {
        let frame = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(WIDTH, 300.0));
        let panel = NSPanel::initWithContentRect_styleMask_backing_defer(
            mtm.alloc(),
            frame,
            NSWindowStyleMask::Borderless | NSWindowStyleMask::NonactivatingPanel,
            NSBackingStoreType::Buffered,
            false,
        );
        panel.setFloatingPanel(true);
        panel.setLevel(objc2_app_kit::NSStatusWindowLevel);
        // Every Space, including full-screen apps; not part of Cmd+` cycling.
        panel.setCollectionBehavior(
            NSWindowCollectionBehavior::CanJoinAllSpaces
                | NSWindowCollectionBehavior::FullScreenAuxiliary
                | NSWindowCollectionBehavior::Stationary
                | NSWindowCollectionBehavior::IgnoresCycle,
        );
        panel.setHidesOnDeactivate(false);
        panel.setBecomesKeyOnlyIfNeeded(true);
        panel.setMovableByWindowBackground(true);
        panel.setOpaque(false);
        panel.setBackgroundColor(Some(&NSColor::clearColor()));
        panel.setHasShadow(true);
        unsafe { panel.setReleasedWhenClosed(false) };
        panel
    }

    fn icon_button(symbol: &str, tooltip: &str, target: &AnyObject, action: Sel, mtm: MainThreadMarker) -> Retained<NSButton> {
        let btn = unsafe {
            NSButton::buttonWithTitle_target_action(&NSString::from_str(""), Some(target), Some(action), mtm)
        };
        btn.setBordered(false);
        if let Some(img) = UIComponents::symbol(symbol, 15.0, WEIGHT_MEDIUM) {
            btn.setImage(Some(&img));
            btn.setImagePosition(NSCellImagePosition::ImageOnly);
        }
        btn.setContentTintColor(Some(&NSColor::secondaryLabelColor()));
        btn.setToolTip(Some(&NSString::from_str(tooltip)));
        btn
    }

    /// Rounded square button with a tinted symbol above a small title.
    fn action_tile(
        title: &str,
        symbol: &str,
        tint: &NSColor,
        target: &AnyObject,
        action: Sel,
        mtm: MainThreadMarker,
    ) -> Retained<NSView> {
        let tile = UIComponents::create_rounded_box(10.0, mtm);
        tile.setBorderWidth(0.0);
        tile.setFillColor(&NSColor::labelColor().colorWithAlphaComponent(0.07));
        tile.heightAnchor().constraintEqualToConstant(58.0).setActive(true);

        let btn = unsafe {
            NSButton::buttonWithTitle_target_action(&NSString::from_str(title), Some(target), Some(action), mtm)
        };
        btn.setBordered(false);
        btn.setFont(Some(&NSFont::systemFontOfSize_weight(10.5, WEIGHT_MEDIUM)));
        if let Some(img) = UIComponents::symbol(symbol, 16.0, WEIGHT_SEMIBOLD) {
            btn.setImage(Some(&img));
            btn.setImagePosition(NSCellImagePosition::ImageAbove);
        }
        btn.setContentTintColor(Some(tint));
        btn.setTranslatesAutoresizingMaskIntoConstraints(false);
        tile.addSubview(&btn);
        // Center the icon+title in the tile; the button keeps its natural height.
        UIComponents::activate(&[
            btn.leadingAnchor().constraintEqualToAnchor(&tile.leadingAnchor()),
            btn.trailingAnchor().constraintEqualToAnchor(&tile.trailingAnchor()),
            btn.centerYAnchor().constraintEqualToAnchor(&tile.centerYAnchor()),
        ]);
        Retained::into_super(tile)
    }

    fn stat_row(
        symbol: &str,
        label: &str,
        tint: &NSColor,
        mtm: MainThreadMarker,
    ) -> (Retained<NSStackView>, Retained<NSTextField>, ProgressBar) {
        let row = UIComponents::create_h_stack(8.0, mtm);
        let icon = UIComponents::create_symbol_view(symbol, 11.0, tint, mtm);
        icon.setTranslatesAutoresizingMaskIntoConstraints(false);
        icon.widthAnchor().constraintEqualToConstant(14.0).setActive(true);
        row.addView_inGravity(&icon, NSStackViewGravity::Leading);
        let name = UIComponents::create_secondary_label(label, 11.5, mtm);
        name.widthAnchor().constraintEqualToConstant(54.0).setActive(true);
        row.addView_inGravity(&name, NSStackViewGravity::Leading);
        let bar = ProgressBar::new(0.0, tint, 5.0, mtm);
        bar.track.setContentHuggingPriority_forOrientation(1.0, NSLayoutConstraintOrientation::Horizontal);
        row.addView_inGravity(&bar.track, NSStackViewGravity::Leading);
        let value = UIComponents::create_label("", 11.5, false, mtm);
        value.setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(11.5, WEIGHT_MEDIUM)));
        value.setAlignment(objc2_app_kit::NSTextAlignment::Right);
        value.widthAnchor().constraintEqualToConstant(78.0).setActive(true);
        row.addView_inGravity(&value, NSStackViewGravity::Trailing);
        (row, value, bar)
    }

    // --- State updates ---

    pub fn refresh(&self, state: &AppState) {
        let worked = state.activity.active_work_seconds;
        let timer = match state.activity.status {
            ActivityStatus::AwayOnBreak => "On break".to_string(),
            ActivityStatus::WaitingForInput => "Ready".to_string(),
            ActivityStatus::NoticeDue => format!("{:02}:{:02} · break!", worked / 60, worked % 60),
            ActivityStatus::ActiveWorking => format!("{:02}:{:02}", worked / 60, worked % 60),
        };
        self.timer_label.setStringValue(&NSString::from_str(&timer));
        let timer_color = if state.activity.status == ActivityStatus::NoticeDue {
            NSColor::systemOrangeColor()
        } else {
            NSColor::secondaryLabelColor()
        };
        self.timer_label.setTextColor(Some(&timer_color));

        let today = &state.today;
        self.water_value.setStringValue(&NSString::from_str(&format!(
            "{:.2}/{:.1} L",
            today.water_ml as f64 / 1000.0,
            today.water_goal_ml as f64 / 1000.0
        )));
        self.water_bar.set_ratio(today.water_ml as f64 / today.water_goal_ml.max(1) as f64);

        let mg = today.caffeine_mg();
        self.coffee_value
            .setStringValue(&NSString::from_str(&format!("{mg}/{CAFFEINE_LIMIT_MG} mg")));
        self.coffee_bar.set_ratio(mg as f64 / CAFFEINE_LIMIT_MG as f64);
        self.coffee_bar.set_tint(&caffeine_tint(mg));

        self.habits_value.setStringValue(&NSString::from_str(&format!(
            "{}/{}",
            today.completed_count(),
            today.tasks.len()
        )));
        self.habits_bar.set_ratio(today.completion_ratio());
    }

    pub fn update_tasks(&self, snapshot: &Snapshot) {
        let mtm = self.mtm;
        let target: &AnyObject = &self.handler;
        for view in self.task_rows.arrangedSubviews().iter() {
            self.task_rows.removeArrangedSubview(&view);
            view.removeFromSuperview();
        }

        let message = match snapshot {
            Snapshot::Access(Access::Granted) | Snapshot::Loading => Some("Loading…"),
            Snapshot::Access(_) => Some("Connect Reminders on the Today page."),
            Snapshot::Items(items) if items.is_empty() => Some("No open tasks today."),
            Snapshot::Items(_) => None,
        };
        let items = match snapshot {
            Snapshot::Items(items) => items.as_slice(),
            _ => &[],
        };
        self.tasks_caption.setStringValue(&NSString::from_str(&if items.is_empty() {
            "TASKS".to_string()
        } else {
            format!("TASKS · {}", items.len())
        }));
        if let Some(text) = message {
            self.task_rows.addView_inGravity(
                &UIComponents::create_secondary_label(text, 11.5, mtm),
                NSStackViewGravity::Top,
            );
        }

        for (idx, item) in items.iter().take(MAX_TASKS).enumerate() {
            let row = UIComponents::create_h_stack(6.0, mtm);
            let checkbox = UIComponents::create_checkbox("", false, idx, target, sel!(onToggleReminder:), mtm);
            checkbox.setControlSize(NSControlSize::Small);
            row.addView_inGravity(&checkbox, NSStackViewGravity::Leading);
            let title = UIComponents::create_label(&item.title, 12.0, false, mtm);
            title.setLineBreakMode(NSLineBreakMode::ByTruncatingTail);
            title.setContentCompressionResistancePriority_forOrientation(250.0, NSLayoutConstraintOrientation::Horizontal);
            row.addView_inGravity(&title, NSStackViewGravity::Leading);
            let due_text = if item.is_overdue() {
                item.due_date.map(relative_day).unwrap_or_default()
            } else {
                item.due_time.clone().unwrap_or_default()
            };
            if !due_text.is_empty() {
                let due = UIComponents::create_secondary_label(&due_text, 10.5, mtm);
                if item.is_overdue() {
                    due.setTextColor(Some(&NSColor::systemRedColor()));
                }
                row.addView_inGravity(&due, NSStackViewGravity::Trailing);
            }
            UIComponents::add_full_width(&self.task_rows, &row);
        }

        if items.len() > MAX_TASKS {
            let more = unsafe {
                NSButton::buttonWithTitle_target_action(
                    &NSString::from_str(&format!("+{} more in Reminders", items.len() - MAX_TASKS)),
                    Some(target),
                    Some(sel!(onOpenReminders:)),
                    mtm,
                )
            };
            more.setBordered(false);
            more.setFont(Some(&NSFont::systemFontOfSize_weight(11.0, WEIGHT_MEDIUM)));
            more.setContentTintColor(Some(&NSColor::controlAccentColor()));
            self.task_rows.addView_inGravity(&more, NSStackViewGravity::Top);
        }
        self.tile.set_badge(items.len());
        self.resize_keeping_top_left();
    }

    // --- Visibility, collapse, position ---

    pub fn is_visible(&self) -> bool {
        self.panel.isVisible()
    }

    pub fn set_visible(&self, visible: bool) {
        if visible {
            self.panel.orderFrontRegardless();
        } else {
            self.panel.orderOut(None);
        }
    }

    pub fn is_collapsed(&self) -> bool {
        self.collapsed.get()
    }

    pub fn set_collapsed(&self, collapsed: bool) {
        if self.collapsed.replace(collapsed) != collapsed {
            self.install_current_view();
        }
    }

    /// Collapses while the screen is recorded and restores the previous state after.
    pub fn on_recording_changed(&self, recording: bool) {
        if recording {
            if !self.collapsed.get() {
                self.collapsed_for_recording.set(true);
                self.set_collapsed(true);
            }
        } else if self.collapsed_for_recording.replace(false) {
            self.set_collapsed(false);
        }
    }

    /// Top-left corner in screen coordinates (persisted between launches).
    pub fn top_left(&self) -> (f64, f64) {
        let f = self.panel.frame();
        (f.origin.x, f.origin.y + f.size.height)
    }

    fn install_current_view(&self) {
        let (old, new): (&NSView, &NSView) = if self.collapsed.get() {
            (&self.expanded, &self.tile)
        } else {
            (&self.tile, &self.expanded)
        };
        old.removeFromSuperview();
        new.setTranslatesAutoresizingMaskIntoConstraints(false);
        self.root.addSubview(new);
        UIComponents::activate(&[
            new.leadingAnchor().constraintEqualToAnchor(&self.root.leadingAnchor()),
            new.trailingAnchor().constraintEqualToAnchor(&self.root.trailingAnchor()),
            new.topAnchor().constraintEqualToAnchor(&self.root.topAnchor()),
            new.bottomAnchor().constraintEqualToAnchor(&self.root.bottomAnchor()),
        ]);
        if let Some(layer) = self.root.layer() {
            layer.setCornerRadius(if self.collapsed.get() { TILE * 0.28 } else { CORNER });
        }
        self.resize_keeping_top_left();
    }

    fn resize_keeping_top_left(&self) {
        let (left, top) = self.top_left();
        let view: &NSView = if self.collapsed.get() { &self.tile } else { &self.expanded };
        view.layoutSubtreeIfNeeded();
        let size = view.fittingSize();
        let frame = NSRect::new(NSPoint::new(left, top - size.height), size);
        self.panel.setFrame_display(frame, true);
        // Re-apply the shadow for the new shape.
        self.panel.invalidateShadow();
    }

    /// Moves the panel to the saved corner, or near the top-right of the main
    /// screen when there is none or it is no longer on any screen.
    fn place(&self, saved: Option<(f64, f64)>) {
        let screens = NSScreen::screens(self.mtm);
        let on_screen = |(x, y): (f64, f64)| {
            screens.iter().any(|s| {
                let f = s.frame();
                x >= f.origin.x && x < f.origin.x + f.size.width && y > f.origin.y && y <= f.origin.y + f.size.height
            })
        };
        let (left, top) = match saved.filter(|p| on_screen(*p)) {
            Some(p) => p,
            None => match NSScreen::mainScreen(self.mtm) {
                Some(screen) => {
                    let v = screen.visibleFrame();
                    (v.origin.x + v.size.width - WIDTH - EDGE_MARGIN, v.origin.y + v.size.height - EDGE_MARGIN)
                }
                None => (100.0, 800.0),
            },
        };
        let size = self.panel.frame().size;
        self.panel.setFrameOrigin(NSPoint::new(left, top - size.height));
    }
}

/// Collapsed tile: app icon plus a task-count badge. Click expands, drag moves.
pub struct CollapsedTileIvars {
    dragged: Cell<bool>,
    badge: std::cell::OnceCell<(Retained<objc2_app_kit::NSBox>, Retained<NSTextField>)>,
}

define_class!(
    #[unsafe(super(NSView))]
    #[thread_kind = MainThreadOnly]
    #[name = "HealthTasksCollapsedTile"]
    #[ivars = CollapsedTileIvars]
    pub struct CollapsedTile;

    impl CollapsedTile {
        #[unsafe(method(acceptsFirstMouse:))]
        fn accepts_first_mouse(&self, _event: Option<&NSEvent>) -> bool {
            true
        }

        /// Handle all clicks here rather than in the icon subviews.
        #[unsafe(method(hitTest:))]
        fn hit_test(&self, point: NSPoint) -> *mut NSView {
            let frame = self.frame();
            let inside = point.x >= frame.origin.x
                && point.x <= frame.origin.x + frame.size.width
                && point.y >= frame.origin.y
                && point.y <= frame.origin.y + frame.size.height;
            if inside { self as *const Self as *mut NSView } else { std::ptr::null_mut() }
        }

        #[unsafe(method(mouseDown:))]
        fn mouse_down(&self, _event: &NSEvent) {
            self.ivars().dragged.set(false);
        }

        #[unsafe(method(mouseDragged:))]
        fn mouse_dragged(&self, event: &NSEvent) {
            if !self.ivars().dragged.replace(true) {
                if let Some(window) = self.window() {
                    window.performWindowDragWithEvent(event);
                }
            }
        }

        #[unsafe(method(mouseUp:))]
        fn mouse_up(&self, _event: &NSEvent) {
            if !self.ivars().dragged.get() {
                controller::toggle_control_center_collapsed();
            }
        }
    }
);

impl CollapsedTile {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = mtm.alloc::<Self>().set_ivars(CollapsedTileIvars {
            dragged: Cell::new(false),
            badge: std::cell::OnceCell::new(),
        });
        let tile: Retained<Self> = unsafe { msg_send![super(this), init] };
        tile.setTranslatesAutoresizingMaskIntoConstraints(false);
        UIComponents::pin_size(&tile, TILE, TILE);
        tile.setToolTip(Some(&NSString::from_str("HealthTask: click to expand, drag to move")));

        if let Some(icon) = UIComponents::app_icon(mtm) {
            let logo = NSImageView::imageViewWithImage(&icon, mtm);
            logo.setImageScaling(objc2_app_kit::NSImageScaling::ScaleProportionallyUpOrDown);
            logo.setTranslatesAutoresizingMaskIntoConstraints(false);
            tile.addSubview(&logo);
            UIComponents::activate(&[
                logo.centerXAnchor().constraintEqualToAnchor(&tile.centerXAnchor()),
                logo.centerYAnchor().constraintEqualToAnchor(&tile.centerYAnchor()),
            ]);
            UIComponents::pin_size(&logo, 46.0, 46.0);
        }

        let (badge_box, badge_label) = UIComponents::create_pill_badge("", &NSColor::systemOrangeColor(), mtm);
        badge_box.setFillColor(&NSColor::systemOrangeColor());
        badge_label.setTextColor(Some(&NSColor::whiteColor()));
        tile.addSubview(&badge_box);
        UIComponents::activate(&[
            badge_box.trailingAnchor().constraintEqualToAnchor_constant(&tile.trailingAnchor(), -2.0),
            badge_box.topAnchor().constraintEqualToAnchor_constant(&tile.topAnchor(), 2.0),
        ]);
        badge_box.setHidden(true);
        let _ = tile.ivars().badge.set((badge_box, badge_label));
        tile
    }

    fn set_badge(&self, count: usize) {
        if let Some((pill, label)) = self.ivars().badge.get() {
            label.setStringValue(&NSString::from_str(&count.to_string()));
            pill.setHidden(count == 0);
        }
    }
}
