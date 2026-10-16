use std::cell::{Cell, RefCell};
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::sel;
use objc2_app_kit::NSApplication;
use objc2_foundation::{MainThreadMarker, NSString, NSTimer};

use crate::action_handler::HealthTasksActionHandler;
use crate::activity_tracker;
use crate::reminders;
use crate::screen_recording;
use crate::app_state;
use crate::menu_bar::MenuBarManager;
use crate::models::{BreakType, CoffeeKind, ExerciseKind};
use crate::notifier::Notifier;
use crate::ui::control_center::ControlCenter;
use crate::ui::MainWindow;

#[allow(dead_code)]
pub struct AppController {
    pub handler: Retained<HealthTasksActionHandler>,
    pub main_window: MainWindow,
    pub menu_bar: MenuBarManager,
    pub control_center: ControlCenter,
    pub timer: Retained<NSTimer>,
    pub mtm: MainThreadMarker,
}

/// Break notice payload: (title, message, sound, show_banner, show_alert).
type Notice = (String, String, String, bool, bool);

thread_local! {
    static CONTROLLER: RefCell<Option<AppController>> = const { RefCell::new(None) };
    /// Notice held back while the screen is being recorded.
    static HELD_NOTICE: RefCell<Option<Notice>> = const { RefCell::new(None) };
    /// Break-timer completion whose dashboard pop-up is held back while recording.
    static HELD_DASHBOARD: Cell<bool> = const { Cell::new(false) };
    static WAS_RECORDING: Cell<bool> = const { Cell::new(false) };
}

pub fn init_controller(mtm: MainThreadMarker) {
    let handler = HealthTasksActionHandler::new(mtm);
    let state = app_state::state().lock().unwrap();

    let main_window = MainWindow::build(&handler, &state, mtm);
    let menu_bar = MenuBarManager::build(&handler, &state, mtm);
    let control_center = ControlCenter::build(&handler, &state, mtm);

    let target_ref: &AnyObject = &handler;
    let timer = unsafe {
        NSTimer::scheduledTimerWithTimeInterval_target_selector_userInfo_repeats(
            1.0,
            target_ref,
            sel!(onTimerTick:),
            None,
            true,
        )
    };

    drop(state);

    let controller = AppController {
        handler,
        main_window,
        menu_bar,
        control_center,
        timer,
        mtm,
    };

    CONTROLLER.with(|c| {
        *c.borrow_mut() = Some(controller);
    });
}

pub fn show_dashboard() {
    CONTROLLER.with(|c| {
        if let Some(ctrl) = c.borrow().as_ref() {
            ctrl.main_window.show();
            let app = NSApplication::sharedApplication(ctrl.mtm);
            app.activate();
        }
    });
}

pub fn quit_app() {
    CONTROLLER.with(|c| {
        if let Some(ctrl) = c.borrow().as_ref() {
            let app = NSApplication::sharedApplication(ctrl.mtm);
            app.terminate(None);
        }
    });
}

pub fn add_water(amount_ml: i32) {
    {
        let mut state = app_state::state().lock().unwrap();
        state.add_water(amount_ml);
    }
    refresh_all();
}

pub fn log_exercise(kind: ExerciseKind) {
    {
        let mut state = app_state::state().lock().unwrap();
        state.log_exercise(kind);
    }
    refresh_all();
}

/// Logs a drink, or undoes the last one when `kind` is None.
pub fn log_coffee(kind: Option<CoffeeKind>) {
    {
        let mut state = app_state::state().lock().unwrap();
        match kind {
            Some(kind) => state.log_coffee(kind),
            None => state.undo_coffee(),
        }
    }
    refresh_all();
}

pub fn toggle_task_by_index(index: usize) {
    let task_id = {
        let state = app_state::state().lock().unwrap();
        state.today.tasks.get(index).map(|t| t.id.clone())
    };

    if let Some(id) = task_id {
        {
            let mut state = app_state::state().lock().unwrap();
            state.toggle_task(&id);
        }
        refresh_all();
    }
}

