//! "Open at login" via ServiceManagement (`SMAppService.mainApp`).

use objc2_foundation::NSBundle;
use objc2_service_management::{SMAppService, SMAppServiceStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginItemState {
    Off,
    On,
    /// Registered, but the user has to allow it in System Settings.
    NeedsApproval,
}

pub fn state() -> LoginItemState {
    match unsafe { SMAppService::mainAppService().status() } {
        SMAppServiceStatus::Enabled => LoginItemState::On,
        SMAppServiceStatus::RequiresApproval => LoginItemState::NeedsApproval,
        _ => LoginItemState::Off,
    }
}

/// Registers or unregisters the app as a login item.
pub fn set_enabled(enabled: bool) -> Result<LoginItemState, String> {
    let service = unsafe { SMAppService::mainAppService() };
    let result = unsafe {
        if enabled {
            service.registerAndReturnError()
        } else {
            service.unregisterAndReturnError()
        }
    };
    result.map_err(|err| err.localizedDescription().to_string())?;
    let state = state();
    if state == LoginItemState::NeedsApproval {
        unsafe { SMAppService::openSystemSettingsLoginItems() };
    }
    Ok(state)
}

/// The login item points at this exact app copy, so it should live in
/// /Applications rather than a build or Downloads folder.
pub fn installed_in_applications() -> bool {
    NSBundle::mainBundle().bundlePath().to_string().starts_with("/Applications/")
}
