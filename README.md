# DexHowTo HealthTask (macOS Native Rust App)

**DexHowTo HealthTask** is a 100% pure Rust native macOS desktop and menu bar companion engineered for software engineers, developers, and desk workers. It runs silently in the background, monitors keyboard and mouse activity across macOS, tracks active work sessions, and delivers customizable health break notices.

It also tracks water and coffee, shows today's Apple Reminders, keeps a floating Control Center on top of every window (even full-screen apps), and stays quiet while you record your screen.

## 📦 Download

Grab the latest macOS build from the [Releases page](https://github.com/dexhowto/Daily-Health-Tasks/releases): unzip it and move **DexHowTo HealthTask.app** to `/Applications`.

The app is not notarized by Apple, so the first launch is blocked by Gatekeeper. Either right-click the app and choose **Open**, or go to **System Settings › Privacy & Security** and click **Open Anyway**. From Terminal you can also run:

```bash
xattr -dr com.apple.quarantine "/Applications/DexHowTo HealthTask.app"
```

---

## ✨ Features

### 1. ⌨️ Background Activity & Idle Detection
- **Auto-starts work session timer**: When you touch any key on your keyboard or move/click the mouse, DexHowTo HealthTask detects active engagement and starts the session timer.
- **60-Minute Work Limit Notice (Customizable)**: If the keyboard or mouse continues to be used up to the 60-minute threshold (or your customized interval), the app triggers a customizable notice.
- **Smart Away/Rest Detection**: If you step away from your keyboard and mouse (e.g., idle for 5+ minutes), the app recognizes you took a break and automatically resets the session timer so your next work block starts fresh.
- **Zero Special Permissions**: Leverages macOS `CoreGraphics` system event source tracking (`CGEventSourceSecondsSinceLastEventType`), requiring zero intrusive Accessibility permissions.

### 2. 🔔 Customizable Notices & Settings
On the **Settings** page, you can customize:
- **Work Interval Threshold**: 30 min, 45 min, 60 min (default), or 90 min.
- **Idle Reset Threshold**: 3 min, 5 min (default), 10 min, or 15 min of inactivity.
- **Notice Title**: Custom title text (default: *"Time for a Health Break! 🧘"*).
- **Notice Body / Message**: Custom notice text (default: *"You've been using your keyboard & mouse for 60 minutes. Stand up, stretch, and rest your eyes!"*).
- **Delivery Channels**:
  - macOS Notification Center Banner (with sound).
  - Native On-Screen Pop-up Alert Dialog.
  - Native macOS Audio Chimes (Glass, Hero, Ping, etc.).
- **"Test Notice Now"**: Instant test button to preview your custom notification wording and delivery style.

### 3. 🖥️ Menu Bar Live Timer (`NSStatusItem`)
- **Live Elapsed Timer**: Directly in the macOS top menu bar, you see a second-by-second live timer counting up from the moment you started using your PC after the last break:
  - `⏱️ 14:32 / 60:00` while actively working at your keyboard/mouse (showing exact elapsed time `MM:SS` and target limit).
  - `🌿 Break (04:15)` when you step away from the keyboard and mouse to rest.
  - `⚠️ 60:00 - Break Due!` when the 60-minute limit is reached.
  - `⏱️ 00:00 (Ready)` waiting for initial input.
- **Quick Menu Actions**:
  - `⏱️ Active Session: 14:32 (Started: 10:45 AM)`
  - `🔔 60m Break Notice in: 45:28`
  - `🌿 Take a Break Now / Reset Timer` (keyboard shortcut: `b`)
  - Quick Log Water (+250ml)
  - Quick Log Stand & Stretch Done
  - Quick Log 20-20-20 Eye Break Done
  - Open Dashboard / Toggle Sound / Quit

### 4. 📋 Desktop Dashboard (`NSWindow`)
- **Sidebar layout**: Today, Break Timer, Insights and Settings pages with SF Symbols, card-based content and live work-session status.
- **Daily Health Routines Checklist**: Water, 20-20-20 eye breaks, stand & stretch, ergonomic posture alignment, 30-minute movement, screen-free meal, and daily vitamins.
- **Hydration Tracker**: Quick buttons (glass 250 ml, bottle 500 ml, undo) with progress toward a 2,000 ml daily target.
- **Micro-Break Countdown Timers**: Dedicated 20-20-20 eye break rule, 45m stretch session, and 5m deep breathing.
- **7-Day History & Insights**: Daily completion percentages, water, coffee and break counts, streak tracking, and ergonomic desk guides.

### 5. ☕ Coffee & Caffeine Tracking
- Log an **Espresso (63 mg)**, **Coffee (95 mg)** or **Latte (126 mg)** with one click; undo the last entry.
- Caffeine total against the **400 mg** daily guideline, with a bar that turns amber near the limit and red over it.
- Tips such as "Caffeine after 14:00 can make it harder to sleep tonight", plus a coffee column in Insights and a quick-log item in the menu bar (`c`).

### 6. ✅ Apple Reminders
- The **Reminders** card on the Today page lists open reminders due today, overdue ones first.
- Tick a reminder to complete it in the Reminders app; the list updates when Reminders changes.
- Asks for Reminders access the first time you click **Connect Reminders**.

### 7. 🎛 Floating Control Center
- A small black vertical panel that stays **above other windows on every Space, including full-screen apps**, and can be dragged to any screen.
- Session timer, quick buttons (**Water, Coffee, Break**), water / caffeine / habits bars, open tasks with checkboxes, and an **Open Dashboard** button.
- Rename a task in place with the pencil button (saved to Reminders).
- Minimize it to a draggable **DH** tile with a task-count badge; click the tile to expand.
- Show or hide it from the menu bar or **Window › Show/Hide Control Center** (`⌘⇧K`). Position and state are remembered.

### 8. 🎥 Screen Recording Aware
- While the screen is being recorded (any recorder, including Cmd+Shift+5, QuickTime and DexHowTo Recorder), break notices, chimes and pop-ups are **held** and delivered as soon as the recording stops.
- The Control Center collapses to its tile during the recording and expands again afterwards.

---

## 🛠️ Architecture

DexHowTo HealthTask is built 100% in Rust using modern Apple AppKit bindings (`objc2`, `objc2-app-kit`, `objc2-foundation`):
- **Language**: Rust (2024 edition)
- **GUI Engine**: Native macOS Cocoa / AppKit (`NSApplication`, `NSWindow`, `NSPanel`, `NSStatusItem`, `NSStackView`, `NSBox`, `NSButton`, `NSTextField`, `NSSegmentedControl`, `NSAlert`, `NSSound`)
- **Activity Detection**: `CoreGraphics` (`CGEventSourceSecondsSinceLastEventType`)
- **Reminders**: `EventKit` (`objc2-event-kit`)
- **Screen Recording Detection**: macOS privacy indicator window (`CGWindowListCopyWindowInfo`) and capture processes (`libproc`)
- **Storage**: JSON file stored in `~/Library/Application Support/HealthTasks/health_tasks_data.json`

```
src/
├── main.rs              # Application entry point, AppKit activation, main menu bar
├── models.rs            # Domain models (DailyRecord, HealthTask, AppSettings, ActivitySession)
├── storage.rs           # JSON persistence in ~/Library/Application Support/HealthTasks
├── app_state.rs         # Thread-safe global state & activity state machine
├── activity_tracker.rs  # CoreGraphics global keyboard and mouse idle detection
├── notifier.rs          # macOS banner notifications & NSAlert dialog delivery
├── audio.rs             # macOS native NSSound effects (Glass, Hero, Tink, Pop)
├── menu_bar.rs          # NSStatusItem companion and dynamic dropdown menu
├── action_handler.rs    # Objective-C event target class (buttons, menus, timer)
├── controller.rs        # App coordination between event loop, state, and UI
├── reminders.rs         # EventKit: today's open reminders, complete & rename
├── screen_recording.rs  # Detects screen recording to hold notices
└── ui/
    ├── mod.rs              # MainWindow sidebar layout & page coordinator
    ├── components.rs       # AppKit view builders (cards, tiles, progress bars, labels)
    ├── dashboard_view.rs   # Today: session, stats, hydration, coffee, routines
    ├── reminders_card.rs   # Today: Apple Reminders card
    ├── control_center.rs   # Floating always-on-top Control Center panel
    ├── timer_view.rs       # Micro-break timers & instructions
    ├── settings_view.rs    # Notice timing, wording & delivery
    └── history_view.rs     # 7-day insights & ergonomics guide

Info.plist                  # Bundle metadata (embedded into the binary by build.rs)
assets/                     # App icon (AppIcon.icns, icon_1024.png)
scripts/bundle_app.sh       # Builds the .app bundle (UNIVERSAL=1 for Intel + Apple Silicon)
scripts/make_icon.swift     # Regenerates the app icon
```

---

## 🚀 Getting Started

### Prerequisites
- macOS (Apple Silicon or Intel)
- Rust toolchain (`cargo`, `rustc`)

### Run Development Build
```bash
cargo run
```
This immediately starts DexHowTo HealthTask, opens the desktop dashboard, and places the live status item into your macOS top menu bar.

### Build Release `.app` Bundle
To create a standalone macOS `.app` bundle ready to run or copy to `/Applications`:
```bash
./scripts/bundle_app.sh
open "target/release/DexHowTo HealthTask.app"
```

To build a universal app (Apple Silicon + Intel) and a zip for distribution:
```bash
UNIVERSAL=1 ./scripts/bundle_app.sh
```

---

## 💡 How the 60-Minute Notice Works

1. When you touch any keyboard key or move the mouse, DexHowTo HealthTask detects active interaction and begins accumulating your active work time.
2. The menu bar title reflects your progress (e.g., `⏳ 15m/60m`).
3. If activity continues up to your configured limit (default: **60 minutes**), the app dispatches your customized notice via your preferred channels (Banner notification, on-screen alert dialog, and/or chime).
4. If you take a natural break and walk away from the computer for 5 minutes (configurable), the timer resets automatically for your next working session.
