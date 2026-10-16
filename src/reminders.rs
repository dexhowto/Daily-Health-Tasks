//! Apple Reminders integration (EventKit): open reminders due today or overdue
//! (never later or undated ones),
//! which can be completed from the app.
//!
//! EventKit calls back on background threads, so callbacks only record plain
//! data and flags here; the main-thread timer calls [`poll`] to pick them up.

use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration as StdDuration, Instant};

use block2::RcBlock;
use chrono::{Datelike, Duration, Local, NaiveDate, TimeZone};
use objc2::rc::Retained;
use objc2::runtime::{Bool, ProtocolObject};
use objc2_event_kit::{
    EKAuthorizationStatus, EKEntityType, EKEventStore, EKEventStoreChangedNotification, EKReminder,
};
use objc2_foundation::{
    NSArray, NSDate, NSDateComponentUndefined, NSDateComponents, NSError, NSNotificationCenter,
    NSObjectProtocol,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    NotRequested,
    Denied,
    Granted,
}

#[derive(Debug, Clone)]
pub struct ReminderItem {
    pub id: String,
    pub title: String,
    pub list: String,
    pub due_date: Option<NaiveDate>,
    /// "HH:MM" when the reminder has a due time, None for all-day.
    pub due_time: Option<String>,
}

impl ReminderItem {
    pub fn is_overdue(&self) -> bool {
        self.due_date.is_some_and(|d| d < Local::now().date_naive())
    }
}

#[derive(Debug, Clone)]
pub enum Snapshot {
    Access(Access),
    Loading,
    Items(Vec<ReminderItem>),
}

/// Set from any thread when data must be (re)fetched on the main thread.
static NEEDS_FETCH: AtomicBool = AtomicBool::new(false);
/// Set when access state or results changed and the UI should redraw.
static DIRTY: AtomicBool = AtomicBool::new(true);
static FETCH: Mutex<FetchState> = Mutex::new(FetchState::new());
static LAST_FETCH: Mutex<Option<Instant>> = Mutex::new(None);
static LAST_ACCESS: Mutex<Option<Access>> = Mutex::new(None);
/// Background refresh interval, so the list follows the clock (day rollover, overdue).
const REFRESH_EVERY: StdDuration = StdDuration::from_secs(60);

struct FetchState {
    in_flight: bool,
    items: Vec<ReminderItem>,
    loaded: bool,
}

impl FetchState {
    const fn new() -> Self {
        Self { in_flight: false, items: Vec::new(), loaded: false }
    }
}

thread_local! {
    static STORE: Retained<EKEventStore> = unsafe { EKEventStore::new() };
    static OBSERVER: RefCell<Option<Retained<ProtocolObject<dyn NSObjectProtocol>>>> =
        const { RefCell::new(None) };
}

pub fn access() -> Access {
    match unsafe { EKEventStore::authorizationStatusForEntityType(EKEntityType::Reminder) } {
        EKAuthorizationStatus::FullAccess => Access::Granted,
        EKAuthorizationStatus::NotDetermined => Access::NotRequested,
        _ => Access::Denied,
    }
}

/// Starts listening for Reminders changes and fetches if access was granted before.
pub fn start() {
    OBSERVER.with(|obs| {
        let block = RcBlock::new(|_| {
            NEEDS_FETCH.store(true, Ordering::SeqCst);
        });
        let token = unsafe {
            NSNotificationCenter::defaultCenter().addObserverForName_object_queue_usingBlock(
                Some(EKEventStoreChangedNotification),
                None,
                None,
                &block,
            )
        };
        *obs.borrow_mut() = Some(token);
    });
    if access() == Access::Granted {
        NEEDS_FETCH.store(true, Ordering::SeqCst);
    }
}

/// Shows the system permission prompt (first time only).
pub fn request_access() {
    let block = RcBlock::new(|granted: Bool, _err: *mut NSError| {
        if granted.as_bool() {
            NEEDS_FETCH.store(true, Ordering::SeqCst);
        }
        DIRTY.store(true, Ordering::SeqCst);
    });
    STORE.with(|store| unsafe {
        store.requestFullAccessToRemindersWithCompletion(RcBlock::as_ptr(&block));
    });
}

pub fn refresh() {
    NEEDS_FETCH.store(true, Ordering::SeqCst);
}

/// Main-thread tick: starts pending fetches and returns a new snapshot when the
/// UI needs to redraw.
pub fn poll() -> Option<Snapshot> {
    let access = access();
    if LAST_ACCESS.lock().unwrap().replace(access) != Some(access) {
        // Covers access changed in System Settings while the app runs.
        DIRTY.store(true, Ordering::SeqCst);
    }
    let stale = LAST_FETCH.lock().unwrap().is_none_or(|t| t.elapsed() >= REFRESH_EVERY);
    if access == Access::Granted && (NEEDS_FETCH.swap(false, Ordering::SeqCst) || stale) {
        *LAST_FETCH.lock().unwrap() = Some(Instant::now());
        start_fetch();
    }
    if !DIRTY.swap(false, Ordering::SeqCst) {
        return None;
    }
    if access != Access::Granted {
        return Some(Snapshot::Access(access));
    }
    let state = FETCH.lock().unwrap();
    Some(if state.loaded { Snapshot::Items(state.items.clone()) } else { Snapshot::Loading })
}

