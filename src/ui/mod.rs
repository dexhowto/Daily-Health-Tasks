pub mod components;
pub mod dashboard_view;
pub mod history_view;
pub mod settings_view;
pub mod timer_view;

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::sel;
use objc2_app_kit::{
    NSBackingStoreType, NSBox, NSButton, NSCellImagePosition, NSColor, NSFont, NSScrollView,
    NSStackViewGravity, NSTextAlignment, NSView, NSVisualEffectBlendingMode,
    NSVisualEffectMaterial, NSVisualEffectState, NSVisualEffectView, NSWindow, NSWindowStyleMask,
    NSWindowTitleVisibility,
};
use objc2_foundation::{MainThreadMarker, NSEdgeInsets, NSPoint, NSRect, NSSize, NSString};

use crate::action_handler::HealthTasksActionHandler;
use crate::app_state::AppState;
use crate::ui::components::{UIComponents, WEIGHT_MEDIUM, WEIGHT_SEMIBOLD};
use crate::ui::dashboard_view::DashboardView;
use crate::ui::history_view::HistoryView;
use crate::ui::settings_view::SettingsView;
use crate::ui::timer_view::TimerView;

const SIDEBAR_WIDTH: f64 = 220.0;
const CONTENT_MAX_WIDTH: f64 = 760.0;

/// Sidebar entries: (tab index, title, SF Symbol). Tab indices match the
/// persisted `selected_tab` values, so the visual order can differ.
const NAV_ITEMS: [(usize, &str, &str); 4] = [
    (0, "Today", "sun.max.fill"),
    (1, "Break Timer", "timer"),
    (3, "Insights", "chart.bar.fill"),
    (2, "Settings", "gearshape.fill"),
];

struct NavItem {
    tab: usize,
    highlight: Retained<NSBox>,
    button: Retained<NSButton>,
}

pub struct MainWindow {
    pub window: Retained<NSWindow>,
    scroll_view: Retained<NSScrollView>,
    nav_items: Vec<NavItem>,
    pub dashboard_view: DashboardView,
    pub timer_view: TimerView,
    pub settings_view: SettingsView,
    pub history_view: HistoryView,
}

impl MainWindow {
    pub fn build(
        handler: &HealthTasksActionHandler,
        state: &AppState,
        mtm: MainThreadMarker,
    ) -> Self {
        let frame = NSRect::new(NSPoint::new(120.0, 100.0), NSSize::new(980.0, 780.0));
        let style = NSWindowStyleMask::Titled
            | NSWindowStyleMask::Closable
            | NSWindowStyleMask::Miniaturizable
            | NSWindowStyleMask::Resizable
            | NSWindowStyleMask::FullSizeContentView;

        let window = unsafe {
            NSWindow::initWithContentRect_styleMask_backing_defer(
                mtm.alloc(),
                frame,
                style,
                NSBackingStoreType::Buffered,
                false,
            )
        };
        window.setTitle(&NSString::from_str(crate::APP_NAME));
        window.setTitlebarAppearsTransparent(true);
        window.setTitleVisibility(NSWindowTitleVisibility::Hidden);
        window.setContentMinSize(NSSize::new(820.0, 600.0));
        unsafe { window.setReleasedWhenClosed(false) };
        window.center();

        let root = NSView::initWithFrame(
            mtm.alloc(),
            NSRect::new(NSPoint::new(0.0, 0.0), frame.size),
        );
        let target: &AnyObject = handler;

        // --- Sidebar ---
        let sidebar = NSVisualEffectView::new(mtm);
        sidebar.setMaterial(NSVisualEffectMaterial::Sidebar);
        sidebar.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);
        sidebar.setState(NSVisualEffectState::FollowsWindowActiveState);
        sidebar.setTranslatesAutoresizingMaskIntoConstraints(false);
        root.addSubview(&sidebar);

        let sidebar_stack = UIComponents::create_v_stack(2.0, mtm);
        sidebar_stack.setEdgeInsets(NSEdgeInsets { top: 56.0, left: 12.0, bottom: 16.0, right: 12.0 });
        sidebar.addSubview(&sidebar_stack);

