mod action_handler;
mod activity_tracker;
mod app_state;
mod audio;
mod controller;
mod menu_bar;
mod models;
mod notifier;
mod reminders;
mod screen_recording;
mod storage;
mod ui;

use objc2::sel;
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSMenu, NSMenuItem,
};
use objc2_foundation::{MainThreadMarker, NSString};

/// User-facing app name. The on-disk data folder keeps its original name.
pub const APP_NAME: &str = "DexHowTo HealthTask";

fn main() {
    let mtm = MainThreadMarker::new().expect("DexHowTo HealthTask must run on the macOS main thread");
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Regular);
    if let Some(icon) = ui::components::UIComponents::app_icon(mtm) {
        unsafe { app.setApplicationIconImage(Some(&icon)) };
    }

    // Setup standard macOS Application menu bar (Quit, Window, etc.)
    setup_main_menu(app.as_ref(), mtm);

    // Initialize UI Controller (MainWindow + MenuBar status item + Timer)
    controller::init_controller(mtm);
    reminders::start();

    // Show desktop window on launch
    controller::show_dashboard();

    println!("{APP_NAME} started successfully! Running native macOS event loop...");
    app.run();
}

fn setup_main_menu(app: &NSApplication, mtm: MainThreadMarker) {
    let main_menu = NSMenu::initWithTitle(mtm.alloc(), &NSString::from_str("MainMenu"));

    // 1. Application Menu
    let app_menu_item = NSMenuItem::new(mtm);
    let app_menu = NSMenu::initWithTitle(mtm.alloc(), &NSString::from_str(APP_NAME));

    let hide_title = NSString::from_str(&format!("Hide {APP_NAME}"));
    let hide_key = NSString::from_str("h");
    let hide_item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            mtm.alloc(),
            &hide_title,
            Some(sel!(hide:)),
            &hide_key,
        )
    };

    let hide_others_title = NSString::from_str("Hide Others");
    let hide_others_key = NSString::from_str("h");
    let hide_others_item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            mtm.alloc(),
            &hide_others_title,
            Some(sel!(hideOtherApplications:)),
            &hide_others_key,
        )
    };

    let quit_title = NSString::from_str(&format!("Quit {APP_NAME}"));
    let quit_key = NSString::from_str("q");
    let quit_item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            mtm.alloc(),
            &quit_title,
            Some(sel!(terminate:)),
            &quit_key,
        )
    };

    app_menu.addItem(&hide_item);
    app_menu.addItem(&hide_others_item);
    app_menu.addItem(&NSMenuItem::separatorItem(mtm));
    app_menu.addItem(&quit_item);
    app_menu_item.setSubmenu(Some(&app_menu));
    main_menu.addItem(&app_menu_item);

    // 2. Window Menu
    let window_menu_item = NSMenuItem::new(mtm);
    let window_menu = NSMenu::initWithTitle(mtm.alloc(), &NSString::from_str("Window"));

    let minimize_title = NSString::from_str("Minimize");
    let minimize_key = NSString::from_str("m");
    let minimize_item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            mtm.alloc(),
            &minimize_title,
            Some(sel!(performMiniaturize:)),
            &minimize_key,
        )
    };

    let close_title = NSString::from_str("Close Window");
    let close_key = NSString::from_str("w");
    let close_item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            mtm.alloc(),
            &close_title,
            Some(sel!(performClose:)),
            &close_key,
        )
    };

    window_menu.addItem(&minimize_item);
    window_menu.addItem(&close_item);
    window_menu_item.setSubmenu(Some(&window_menu));
    main_menu.addItem(&window_menu_item);

    app.setMainMenu(Some(&main_menu));
}
