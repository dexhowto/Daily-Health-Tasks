use std::ffi::{c_char, c_int, c_void, CStr};

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_foundation::{NSArray, NSDictionary, NSString};

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGWindowListCopyWindowInfo(option: u32, relative_to_window: u32) -> *mut c_void;
}

const K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY: u32 = 1 << 0;

// libproc (part of libSystem, always linked on macOS).
unsafe extern "C" {
    fn proc_listallpids(buffer: *mut c_void, buffersize: c_int) -> c_int;
    fn proc_name(pid: c_int, buffer: *mut c_void, buffersize: u32) -> c_int;
}

/// Processes that run only while macOS's built-in capture is active.
/// Cmd+Shift+5 and QuickTime screen recording both go through `screencaptureui`
/// (plus `screencapture` while the toolbar is open); both exit when the
/// recording stops.
const CAPTURE_PROCESSES: [&str; 2] = ["screencaptureui", "screencapture"];

/// True while the screen is being captured: macOS shows its privacy indicator,
/// or the built-in screenshot/recording toolbar is open.
pub fn is_recording() -> bool {
    privacy_indicator_visible()
        || running_process_names()
            .iter()
            .any(|name| CAPTURE_PROCESSES.contains(&name.as_str()))
}

/// The menu bar privacy indicator is a Window Server window named
/// "StatusIndicator". It is on screen while any app records the screen
/// (it also appears for microphone or camera use, e.g. during calls).
fn privacy_indicator_visible() -> bool {
    let raw = unsafe { CGWindowListCopyWindowInfo(K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY, 0) };
    if raw.is_null() {
        return false;
    }
    // CFArray of CFDictionary is toll-free bridged; the copy is owned (+1).
    let windows: Retained<NSArray<NSDictionary<NSString, AnyObject>>> =
        unsafe { Retained::from_raw(raw.cast()) }.expect("non-null window list");

    let owner_key = NSString::from_str("kCGWindowOwnerName");
    let name_key = NSString::from_str("kCGWindowName");
    let string_for = |dict: &NSDictionary<NSString, AnyObject>, key: &NSString| {
        dict.objectForKey(key)
            .and_then(|v| v.downcast::<NSString>().ok())
            .map(|s| s.to_string())
    };
    windows.iter().any(|w| {
        string_for(&w, &owner_key).as_deref() == Some("Window Server")
            && string_for(&w, &name_key).as_deref() == Some("StatusIndicator")
    })
}

fn running_process_names() -> Vec<String> {
    unsafe {
        let count = proc_listallpids(std::ptr::null_mut(), 0);
        if count <= 0 {
            return Vec::new();
        }
        // Leave headroom for processes started between the two calls.
        let mut pids = vec![0 as c_int; count as usize + 64];
        let bytes = (pids.len() * std::mem::size_of::<c_int>()) as c_int;
        let n = proc_listallpids(pids.as_mut_ptr().cast(), bytes);
        if n <= 0 {
            return Vec::new();
        }

        let mut buf = [0 as c_char; 256];
        pids[..(n as usize).min(pids.len())]
            .iter()
            .filter_map(|&pid| {
                let len = proc_name(pid, buf.as_mut_ptr().cast(), buf.len() as u32);
                (len > 0).then(|| CStr::from_ptr(buf.as_ptr()).to_string_lossy().into_owned())
            })
            .collect()
    }
}
