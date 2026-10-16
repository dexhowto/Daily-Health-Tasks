use objc2::rc::Retained;
use objc2::{define_class, MainThreadOnly};
use objc2_app_kit::{NSButton, NSMenuItem, NSSegmentedControl};
use objc2_foundation::{MainThreadMarker, NSObject};

use crate::controller;
use crate::models::{CoffeeKind, ExerciseKind};

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "HealthTasksActionHandler"]
    pub struct HealthTasksActionHandler;

    impl HealthTasksActionHandler {
        #[unsafe(method(onShowDashboard:))]
        fn on_show_dashboard(&self, _sender: &NSObject) {
            controller::show_dashboard();
        }

        #[unsafe(method(onQuitApp:))]
        fn on_quit_app(&self, _sender: &NSObject) {
            controller::quit_app();
        }

        #[unsafe(method(onAddWater250:))]
        fn on_add_water_250(&self, _sender: &NSObject) {
            controller::add_water(250);
        }

        #[unsafe(method(onAddWater500:))]
        fn on_add_water_500(&self, _sender: &NSObject) {
            controller::add_water(500);
        }

        #[unsafe(method(onMinusWater250:))]
        fn on_minus_water_250(&self, _sender: &NSObject) {
            controller::add_water(-250);
        }

        #[unsafe(method(onAddEspresso:))]
        fn on_add_espresso(&self, _sender: &NSObject) {
            controller::log_coffee(Some(CoffeeKind::Espresso));
        }

        #[unsafe(method(onAddCoffee:))]
        fn on_add_coffee(&self, _sender: &NSObject) {
            controller::log_coffee(Some(CoffeeKind::Coffee));
        }

        #[unsafe(method(onAddLatte:))]
        fn on_add_latte(&self, _sender: &NSObject) {
            controller::log_coffee(Some(CoffeeKind::Latte));
        }

        #[unsafe(method(onLogPlank:))]
        fn on_log_plank(&self, _sender: &NSObject) {
            controller::log_exercise(ExerciseKind::Plank);
        }

        #[unsafe(method(onLogStretching:))]
        fn on_log_stretching(&self, _sender: &NSObject) {
            controller::log_exercise(ExerciseKind::Stretching);
        }

        #[unsafe(method(onLogWalk:))]
        fn on_log_walk(&self, _sender: &NSObject) {
            controller::log_exercise(ExerciseKind::Walk);
        }

        #[unsafe(method(onUndoCoffee:))]
        fn on_undo_coffee(&self, _sender: &NSObject) {
            controller::log_coffee(None);
        }

        #[unsafe(method(onConnectReminders:))]
        fn on_connect_reminders(&self, _sender: &NSObject) {
            controller::connect_reminders();
        }

        #[unsafe(method(onOpenReminders:))]
        fn on_open_reminders(&self, _sender: &NSObject) {
            controller::open_reminders_app();
        }

        #[unsafe(method(onToggleReminder:))]
        fn on_toggle_reminder(&self, sender: &NSButton) {
            controller::complete_reminder(sender.tag() as usize);
        }

        #[unsafe(method(onToggleControlCenter:))]
        fn on_toggle_control_center(&self, _sender: &NSObject) {
            controller::toggle_control_center_visible();
        }

        #[unsafe(method(onToggleControlCenterCollapsed:))]
        fn on_toggle_control_center_collapsed(&self, _sender: &NSObject) {
            controller::toggle_control_center_collapsed();
        }

        #[unsafe(method(onEditReminder:))]
        fn on_edit_reminder(&self, sender: &NSButton) {
            controller::edit_reminder(sender.tag() as usize);
        }

        #[unsafe(method(onRenameReminder:))]
        fn on_rename_reminder(&self, sender: &objc2_app_kit::NSTextField) {
            controller::rename_reminder(sender.stringValue().to_string());
        }

        #[unsafe(method(onCalendarPrev:))]
        fn on_calendar_prev(&self, _sender: &NSObject) {
            controller::calendar(|c| c.show_previous_month());
        }

        #[unsafe(method(onCalendarNext:))]
        fn on_calendar_next(&self, _sender: &NSObject) {
            controller::calendar(|c| c.show_next_month());
        }

        #[unsafe(method(onCalendarToday:))]
        fn on_calendar_today(&self, _sender: &NSObject) {
            controller::calendar(|c| c.show_today());
        }

        #[unsafe(method(onCalendarMonthPicked:))]
        fn on_calendar_month_picked(&self, sender: &objc2_app_kit::NSPopUpButton) {
            let index = sender.indexOfSelectedItem();
            controller::calendar(|c| c.show_picked_month(index));
        }

        #[unsafe(method(onSelectWeekDay:))]
        fn on_select_week_day(&self, sender: &NSButton) {
            let index = sender.tag() as usize;
            controller::week(|c| c.select_cell(index));
        }

        #[unsafe(method(onSelectCalendarDay:))]
        fn on_select_calendar_day(&self, sender: &NSButton) {
            let index = sender.tag() as usize;
            controller::calendar(|c| c.select_cell(index));
        }

        #[unsafe(method(onToggleLaunchAtLogin:))]
        fn on_toggle_launch_at_login(&self, sender: &objc2_app_kit::NSSwitch) {
            controller::set_launch_at_login(sender.state() == objc2_app_kit::NSControlStateValueOn);
        }

        #[unsafe(method(onSelectGuide:))]
        fn on_select_guide(&self, sender: &NSButton) {
            controller::select_guide(sender.tag() as usize);
        }

        #[unsafe(method(onShowRoutineGuide:))]
        fn on_show_routine_guide(&self, sender: &NSButton) {
            controller::show_routine_guide(sender.tag() as usize);
        }

        #[unsafe(method(onToggleTask:))]
        fn on_toggle_task(&self, sender: &NSButton) {
            let tag = sender.tag() as usize;
            controller::toggle_task_by_index(tag);
        }

        #[unsafe(method(onQuickBreakDone:))]
        fn on_quick_break_done(&self, sender: &NSMenuItem) {
            let tag = sender.tag() as usize;
            controller::complete_quick_break(tag);
        }

        #[unsafe(method(onTimerToggle:))]
        fn on_timer_toggle(&self, _sender: &NSObject) {
            controller::toggle_timer();
        }

        #[unsafe(method(onTimerReset:))]
        fn on_timer_reset(&self, _sender: &NSObject) {
            controller::reset_timer();
        }

        #[unsafe(method(onManualTakeBreak:))]
        fn on_manual_take_break(&self, _sender: &NSObject) {
            controller::manual_take_break();
        }

        #[unsafe(method(onSelectBreakMode:))]
        fn on_select_break_mode(&self, sender: &NSSegmentedControl) {
            let selected = sender.selectedSegment() as usize;
            controller::set_timer_mode(selected);
        }

        #[unsafe(method(onTabSwitch:))]
        fn on_tab_switch(&self, sender: &NSButton) {
            controller::switch_tab(sender.tag() as usize);
        }

        #[unsafe(method(onToggleSound:))]
        fn on_toggle_sound(&self, _sender: &NSObject) {
            controller::toggle_sound();
        }

        #[unsafe(method(onWorkIntervalChanged:))]
        fn on_work_interval_changed(&self, sender: &NSSegmentedControl) {
            let selected = sender.selectedSegment() as usize;
            controller::set_work_interval_preset(selected);
        }

        #[unsafe(method(onIdleIntervalChanged:))]
        fn on_idle_interval_changed(&self, sender: &NSSegmentedControl) {
            let selected = sender.selectedSegment() as usize;
            controller::set_idle_interval_preset(selected);
        }

        #[unsafe(method(onToggleBannerSetting:))]
        fn on_toggle_banner_setting(&self, _sender: &NSObject) {
            controller::toggle_banner_setting();
        }

        #[unsafe(method(onToggleAlertSetting:))]
        fn on_toggle_alert_setting(&self, _sender: &NSObject) {
            controller::toggle_alert_setting();
        }

        #[unsafe(method(onNoticeTakeBreak:))]
        fn on_notice_take_break(&self, _sender: &NSObject) {
            controller::notice_take_break();
        }

        #[unsafe(method(onNoticeDismiss:))]
        fn on_notice_dismiss(&self, _sender: &NSObject) {
            controller::notice_dismiss();
        }

        #[unsafe(method(onBreakBackToWork:))]
        fn on_break_back_to_work(&self, _sender: &NSObject) {
            controller::answer_break_question(true);
        }

        #[unsafe(method(onBreakKeepResting:))]
        fn on_break_keep_resting(&self, _sender: &NSObject) {
            controller::answer_break_question(false);
        }

        #[unsafe(method(onTestNotice:))]
        fn on_test_notice(&self, _sender: &NSObject) {
            controller::test_notice();
        }

        #[unsafe(method(onSaveSettings:))]
        fn on_save_settings(&self, _sender: &NSObject) {
            controller::save_settings_from_ui();
        }

        #[unsafe(method(onTimerTick:))]
        fn on_timer_tick(&self, _sender: &NSObject) {
            controller::on_timer_tick();
        }
    }
);

impl HealthTasksActionHandler {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { objc2::msg_send![mtm.alloc::<Self>(), init] }
    }
}
