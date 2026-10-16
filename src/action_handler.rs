use objc2::rc::Retained;
use objc2::{define_class, MainThreadOnly};
use objc2_app_kit::{NSButton, NSMenuItem, NSSegmentedControl};
use objc2_foundation::{MainThreadMarker, NSObject};

use crate::controller;
use crate::models::CoffeeKind;

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

        #[unsafe(method(onUndoCoffee:))]
        fn on_undo_coffee(&self, _sender: &NSObject) {
            controller::log_coffee(None);
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
