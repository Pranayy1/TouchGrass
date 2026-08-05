//! # UI Layer (Slint Adapter)
//!
//! ## Responsibility
//! The UI layer contains the Slint user‑interface definitions and a thin adapter
//! that translates UI events into calls to the `services` layer (or
//! `app` layer) and pushes state changes from the application back into the
//! UI. It knows nothing about business rules; it only presents data and
//! forwards user gestures.
//!
//! ## What belongs here
//! - `.slint` files that define windows, widgets, callbacks, and bindings.
//! - A Rust adapter struct (often called `UiHandle` or similar) that:
//!   * Instantiates the Slint component via `slint::include_modules!()`.
//!   * Holds references to services or app‑layer objects needed to handle
//!     user actions.
//!   * Connects Slint callbacks to asynchronous service calls (using
//!     `slint::ComponentWeakHandle` or `spawn_local`).
//!   * Provides methods for the application to update the UI (e.g.,
//!     `set_user_name(&self, name: &str)`).
//! - Styling resources, custom widgets, or reusable UI components.
//! - Localization strings if they are tightly coupled to the UI.
//! - Animation or transition descriptions that are purely presentational.
//!
//! ## What must NOT belong here
//! - Business logic: decisions about when to save data, validation rules,
//!   or workflow orchestration.
//! - Direct calls to platform APIs (file system, networking, etc.); those
//!   should be delegated to services or the platform layer.
//! - Core domain objects or use‑case orchestration.
//! - Persistence details: SQL queries, file handles, or repository traits.
//! - Application‑wide state that should be owned by the `app` or `services`
//!   layers.
//! - Blocking or long‑running work performed directly in UI callbacks.
//!
//! ## Dependencies
//! - `models` (to receive data for display and to send user‑generated data).
//! - `services` (to invoke use‑cases in response to UI events).
//! - `app` (only if the UI needs to start/stop the application lifecycle).
//! - The `slint` crate and `slint-build` (already declared in Cargo.toml).
//!
//! It must NOT depend on:
//! - `core` (to avoid leaking domain logic into the presentation layer).
//! - `storage` or `platform` directly.
//!
//! ## Example placeholder
//! The UI simply shows a placeholder window; the adapter exposes a method
//! to trigger a dummy service call.
//!
use slint::ComponentHandle;

// Bring in the Slint component defined in `src/ui/main.slint`.
slint::include_modules!();

pub use self::preview::PreviewWindow;

// A simple handle that the application can use to update UI text.
// In a real implementation this would hold weak handles to the Slint
// component and possibly callbacks from services.
pub struct UiHandle {
    /// Handle to the Slint window so we can set properties.
    // We'll store the window directly; for a prototype we don't need
    // weak handles.
    pub window: PreviewWindow,
}

impl UiHandle {
    /// Create a new UI handle from a Slint component instance.
    pub fn new(window: PreviewWindow) -> Self {
        Self { window }
    }

    /// Example slot: called from the UI when a button is pressed.
    /// In a real app this would forward to a service.
    pub async fn on_button_clicked(&self) {
        // Placeholder: in reality we would invoke a service here.
        // For now just log.
        println!("UI button clicked (placeholder)");
        // We cannot set the title because the title property is not settable from Rust.
        // Instead, we could change the text of the internal Text element, but we don't
        // have access to it without exposing it in the .slint file.
        // So we just leave it as is.
    }
}