use std::time::Duration;

/// A service that notifies subscribers when the system becomes idle or
/// active again. Implementations exist for each desktop platform.
pub trait IdleNotifier: Send + Sync {
    /// Returns `true` if the system is currently idle.
    fn is_idle(&self) -> bool;

    /// Subscribes to idle state changes.
    ///
    /// The closure `on_change` is invoked whenever the idle state flips.
    /// The subscription remains active until the returned `IdleHandle`
    /// is dropped.
    fn subscribe_idle<F>(&self, on_change: F) -> Box<dyn IdleHandle>
    where
        F: Fn(bool) + Send + Sync + 'static;
}

/// Handle to an active idle‑notifications subscription.
pub trait IdleHandle: Send + Sync {
    /// Stops receiving further idle‑state notifications.
    fn stop(self: Box<Self>);
}

// Platform‑specific implementations would live in submodules, e.g.:
// #[cfg(windows)]
// mod win32;
// #[cfg(any(target_os = "macos", target_os = "ios"))]
// mod apple;
// #[cfg(all(unix, not(target_os = "macos"), not(target_os = "ios")))]
// mod unix;

#[cfg(target_os = "windows")]
pub mod windows;

/// Get the name of the currently active user application.
///
/// Returns None if there is no active user application (e.g., system lock screen,
/// or only system applications are running).
///
/// # Platform-specific
/// On Windows, this uses the Windows API to get the foreground window, then the
/// process ID, then the executable name, and filters out system applications.
// On other platforms, returns None (to be implemented later).
pub fn get_active_user_app() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        return crate::platform::windows::get_active_user_app();
    }

    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

/// System tray service abstraction
pub mod tray;
pub use tray::{PlatformTrayService, TrayService};