use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::sel;
use objc2_app_kit::{NSMenu, NSMenuItem, NSStatusBar, NSStatusItem};
use objc2_foundation::{MainThreadMarker, NSString};

use crate::action_handler::HealthTasksActionHandler;
use crate::app_state::AppState;

pub struct MenuBarManager {
    pub status_item: Retained<NSStatusItem>,
    pub activity_item: Retained<NSMenuItem>,
    pub notice_item: Retained<NSMenuItem>,
    pub water_item: Retained<NSMenuItem>,
    pub coffee_item: Retained<NSMenuItem>,
    pub progress_item: Retained<NSMenuItem>,
    pub sound_item: Retained<NSMenuItem>,
    pub control_center_item: Retained<NSMenuItem>,
}

impl MenuBarManager {
    pub fn build(
        handler: &HealthTasksActionHandler,
        state: &AppState,
        mtm: MainThreadMarker,
    ) -> Self {
        let status_bar = NSStatusBar::systemStatusBar();
        let status_item = status_bar.statusItemWithLength(-1.0); // Variable length
        let target: &AnyObject = handler;

        if let Some(button) = status_item.button(mtm) {
            let title = state.menu_bar_title();
            button.setTitle(&NSString::from_str(&title));
        }

        let menu = NSMenu::initWithTitle(mtm.alloc(), &NSString::from_str(crate::APP_NAME));

        // 1. App Header Title
        let header_item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                mtm.alloc(),
                &NSString::from_str(&format!("❤️ {} Companion", crate::APP_NAME)),
                None,
                &NSString::from_str(""),
            )
        };
        header_item.setEnabled(false);
        menu.addItem(&header_item);

        menu.addItem(&NSMenuItem::separatorItem(mtm));

        // 2. Active Session Work Time (since last break)
        let work_mins = state.activity.active_work_seconds / 60;
        let work_secs = state.activity.active_work_seconds % 60;
        let started_str = state.activity.session_start_time.as_deref().unwrap_or("Ready");
        let activity_title = format!(
            "⏱️ Active Time: {:02}:{:02} (Started: {})",
            work_mins, work_secs, started_str
        );
        let activity_item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                mtm.alloc(),
                &NSString::from_str(&activity_title),
                None,
                &NSString::from_str(""),
            )
        };
        activity_item.setEnabled(false);
        menu.addItem(&activity_item);

        // 3. Notice Countdown
        let remaining = state.next_notice_in();
        let notice_title = format!("🔔 Break Notice in: {:02}:{:02}", remaining / 60, remaining % 60);
        let notice_item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                mtm.alloc(),
                &NSString::from_str(&notice_title),
                None,
                &NSString::from_str(""),
            )
        };
        notice_item.setEnabled(false);
        menu.addItem(&notice_item);

        // 4. Take Break Now / Reset Timer
        let take_break_item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                mtm.alloc(),
                &NSString::from_str("🌿 Take a Break Now / Reset Timer"),
                Some(sel!(onManualTakeBreak:)),
                &NSString::from_str("b"),
            )
        };
        unsafe { take_break_item.setTarget(Some(target)) };
        menu.addItem(&take_break_item);

        menu.addItem(&NSMenuItem::separatorItem(mtm));

        // 5. Quick Log Water
        let water_title = format!(
            "💧 Quick Log Water (+250ml) [{} / {} ml]",
            state.today.water_ml, state.today.water_goal_ml
        );
        let water_item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                mtm.alloc(),
                &NSString::from_str(&water_title),
                Some(sel!(onAddWater250:)),
                &NSString::from_str("w"),
            )
        };
        unsafe { water_item.setTarget(Some(target)) };
        menu.addItem(&water_item);

        // 5b. Quick Log Coffee
        let coffee_item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                mtm.alloc(),
                &NSString::from_str(&coffee_title(state)),
                Some(sel!(onAddCoffee:)),
                &NSString::from_str("c"),
            )
        };
        unsafe { coffee_item.setTarget(Some(target)) };
        menu.addItem(&coffee_item);

        // 6. Quick Stand & Stretch Break
        let stretch_item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                mtm.alloc(),
                &NSString::from_str("🧍 Quick Log: Stand & Stretch Done"),
                Some(sel!(onQuickBreakDone:)),
                &NSString::from_str("s"),
            )
        };
        stretch_item.setTag(1);
        unsafe { stretch_item.setTarget(Some(target)) };
        menu.addItem(&stretch_item);

        // 7. Quick Eye Break
        let eye_item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                mtm.alloc(),
                &NSString::from_str("👀 Quick Log: 20-20-20 Eye Break Done"),
                Some(sel!(onQuickBreakDone:)),
                &NSString::from_str("e"),
            )
        };
        eye_item.setTag(0);
        unsafe { eye_item.setTarget(Some(target)) };
        menu.addItem(&eye_item);

        menu.addItem(&NSMenuItem::separatorItem(mtm));

        // 8. Progress line
        let progress_title = format!(
            "📋 Today: {}/{} Habits Done ({:.0}%)",
            state.today.completed_count(),
            state.today.tasks.len(),
            state.today.completion_ratio() * 100.0
        );
        let progress_item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                mtm.alloc(),
                &NSString::from_str(&progress_title),
                None,
                &NSString::from_str(""),
            )
        };
        progress_item.setEnabled(false);
        menu.addItem(&progress_item);

        menu.addItem(&NSMenuItem::separatorItem(mtm));

        // 9. Show Dashboard
        let show_dashboard = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                mtm.alloc(),
                &NSString::from_str(&format!("🖥 Open {} Dashboard", crate::APP_NAME)),
                Some(sel!(onShowDashboard:)),
                &NSString::from_str("d"),
            )
        };
        unsafe { show_dashboard.setTarget(Some(target)) };
        menu.addItem(&show_dashboard);

        // 9b. Floating control center toggle
        let control_center_item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                mtm.alloc(),
                &NSString::from_str(control_center_title(state)),
                Some(sel!(onToggleControlCenter:)),
                &NSString::from_str("k"),
            )
        };
        unsafe { control_center_item.setTarget(Some(target)) };
        menu.addItem(&control_center_item);

        // 10. Sound Toggle
        let sound_title = if state.settings.sound_enabled {
            "🔔 Sound Effects: Enabled"
        } else {
            "🔕 Sound Effects: Muted"
        };
        let sound_item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                mtm.alloc(),
                &NSString::from_str(sound_title),
                Some(sel!(onToggleSound:)),
                &NSString::from_str(""),
            )
        };
        unsafe { sound_item.setTarget(Some(target)) };
        menu.addItem(&sound_item);

        menu.addItem(&NSMenuItem::separatorItem(mtm));

        // 11. Quit App
        let quit_item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                mtm.alloc(),
                &NSString::from_str(&format!("Quit {}", crate::APP_NAME)),
                Some(sel!(onQuitApp:)),
                &NSString::from_str("q"),
            )
        };
        unsafe { quit_item.setTarget(Some(target)) };
        menu.addItem(&quit_item);

        status_item.setMenu(Some(&menu));

        Self {
            status_item,
            activity_item,
            notice_item,
            water_item,
            coffee_item,
            progress_item,
            sound_item,
            control_center_item,
        }
    }

    pub fn refresh(&self, state: &AppState, mtm: MainThreadMarker) {
        if let Some(button) = self.status_item.button(mtm) {
            let title = state.menu_bar_title();
            button.setTitle(&NSString::from_str(&title));
        }

        let work_mins = state.activity.active_work_seconds / 60;
        let work_secs = state.activity.active_work_seconds % 60;
        let started_str = state.activity.session_start_time.as_deref().unwrap_or("Ready");
        let activity_title = format!(
            "⏱️ Active Time: {:02}:{:02} (Started: {})",
            work_mins, work_secs, started_str
        );
        self.activity_item
            .setTitle(&NSString::from_str(&activity_title));

        let remaining = state.next_notice_in();
        let notice_title = format!("🔔 Break Notice in: {:02}:{:02}", remaining / 60, remaining % 60);
        self.notice_item
            .setTitle(&NSString::from_str(&notice_title));

        let water_title = format!(
            "💧 Quick Log Water (+250ml) [{} / {} ml]",
            state.today.water_ml, state.today.water_goal_ml
        );
        self.water_item
            .setTitle(&NSString::from_str(&water_title));
        self.coffee_item
            .setTitle(&NSString::from_str(&coffee_title(state)));
        self.control_center_item
            .setTitle(&NSString::from_str(control_center_title(state)));

        let progress_title = format!(
            "📋 Today: {}/{} Habits Done ({:.0}%)",
            state.today.completed_count(),
            state.today.tasks.len(),
            state.today.completion_ratio() * 100.0
        );
        self.progress_item
            .setTitle(&NSString::from_str(&progress_title));

        let sound_title = if state.settings.sound_enabled {
            "🔔 Sound Effects: Enabled"
        } else {
            "🔕 Sound Effects: Muted"
        };
        self.sound_item
            .setTitle(&NSString::from_str(sound_title));
    }
}

fn coffee_title(state: &AppState) -> String {
    format!(
        "☕ Quick Log Coffee [{} cups · {} / {} mg]",
        state.today.coffee_cups(),
        state.today.caffeine_mg(),
        crate::models::CAFFEINE_LIMIT_MG
    )
}

fn control_center_title(state: &AppState) -> &'static str {
    if state.settings.control_center_visible {
        "🎛 Hide Control Center"
    } else {
        "🎛 Show Control Center"
    }
}
