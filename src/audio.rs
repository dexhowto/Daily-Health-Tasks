use objc2_app_kit::NSSound;
use objc2_foundation::NSString;

pub struct SoundEffects;

impl SoundEffects {
    pub fn play_named(name: &str) {
        let ns_name = NSString::from_str(name);
        if let Some(sound) = NSSound::soundNamed(&ns_name) {
            sound.play();
        }
    }

    pub fn play_task_complete() {
        Self::play_named("Hero");
    }

    pub fn play_water_logged() {
        Self::play_named("Tink");
    }

    pub fn play_timer_finish() {
        Self::play_named("Glass");
    }

    pub fn play_button_click() {
        Self::play_named("Pop");
    }
}
