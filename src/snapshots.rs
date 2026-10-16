//! Debug builds only: `HT_SNAPSHOTS=<dir> cargo run` renders every page of the
//! main window, the control center and the break notice to PNG files in
//! <dir>, then quits (add HT_LIGHT=1 for light mode). Used to review the design without clicking through.

use block2::RcBlock;
use objc2_app_kit::{NSApplication, NSBitmapImageFileType, NSView};
use objc2_foundation::{MainThreadMarker, NSDictionary, NSString, NSTimer};

use crate::controller;

pub fn schedule(dir: String, mtm: MainThreadMarker) {
    let block = RcBlock::new(move |_timer: std::ptr::NonNull<NSTimer>| {
        controller::render_snapshots(&dir);
        NSApplication::sharedApplication(mtm).terminate(None);
    });
    // After a few ticks, so the reminders and stats are loaded.
    unsafe { NSTimer::scheduledTimerWithTimeInterval_repeats_block(2.5, false, &block) };
}

/// Writes `view` as a PNG at 2x.
pub fn write_png(view: &NSView, path: &str) {
    view.layoutSubtreeIfNeeded();
    let bounds = view.bounds();
    let Some(rep) = view.bitmapImageRepForCachingDisplayInRect(bounds) else { return };
    view.cacheDisplayInRect_toBitmapImageRep(bounds, &rep);
    let data = unsafe { rep.representationUsingType_properties(NSBitmapImageFileType::PNG, &NSDictionary::new()) };
    if let Some(data) = data {
        data.writeToFile_atomically(&NSString::from_str(path), true);
    }
}
