//! Application and main-window delegate: decides whether to show the
//! dashboard at launch, reopens it from the Dock, and saves state when the
//! window closes or the app quits.

use objc2::rc::Retained;
use objc2::runtime::{NSObject, NSObjectProtocol};
use objc2::{define_class, msg_send, MainThreadOnly};
use objc2_app_kit::{NSApplication, NSApplicationDelegate, NSWindowDelegate};
use objc2_foundation::{MainThreadMarker, NSAppleEventDescriptor, NSAppleEventManager, NSNotification};

use crate::{app_state, controller};

const fn four_cc(code: &[u8; 4]) -> u32 {
    u32::from_be_bytes(*code)
}

/// True when macOS started the app as a login item ("Open at login").
fn launched_at_login() -> bool {
    let Some(event) = NSAppleEventManager::sharedAppleEventManager().currentAppleEvent() else {
        return false;
    };
    // Plain messages: the typed bindings for these need the CoreServices crate.
    let event_id: u32 = unsafe { msg_send![&*event, eventID] };
    let prop: Option<Retained<NSAppleEventDescriptor>> =
        unsafe { msg_send![&*event, paramDescriptorForKeyword: four_cc(b"prdt")] };
    event_id == four_cc(b"oapp") && prop.is_some_and(|d| d.enumCodeValue() == four_cc(b"lgit"))
}

fn save_state() {
    app_state::state().lock().unwrap().save();
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "HealthTasksAppDelegate"]
    pub struct AppDelegate;

    unsafe impl NSObjectProtocol for AppDelegate {}

    unsafe impl NSApplicationDelegate for AppDelegate {
        #[unsafe(method(applicationDidFinishLaunching:))]
        fn application_did_finish_launching(&self, _notification: &NSNotification) {
            // At login, start quietly: menu bar + Control Center only.
            if launched_at_login() {
                println!("Launched at login: dashboard stays hidden");
            } else {
                controller::show_dashboard();
            }
        }

        /// Clicking the Dock icon brings the dashboard back.
        #[unsafe(method(applicationShouldHandleReopen:hasVisibleWindows:))]
        fn application_should_handle_reopen(&self, _app: &NSApplication, _has_visible_windows: bool) -> bool {
            controller::show_dashboard();
            true
        }

        #[unsafe(method(applicationWillTerminate:))]
        fn application_will_terminate(&self, _notification: &NSNotification) {
            save_state();
        }
    }

    unsafe impl NSWindowDelegate for AppDelegate {
        /// The close button only hides the window; save the running session anyway.
        #[unsafe(method(windowWillClose:))]
        fn window_will_close(&self, _notification: &NSNotification) {
            save_state();
        }
    }
);

impl AppDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), init] }
    }
}
