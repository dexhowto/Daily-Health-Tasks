use std::cell::OnceCell;
use std::process::Command;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::{Bool, NSObject, NSObjectProtocol, ProtocolObject};
use objc2::{define_class, msg_send, AllocAnyThread};
use objc2_foundation::{NSBundle, NSError, NSString};
use objc2_user_notifications::{
    UNAuthorizationOptions, UNMutableNotificationContent, UNNotification,
    UNNotificationPresentationOptions, UNNotificationRequest, UNUserNotificationCenter,
    UNUserNotificationCenterDelegate,
};

use crate::audio::SoundEffects;

thread_local! {
    /// The notification center keeps only a weak reference to its delegate.
    static BANNER_DELEGATE: OnceCell<Retained<BannerDelegate>> = const { OnceCell::new() };
}

pub struct Notifier;

impl Notifier {
    /// Plays the notice sound and posts the Notification Center banner. The
    /// on-screen panel is shown by the controller.
    pub fn send_notice(title: &str, message: &str, sound_name: &str, show_banner: bool) {
        if !sound_name.is_empty() && sound_name != "Mute" {
            SoundEffects::play_named(sound_name);
        }
        if show_banner {
            post_banner(title, message);
        }
    }

    /// Sets up the banner delegate and asks for notification access (macOS
    /// asks only once). Call at launch, on the main thread.
    pub fn setup() {
        if !is_app_bundle() {
            return;
        }
        let center = UNUserNotificationCenter::currentNotificationCenter();
        BANNER_DELEGATE.with(|cell| {
            let delegate = cell.get_or_init(BannerDelegate::new);
            center.setDelegate(Some(ProtocolObject::from_ref(&**delegate)));
        });
        let done = RcBlock::new(|granted: Bool, _error: *mut NSError| {
            if !granted.as_bool() {
                println!("Break banners are off for this app in System Settings > Notifications");
            }
        });
        center.requestAuthorizationWithOptions_completionHandler(
            UNAuthorizationOptions::Alert | UNAuthorizationOptions::Sound,
            &done,
        );
    }
}

/// The banner comes from the app itself: its name and icon, and its own
/// switch in System Settings > Notifications. Outside the .app bundle
/// (`cargo run`) UserNotifications isn't available, so fall back to osascript.
fn post_banner(title: &str, message: &str) {
    if is_app_bundle() {
        let content = UNMutableNotificationContent::new();
        content.setTitle(&NSString::from_str(title));
        content.setBody(&NSString::from_str(message));
        // No sound here: the notice sound is already played.
        let request = UNNotificationRequest::requestWithIdentifier_content_trigger(
            &NSString::from_str("break-notice"),
            &content,
            None,
        );
        let done = RcBlock::new(|error: *mut NSError| {
            if let Some(error) = unsafe { error.as_ref() } {
                eprintln!("Warning: could not post the break banner: {error:?}");
            }
        });
        UNUserNotificationCenter::currentNotificationCenter()
            .addNotificationRequest_withCompletionHandler(&request, Some(&done));
        return;
    }

    let t = title.replace('"', "\\\"");
    let m = message.replace('"', "\\\"");
    let script = format!("display notification \"{}\" with title \"{}\"", m, t);
    std::thread::spawn(move || {
        let _ = Command::new("osascript").arg("-e").arg(&script).output();
    });
}

fn is_app_bundle() -> bool {
    NSBundle::mainBundle().bundlePath().to_string().ends_with(".app")
}

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "HealthTasksBannerDelegate"]
    struct BannerDelegate;

    unsafe impl NSObjectProtocol for BannerDelegate {}

    unsafe impl UNUserNotificationCenterDelegate for BannerDelegate {
        /// Show the banner even while the app is in front (macOS hides it by default).
        #[unsafe(method(userNotificationCenter:willPresentNotification:withCompletionHandler:))]
        fn will_present(
            &self,
            _center: &UNUserNotificationCenter,
            _notification: &UNNotification,
            completion_handler: &block2::DynBlock<dyn Fn(UNNotificationPresentationOptions)>,
        ) {
            completion_handler.call((
                UNNotificationPresentationOptions::Banner | UNNotificationPresentationOptions::List,
            ));
        }
    }
);

impl BannerDelegate {
    fn new() -> Retained<Self> {
        unsafe { msg_send![Self::alloc(), init] }
    }
}