pub fn complete_quick_break(break_type_idx: usize) {
    {
        let mut state = app_state::state().lock().unwrap();
        match break_type_idx {
            0 => state.increment_task("eye_breaks", 1),
            1 => state.increment_task("stand_stretch", 1),
            _ => {}
        }
        state.today.breaks_taken += 1;
        state.save();
    }
    refresh_all();
}

pub fn toggle_timer() {
    {
        let mut state = app_state::state().lock().unwrap();
        state.toggle_timer();
    }
    refresh_all();
}

pub fn reset_timer() {
    {
        let mut state = app_state::state().lock().unwrap();
        state.reset_timer();
    }
    refresh_all();
}

pub fn manual_take_break() {
    {
        let mut state = app_state::state().lock().unwrap();
        state.manual_take_break();
    }
    refresh_all();
}

pub fn set_timer_mode(mode_idx: usize) {
    let break_type = match mode_idx {
        0 => BreakType::EyeBreak20,
        1 => BreakType::StandStretch,
        2 => BreakType::DeepBreathing,
        3 => BreakType::PostureCheck,
        _ => BreakType::EyeBreak20,
    };
    {
        let mut state = app_state::state().lock().unwrap();
        state.select_break_type(break_type);
    }
    refresh_all();
}

pub fn switch_tab(tab_idx: usize) {
    CONTROLLER.with(|c| {
        if let Some(ctrl) = c.borrow().as_ref() {
            ctrl.main_window.switch_tab(tab_idx);
        }
    });
    {
        let mut state = app_state::state().lock().unwrap();
        state.settings.selected_tab = tab_idx;
        state.save();
    }
}

pub fn toggle_sound() {
    {
        let mut state = app_state::state().lock().unwrap();
        state.settings.sound_enabled = !state.settings.sound_enabled;
        state.save();
    }
    refresh_all();
}

pub fn set_work_interval_preset(selected: usize) {
    let minutes = match selected {
        0 => 30,
        1 => 45,
        2 => 60,
        3 => 90,
        _ => 60,
    };
    {
        let mut state = app_state::state().lock().unwrap();
        state.settings.work_interval_minutes = minutes;
        state.save();
    }
    refresh_all();
}

pub fn set_idle_interval_preset(selected: usize) {
    let minutes = match selected {
        0 => 3,
        1 => 5,
        2 => 10,
        3 => 15,
        _ => 5,
    };
    {
        let mut state = app_state::state().lock().unwrap();
        state.settings.idle_reset_minutes = minutes;
        state.save();
    }
    refresh_all();
}

pub fn toggle_banner_setting() {
    {
        let mut state = app_state::state().lock().unwrap();
        state.settings.show_banner_notification = !state.settings.show_banner_notification;
        state.save();
    }
    refresh_all();
}

pub fn toggle_alert_setting() {
    {
        let mut state = app_state::state().lock().unwrap();
        state.settings.show_alert_dialog = !state.settings.show_alert_dialog;
        state.save();
    }
    refresh_all();
}

pub fn save_settings_from_ui() {
    let mut title = String::new();
    let mut message = String::new();

    CONTROLLER.with(|c| {
        if let Some(ctrl) = c.borrow().as_ref() {
            title = ctrl.main_window.settings_view.title_field.stringValue().to_string();
            message = ctrl.main_window.settings_view.message_field.stringValue().to_string();
            ctrl.main_window.settings_view.status_feedback_label.setStringValue(
                &NSString::from_str("✅ Settings saved successfully!"),
            );
        }
    });

    if !title.is_empty() || !message.is_empty() {
        let mut state = app_state::state().lock().unwrap();
        if !title.is_empty() {
            state.settings.notice_title = title;
        }
        if !message.is_empty() {
            state.settings.notice_message = message;
        }
        state.save();
    }
}