        let brand = UIComponents::create_h_stack(10.0, mtm);
        brand.setEdgeInsets(NSEdgeInsets { top: 0.0, left: 2.0, bottom: 0.0, right: 0.0 });
        if let Some(icon) = UIComponents::app_icon(mtm) {
            let logo = objc2_app_kit::NSImageView::imageViewWithImage(&icon, mtm);
            logo.setImageScaling(objc2_app_kit::NSImageScaling::ScaleProportionallyUpOrDown);
            logo.setTranslatesAutoresizingMaskIntoConstraints(false);
            UIComponents::pin_size(&logo, 40.0, 40.0);
            brand.addView_inGravity(&logo, NSStackViewGravity::Leading);
        }
        let brand_text = UIComponents::create_v_stack(0.0, mtm);
        brand_text.addView_inGravity(
            &UIComponents::create_secondary_label("DexHowTo", 11.0, mtm),
            NSStackViewGravity::Top,
        );
        brand_text.addView_inGravity(
            &UIComponents::create_weighted_label("HealthTask", 15.0, WEIGHT_SEMIBOLD, mtm),
            NSStackViewGravity::Top,
        );
        brand.addView_inGravity(&brand_text, NSStackViewGravity::Leading);
        sidebar_stack.addView_inGravity(&brand, NSStackViewGravity::Top);
        sidebar_stack.setCustomSpacing_afterView(22.0, &brand);

        let mut nav_items = Vec::new();
        for (tab, title, symbol) in NAV_ITEMS {
            let highlight = UIComponents::create_rounded_box(7.0, mtm);
            highlight.setBorderWidth(0.0);
            highlight.setFillColor(&NSColor::clearColor());

            let button = unsafe {
                NSButton::buttonWithTitle_target_action(
                    &NSString::from_str(title),
                    Some(target),
                    Some(sel!(onTabSwitch:)),
                    mtm,
                )
            };
            button.setBordered(false);
            button.setTag(tab as isize);
            button.setAlignment(NSTextAlignment::Left);
            button.setFont(Some(&NSFont::systemFontOfSize_weight(13.0, WEIGHT_MEDIUM)));
            if let Some(img) = UIComponents::symbol(symbol, 14.0, WEIGHT_MEDIUM) {
                button.setImage(Some(&img));
                button.setImagePosition(NSCellImagePosition::ImageLeading);
            }
            button.setTranslatesAutoresizingMaskIntoConstraints(false);
            highlight.addSubview(&button);
            UIComponents::activate(&[
                highlight.heightAnchor().constraintEqualToConstant(32.0),
                button.leadingAnchor().constraintEqualToAnchor_constant(&highlight.leadingAnchor(), 10.0),
                button.trailingAnchor().constraintEqualToAnchor_constant(&highlight.trailingAnchor(), -10.0),
                button.topAnchor().constraintEqualToAnchor(&highlight.topAnchor()),
                button.bottomAnchor().constraintEqualToAnchor(&highlight.bottomAnchor()),
            ]);

            UIComponents::add_full_width(&sidebar_stack, &highlight);
            nav_items.push(NavItem { tab, highlight, button });
        }

        // --- Content (scrollable) ---
        let scroll_view = NSScrollView::new(mtm);
        scroll_view.setDrawsBackground(true);
        scroll_view.setBackgroundColor(&NSColor::windowBackgroundColor());
        scroll_view.setHasVerticalScroller(true);
        scroll_view.setAutohidesScrollers(true);
        scroll_view.setTranslatesAutoresizingMaskIntoConstraints(false);
        root.addSubview(&scroll_view);

        // Plain filled box: a Separator-type NSBox hugs its 1pt intrinsic height
        // and would collapse the window vertically.
        let divider = UIComponents::create_rounded_box(0.0, mtm);
        divider.setBorderWidth(0.0);
        divider.setFillColor(&NSColor::separatorColor());
        root.addSubview(&divider);

        let document = FlippedView::new(mtm);
        document.setTranslatesAutoresizingMaskIntoConstraints(false);
        scroll_view.setDocumentView(Some(&document));

        let pages = UIComponents::create_v_stack(0.0, mtm);
        pages.setEdgeInsets(NSEdgeInsets { top: 52.0, left: 36.0, bottom: 40.0, right: 36.0 });
        document.addSubview(&pages);

        let dashboard_view = DashboardView::build(handler, state, mtm);
        let timer_view = TimerView::build(handler, state, mtm);
        let settings_view = SettingsView::build(handler, state, mtm);
        let history_view = HistoryView::build(state, mtm);
        for page in [
            &dashboard_view.container,
            &timer_view.container,
            &settings_view.container,
            &history_view.container,
        ] {
            UIComponents::add_full_width(&pages, page);
        }
        pages.setDetachesHiddenViews(true);

