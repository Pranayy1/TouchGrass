//! # Platform Adaptation Layer
//!
//! ## Responsibility
//! The platform layer translates cross‑platform abstractions into concrete
//! OS‑ or runtime‑specific implementations. It isolates the rest of the
//! codebase from differences between Windows, macOS, Linux, and the web.
//! This layer provides a stable, synchronous interface for capabilities such
//! as file system access, networking, timers, notifications, power state,
//! and hardware access.
//!
//! ## What belongs here
//! - Trait definitions that describe platform services (e.g., `FileSystem`,
//!   `NetworkClient`, `Timer`, `NotificationService`).
//! - Concrete implementations of those traits for each target platform,
//!   selected via compile‑time feature flags or runtime detection.
//! - Adapters that convert OS‑specific handles or error types into the
//!   layer’s uniform error model.
//! - Safe abstractions over raw system APIs (e.g., wrapping raw file
//!   descriptors, registry keys, or GPU contexts).
//! - Minimal, well‑tested shims that provide the same semantics on all
//!   supported platforms.
//!
//! ## What must **never** belong here
//! - Business logic: decisions about when to save a file, what to notify
//!   the user about, or how to interpret a response.
//! - UI concerns: widgets, layouts, or event handling.
//! - Core domain objects or use‑case orchestration.
//! - Persistence details: query builders, migration scripts, or schema
//!   definitions.
//! - Application‑wide state or singletons that should be managed by the
//!   `app` or `services` layers.
//!
//! ## Dependencies
//! This layer may depend on:
//! - `models` (for data structures that cross the boundary, e.g.,
//!    a `FileMetadata` struct).
//! - Widely‑used, pure crates that provide portable abstractions
//!   (e.g., `async-trait`, `futures`, `tokio` if the project is async).
//! - OS‑specific crates only behind feature flags (e.g., `windows-sys`,
//!   `libc`, `apple-sdk`).
//!
//! It must **not** depend on:
//! - `core`, `services`, `app`, `ui`.
//! - Any crate that would pull in UI or business‑logic concerns.
//!
//! Example placeholder: a trait for receiving system idle notifications.
//!
use std::time::Duration; // TODO: remove if not used

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