/// Completes the reminder at `index` of the last snapshot in the Reminders app.
pub fn complete(index: usize) {
    let Some(id) = FETCH.lock().unwrap().items.get(index).map(|r| r.id.clone()) else {
        return;
    };
    STORE.with(|store| unsafe {
        let Some(item) = store.calendarItemWithIdentifier(&objc2_foundation::NSString::from_str(&id))
        else {
            return;
        };
        let Ok(reminder) = item.downcast::<EKReminder>() else {
            return;
        };
        reminder.setCompleted(true);
        if let Err(err) = store.saveReminder_commit_error(&reminder, true) {
            eprintln!("Could not update reminder: {}", err.localizedDescription());
        }
    });
    {
        // Drop it from the cached list right away; the refetch confirms it.
        let mut state = FETCH.lock().unwrap();
        if index < state.items.len() {
            state.items.remove(index);
        }
    }
    DIRTY.store(true, Ordering::SeqCst);
    refresh();
}

/// Renames the reminder at `index` of the last snapshot in the Reminders app.
pub fn rename(index: usize, title: &str) {
    let title = title.trim();
    let Some(id) = FETCH.lock().unwrap().items.get(index).filter(|r| r.title != title).map(|r| r.id.clone())
    else {
        return;
    };
    if title.is_empty() {
        return;
    }
    STORE.with(|store| unsafe {
        let Some(item) = store.calendarItemWithIdentifier(&objc2_foundation::NSString::from_str(&id))
        else {
            return;
        };
        let Ok(reminder) = item.downcast::<EKReminder>() else {
            return;
        };
        reminder.setTitle(Some(&objc2_foundation::NSString::from_str(title)));
        if let Err(err) = store.saveReminder_commit_error(&reminder, true) {
            eprintln!("Could not rename reminder: {}", err.localizedDescription());
        }
    });
    if let Some(r) = FETCH.lock().unwrap().items.get_mut(index) {
        r.title = title.to_string();
    }
    DIRTY.store(true, Ordering::SeqCst);
    refresh();
}

fn ns_date(t: chrono::DateTime<Local>) -> Retained<NSDate> {
    NSDate::dateWithTimeIntervalSince1970(t.timestamp() as f64)
}

fn start_fetch() {
    {
        let mut state = FETCH.lock().unwrap();
        if state.in_flight {
            // Fetch again once the current one lands.
            NEEDS_FETCH.store(true, Ordering::SeqCst);
            return;
        }
        state.in_flight = true;
    }

    let today = Local::now().date_naive();
    let end_of_today = Local
        .from_local_datetime(&today.and_hms_opt(0, 0, 0).unwrap())
        .unwrap()
        + Duration::days(1);

    STORE.with(|store| unsafe {
        // Open reminders due today or earlier (overdue).
        let open_pred = store.predicateForIncompleteRemindersWithDueDateStarting_ending_calendars(
            None,
            Some(&ns_date(end_of_today)),
            None,
        );
        let open_block = RcBlock::new(move |list: *mut NSArray<EKReminder>| {
            // Keep only reminders due today or overdue. The date range above
            // isn't enough on its own: undated and all-day reminders for later
            // days can still come back from EventKit.
            let mut items: Vec<ReminderItem> = convert(list)
                .into_iter()
                .filter(|r| r.due_date.is_some_and(|d| d <= today))
                .collect();
            // Overdue first, then by due date/time, then title.
            items.sort_by(|a, b| {
                (a.due_date, a.due_time.clone(), a.title.to_lowercase()).cmp(&(
                    b.due_date,
                    b.due_time.clone(),
                    b.title.to_lowercase(),
                ))
            });
            let mut state = FETCH.lock().unwrap();
            state.items = items;
            state.loaded = true;
            state.in_flight = false;
            DIRTY.store(true, Ordering::SeqCst);
        });
        store.fetchRemindersMatchingPredicate_completion(&open_pred, &open_block);
    });
}

fn convert(list: *mut NSArray<EKReminder>) -> Vec<ReminderItem> {
    let Some(list) = (unsafe { list.as_ref() }) else {
        return Vec::new();
    };
    list.iter()
        .map(|r| unsafe {
            let (due_date, due_time) = r
                .dueDateComponents()
                .map(|c| due_from_components(&c))
                .unwrap_or((None, None));
            ReminderItem {
                id: r.calendarItemIdentifier().to_string(),
                title: r.title().to_string(),
                list: r.calendar().map(|c| c.title().to_string()).unwrap_or_default(),
                due_date,
                due_time,
            }
        })
        .collect()
}

fn due_from_components(c: &NSDateComponents) -> (Option<NaiveDate>, Option<String>) {
    let defined = |v: isize| v != NSDateComponentUndefined;
    let date = (defined(c.year()) && defined(c.month()) && defined(c.day()))
        .then(|| NaiveDate::from_ymd_opt(c.year() as i32, c.month() as u32, c.day() as u32))
        .flatten();
    let time = (defined(c.hour()) && defined(c.minute()))
        .then(|| format!("{:02}:{:02}", c.hour(), c.minute()));
    (date, time)
}

/// "Today", "Yesterday" or e.g. "Oct 14".
pub fn relative_day(date: NaiveDate) -> String {
    let today = Local::now().date_naive();
    match (today - date).num_days() {
        0 => "Today".into(),
        1 => "Yesterday".into(),
        _ if date.year() == today.year() => date.format("%b %-d").to_string(),
        _ => date.format("%b %-d, %Y").to_string(),
    }
}