pub fn test_notice() {
    let mut title = String::new();
    let mut message = String::new();
    let mut mtm_opt = None;

    CONTROLLER.with(|c| {
        if let Some(ctrl) = c.borrow().as_ref() {
            title = ctrl.main_window.settings_view.title_field.stringValue().to_string();
            message = ctrl.main_window.settings_view.message_field.stringValue().to_string();
            mtm_opt = Some(ctrl.mtm);
        }
    });

    let (sound, show_banner, show_alert) = {
        let state = app_state::state().lock().unwrap();
        if title.is_empty() {
            title = state.settings.notice_title.clone();
        }
        if message.is_empty() {
            message = state.settings.notice_message.clone();
        }
        (
            state.settings.sound_name.clone(),
            state.settings.show_banner_notification,
            state.settings.show_alert_dialog,
        )
    };

    if let Some(mtm) = mtm_opt {
        Notifier::send_notice(&title, &message, &sound, show_banner, show_alert, mtm);
    }
}

pub fn connect_reminders() {
    match reminders::access() {
        reminders::Access::Denied => open_url(
            "x-apple.systempreferences:com.apple.preference.security?Privacy_Reminders",
        ),
        _ => reminders::request_access(),
    }
}

pub fn open_reminders_app() {
    open_url("x-apple-reminderkit://");
}

pub fn complete_reminder(index: usize) {
    reminders::complete(index);
    update_reminders();
}

/// Runs a navigation/selection change on the Statistics month calendar, then redraws.
pub fn calendar(change: impl FnOnce(&crate::ui::calendar_card::CalendarCard)) {
    CONTROLLER.with(|c| {
        if let Some(ctrl) = c.borrow().as_ref() {
            change(&ctrl.main_window.statistics_view.calendar);
        }
    });
    refresh_all();
}

/// Same for the "This Week" strip on Today.
pub fn week(change: impl FnOnce(&crate::ui::calendar_card::CalendarCard)) {
    CONTROLLER.with(|c| {
        if let Some(ctrl) = c.borrow().as_ref() {
            change(&ctrl.main_window.dashboard_view.week);
        }
    });
    refresh_all();
}

pub fn edit_reminder(index: usize) {
    CONTROLLER.with(|c| {
        if let Some(ctrl) = c.borrow().as_ref() {
            ctrl.control_center.begin_edit(index);
        }
    });
}

/// Saves the title typed into the control center (Return or focus loss).
pub fn rename_reminder(title: String) {
    let index = CONTROLLER.with(|c| c.borrow().as_ref().and_then(|ctrl| ctrl.control_center.end_edit()));
    if let Some(index) = index {
        reminders::rename(index, &title);
        update_reminders();
    }
}

fn open_url(url: &str) {
    if let Some(url) = objc2_foundation::NSURL::URLWithString(&NSString::from_str(url)) {
        objc2_app_kit::NSWorkspace::sharedWorkspace().openURL(&url);
    }
}

/// Pulls fresh reminders data (if any) into the Today page.
fn update_reminders() {
    if let Some(snapshot) = reminders::poll() {
        if let reminders::Snapshot::Items(items) = &snapshot {
            println!("Reminders: {} open for today", items.len());
        }
        CONTROLLER.with(|c| {
            if let Some(ctrl) = c.borrow().as_ref() {
                ctrl.main_window.dashboard_view.reminders.update(&snapshot);
                ctrl.control_center.update_tasks(&snapshot);
            }
        });
    }
}

pub fn toggle_control_center_collapsed() {
    let collapsed = CONTROLLER.with(|c| {
        c.borrow().as_ref().map(|ctrl| {
            let cc = &ctrl.control_center;
            cc.set_collapsed(!cc.is_collapsed());
            cc.is_collapsed()
        })
    });
    if let Some(collapsed) = collapsed {
        let mut state = app_state::state().lock().unwrap();
        state.settings.control_center_collapsed = collapsed;
        state.save();
    }
}

pub fn set_launch_at_login(enabled: bool) {
    let error = crate::login_item::set_enabled(enabled).err();
    CONTROLLER.with(|c| {
        if let Some(ctrl) = c.borrow().as_ref() {
            ctrl.main_window.settings_view.update_login_item(error.as_deref());
        }
    });
}

