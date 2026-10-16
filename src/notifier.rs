use std::process::Command;
use objc2_app_kit::{NSAlert, NSAlertFirstButtonReturn};
use objc2_foundation::{MainThreadMarker, NSString};

use crate::audio::SoundEffects;

pub struct Notifier;

impl Notifier {
    /// Returns true when the user picks "Take a Break" in the alert.
    pub fn send_notice(
        title: &str,
        message: &str,
        sound_name: &str,
        show_banner: bool,
        show_alert: bool,
        mtm: MainThreadMarker,
    ) -> bool {
        // 1. Play sound
        if !sound_name.is_empty() && sound_name != "Mute" {
            SoundEffects::play_named(sound_name);
        }

        // 2. Deliver macOS Notification Center banner via osascript
        if show_banner {
            let t = title.replace('"', "\\\"");
            let m = message.replace('"', "\\\"");
            let s = sound_name.replace('"', "\\\"");
            let script = format!(
                "display notification \"{}\" with title \"{}\" sound name \"{}\"",
                m, t, s
            );
            std::thread::spawn(move || {
                let _ = Command::new("osascript")
                    .arg("-e")
                    .arg(&script)
                    .output();
            });
        }

        // 3. Show native NSAlert modal dialog on screen if enabled
        if show_alert {
            let alert = NSAlert::new(mtm);
            alert.setMessageText(&NSString::from_str(title));
            alert.setInformativeText(&NSString::from_str(message));
            alert.addButtonWithTitle(&NSString::from_str("Take a Break"));
            alert.addButtonWithTitle(&NSString::from_str("Dismiss"));
            return alert.runModal() == NSAlertFirstButtonReturn;
        }
        false
    }

    /// Asked when the keyboard or mouse is used during a break started with
    /// "Take a Break". Returns true for "Back to Work".
    pub fn ask_end_break(mtm: MainThreadMarker) -> bool {
        let alert = NSAlert::new(mtm);
        alert.setMessageText(&NSString::from_str("You're on a break"));
        alert.setInformativeText(&NSString::from_str(
            "You used the keyboard or mouse. End the break and go back to work, or keep resting?",
        ));
        alert.addButtonWithTitle(&NSString::from_str("Back to Work"));
        alert.addButtonWithTitle(&NSString::from_str("Keep Resting"));
        alert.runModal() == NSAlertFirstButtonReturn
    }
}
