use std::sync::mpsc;

/// Application commands that can be sent from the tray to the main application
#[derive(Debug, Clone)]
pub enum ApplicationCommand {
    ShowWindow,
    Quit,
}

/// Abstract tray service interface
pub trait TrayService: Send + Sync {
    /// Create a new tray service
    fn new(command_tx: mpsc::Sender<ApplicationCommand>) -> Self where Self: Sized;

    /// Start the tray service
    fn start(&self);

    /// Stop the tray service
    fn stop(&self);
}

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::WindowsTrayService as PlatformTrayService;

#[cfg(not(target_os = "windows"))]
mod dummy;
#[cfg(not(target_os = "windows"))]
pub use dummy::DummyTrayService as PlatformTrayService;