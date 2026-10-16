use std::cell::RefCell;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::sel;
use objc2_app_kit::NSApplication;
use objc2_foundation::{MainThreadMarker, NSString, NSTimer};

use crate::action_handler::HealthTasksActionHandler;
use crate::activity_tracker;
use crate::app_state;
use crate::menu_bar::MenuBarManager;
use crate::models::{BreakType, CoffeeKind};
use crate::notifier::Notifier;
use crate::ui::MainWindow;

#[allow(dead_code)]
pub struct AppController {
    pub handler: Retained<HealthTasksActionHandler>,
    pub main_window: MainWindow,
    pub menu_bar: MenuBarManager,
    pub timer: Retained<NSTimer>,
    pub mtm: MainThreadMarker,
}

thread_local! {
    static CONTROLLER: RefCell<Option<AppController>> = const { RefCell::new(None) };
}

pub fn init_controller(mtm: MainThreadMarker) {
    let handler = HealthTasksActionHandler::new(mtm);
    let state = app_state::state().lock().unwrap();

    let main_window = MainWindow::build(&handler, &state, mtm);
    let menu_bar = MenuBarManager::build(&handler, &state, mtm);

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

pub fn on_timer_tick() {
    let idle_seconds = activity_tracker::get_system_idle_seconds();

    // 1. Check background keyboard and mouse activity
    let notice_due = {
        let mut state = app_state::state().lock().unwrap();
        state.tick_activity_monitor(idle_seconds)
    };

    // If 60 minutes of activity reached, send the customizable notice!
    if let Some((title, message, sound, show_banner, show_alert)) = notice_due {
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

    // 2. Check manual interval countdown timer
    let completed_break = {
        let mut state = app_state::state().lock().unwrap();
        state.tick_timer()
    };

    if completed_break.is_some() {
        show_dashboard();
    }

    refresh_all();
}

pub fn refresh_all() {
    let state = app_state::state().lock().unwrap();
    CONTROLLER.with(|c| {
        if let Some(ctrl) = c.borrow().as_ref() {
            ctrl.main_window.refresh(&state, ctrl.mtm);
            ctrl.menu_bar.refresh(&state, ctrl.mtm);
        }
    });
}