        let clip = scroll_view.contentView();
        UIComponents::activate(&[
            sidebar.leadingAnchor().constraintEqualToAnchor(&root.leadingAnchor()),
            sidebar.topAnchor().constraintEqualToAnchor(&root.topAnchor()),
            sidebar.bottomAnchor().constraintEqualToAnchor(&root.bottomAnchor()),
            sidebar.widthAnchor().constraintEqualToConstant(SIDEBAR_WIDTH),
            sidebar_stack.leadingAnchor().constraintEqualToAnchor(&sidebar.leadingAnchor()),
            sidebar_stack.trailingAnchor().constraintEqualToAnchor(&sidebar.trailingAnchor()),
            sidebar_stack.topAnchor().constraintEqualToAnchor(&sidebar.topAnchor()),
            divider.leadingAnchor().constraintEqualToAnchor(&sidebar.trailingAnchor()),
            divider.topAnchor().constraintEqualToAnchor(&root.topAnchor()),
            divider.bottomAnchor().constraintEqualToAnchor(&root.bottomAnchor()),
            divider.widthAnchor().constraintEqualToConstant(1.0),
            scroll_view.leadingAnchor().constraintEqualToAnchor(&divider.trailingAnchor()),
            scroll_view.trailingAnchor().constraintEqualToAnchor(&root.trailingAnchor()),
            scroll_view.topAnchor().constraintEqualToAnchor(&root.topAnchor()),
            scroll_view.bottomAnchor().constraintEqualToAnchor(&root.bottomAnchor()),
            document.leadingAnchor().constraintEqualToAnchor(&clip.leadingAnchor()),
            document.trailingAnchor().constraintEqualToAnchor(&clip.trailingAnchor()),
            document.topAnchor().constraintEqualToAnchor(&clip.topAnchor()),
            pages.topAnchor().constraintEqualToAnchor(&document.topAnchor()),
            pages.bottomAnchor().constraintEqualToAnchor(&document.bottomAnchor()),
            pages.centerXAnchor().constraintEqualToAnchor(&document.centerXAnchor()),
            pages.widthAnchor().constraintLessThanOrEqualToConstant(CONTENT_MAX_WIDTH),
            pages.widthAnchor().constraintLessThanOrEqualToAnchor(&document.widthAnchor()),
        ]);
        // Fill the available width up to the max.
        let fill = pages.widthAnchor().constraintEqualToAnchor(&document.widthAnchor());
        fill.setPriority(750.0);
        fill.setActive(true);

        window.setContentView(Some(&root));

        let main_window = Self {
            window,
            scroll_view,
            nav_items,
            dashboard_view,
            timer_view,
            settings_view,
            history_view,
        };
        main_window.switch_tab(state.settings.selected_tab);
        main_window
    }

    pub fn switch_tab(&self, index: usize) {
        self.dashboard_view.container.setHidden(index != 0);
        self.timer_view.container.setHidden(index != 1);
        self.settings_view.container.setHidden(index != 2);
        self.history_view.container.setHidden(index != 3);

        for item in &self.nav_items {
            let selected = item.tab == index;
            if selected {
                item.highlight
                    .setFillColor(&NSColor::controlAccentColor().colorWithAlphaComponent(0.16));
                item.button.setContentTintColor(Some(&NSColor::controlAccentColor()));
            } else {
                item.highlight.setFillColor(&NSColor::clearColor());
                item.button.setContentTintColor(Some(&NSColor::secondaryLabelColor()));
            }
        }
        if let Some(doc) = self.scroll_view.documentView() {
            doc.scrollPoint(NSPoint::new(0.0, 0.0));
        }
    }

    pub fn refresh(&self, state: &AppState, mtm: MainThreadMarker) {
        self.dashboard_view.refresh(state);
        self.timer_view.refresh(state);
        self.settings_view.refresh(state);
        self.history_view.refresh(state, mtm);
    }

    pub fn show(&self) {
        self.window.makeKeyAndOrderFront(None);
    }
}

objc2::define_class!(
    /// Top-to-bottom document view so short pages sit at the top of the scroll view.
    #[unsafe(super(NSView))]
    #[thread_kind = objc2::MainThreadOnly]
    #[name = "HealthTasksFlippedView"]
    struct FlippedView;

    impl FlippedView {
        #[unsafe(method(isFlipped))]
        fn is_flipped(&self) -> bool {
            true
        }
    }
);

impl FlippedView {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { objc2::msg_send![mtm.alloc::<Self>(), init] }
    }
}