pub fn set_window_delegate(delegate: &objc2::runtime::ProtocolObject<dyn objc2_app_kit::NSWindowDelegate>) {
    CONTROLLER.with(|c| {
        if let Some(ctrl) = c.borrow().as_ref() {
            ctrl.main_window.window.setDelegate(Some(delegate));
        }
    });
}

/// Points a main-menu item at the app's action handler.
pub fn set_menu_target(item: &objc2_app_kit::NSMenuItem) {
    CONTROLLER.with(|c| {
        if let Some(ctrl) = c.borrow().as_ref() {
            let target: &AnyObject = &ctrl.handler;
            unsafe { item.setTarget(Some(target)) };
        }
    });
}

pub fn toggle_control_center_visible() {
    let visible = CONTROLLER.with(|c| {
        c.borrow().as_ref().map(|ctrl| {
            let cc = &ctrl.control_center;
            cc.set_visible(!cc.is_visible());
            cc.is_visible()
        })
    });
    if let Some(visible) = visible {
        {
            let mut state = app_state::state().lock().unwrap();
            state.settings.control_center_visible = visible;
            state.save();
        }
        refresh_all();
    }
}

/// Saves the control center position after the user drags it.
fn persist_control_center_position() {
    let Some(top_left) = CONTROLLER.with(|c| c.borrow().as_ref().map(|ctrl| ctrl.control_center.top_left()))
    else {
        return;
    };
    let mut state = app_state::state().lock().unwrap();
    if state.settings.control_center_top_left != Some(top_left) {
        state.settings.control_center_top_left = Some(top_left);
        state.save();
    }
}

pub fn on_timer_tick() {
    update_reminders();
    persist_control_center_position();

    let idle_seconds = activity_tracker::get_system_idle_seconds();

    // 1. Check background keyboard and mouse activity
    let notice_due = {
        let mut state = app_state::state().lock().unwrap();
        state.tick_activity_monitor(idle_seconds)
    };

    // Don't interrupt a screen recording: hold notices and pop-ups until it ends.
    let recording = screen_recording::is_recording();
    if recording != WAS_RECORDING.replace(recording) {
        println!(
            "Screen recording {}",
            if recording { "started: holding break notices" } else { "ended" }
        );
        CONTROLLER.with(|c| {
            if let Some(ctrl) = c.borrow().as_ref() {
                ctrl.control_center.on_recording_changed(recording);
            }
        });
    }

    // If the work limit is reached, send the customizable notice (or hold it).
    if let Some(notice) = notice_due {
        if recording {
            HELD_NOTICE.with(|h| *h.borrow_mut() = Some(notice));
        } else {
            deliver_notice(notice);
        }
    }
    if !recording {
        if let Some(notice) = HELD_NOTICE.with(|h| h.borrow_mut().take()) {
            deliver_notice(notice);
        }
    }

    // 2. Check manual interval countdown timer
    let completed_break = {
        let mut state = app_state::state().lock().unwrap();
        state.notice_held = HELD_NOTICE.with(|h| h.borrow().is_some());
        state.tick_timer(recording)
    };

    if completed_break.is_some() {
        HELD_DASHBOARD.set(true);
    }
    if !recording && HELD_DASHBOARD.replace(false) {
        show_dashboard();
    }

    refresh_all();
}

fn deliver_notice((title, message, sound, show_banner, show_alert): Notice) {
    let mut mtm_opt = None;
    CONTROLLER.with(|c| {
        if let Some(ctrl) = c.borrow().as_ref() {
            mtm_opt = Some(ctrl.mtm);
        }
    });

    if let Some(mtm) = mtm_opt {
        Notifier::send_notice(&title, &message, &sound, show_banner, show_alert, mtm);
    }
}

pub fn refresh_all() {
    let state = app_state::state().lock().unwrap();
    CONTROLLER.with(|c| {
        if let Some(ctrl) = c.borrow().as_ref() {
            ctrl.main_window.refresh(&state, ctrl.mtm);
            ctrl.control_center.refresh(&state);
            ctrl.menu_bar.refresh(&state, ctrl.mtm);
        }
    });
}
