#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGEventSourceSecondsSinceLastEventType(source_state: i32, event_type: u32) -> f64;
}

/// Returns the number of seconds since the last keyboard or mouse event across macOS.
/// `source_state = 1` corresponds to `kCGEventSourceStateCombinedSessionState`.
/// `event_type = !0` corresponds to `kCGAnyInputEventType`.
pub fn get_system_idle_seconds() -> f64 {
    unsafe { CGEventSourceSecondsSinceLastEventType(1, !0) }
}
