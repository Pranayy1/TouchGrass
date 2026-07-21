//! Entry point for the TouchGrass Slint prototype.
//!
//! This file is deliberately thin: it only sets up the UI, creates placeholder
//! implementations of the lower layers, and connects them together. No business
//! logic lives here – all domain concerns are isolated in the respective
//! layers (`core`, `services`, `storage`, etc.).
//!
//! The actual architecture diagram (see bottom of this file) shows the intended
//! data flow: UI → Services → Core → Storage/Platform, with Models shared
//! across layers.
//!
//! Note: For the prototype we use concrete types for store and core, and we
//! pass them by value to the service. In a real application, we would likely
//! use dependency injection and possibly wrap shared state in `Arc`.

// Import the layers we will wire together.
// In a real implementation we would replace the placeholder types with
// concrete ones built from dependency injection.
mod app;
mod core;
mod models;
mod platform;
mod services;
mod storage;
mod ui;

use crate::ui::{PreviewWindow, UiHandle};
use slint::ComponentHandle;

fn main() {
    // -------------------------------------------------
    // 1. Set up the Slint UI.
    // -------------------------------------------------
    // `PreviewWindow` is the component defined in src/ui/preview.slint.
    let window = PreviewWindow::new().expect("failed to create UI window");
    let ui_handle = UiHandle::new(window);

    // -------------------------------------------------
    // 2. Create placeholder lower‑layer instances.
    // -------------------------------------------------
    let store = storage::MemStore::default();
    let core = core::CoreEngine; // Zero‑sized, cheap to move.
    let services = services::RegisterUserService::new(store, core);

    // -------------------------------------------------
    // 3. Wire UI events to service calls (placeholder).
    // -------------------------------------------------
    // For the prototype we just show that the UI handle exists.
    // In a real implementation, we would connect Slint callbacks to
    // invoke services via `spawn_local` or similar.
    // -------------------------------------------------

    // -------------------------------------------------
    // 4. Run the Slint event loop.
    // -------------------------------------------------
    // Note: `window` has been moved into `ui_handle`, but we can still
    // access it via `ui_handle.window`.
    ui_handle.window.run().expect("failed to run UI event loop");
}

/* -------------------------------------------------------------------------
   Architecture diagram (Markdown)

   UI
   ↓
   Services
   ↓
   Core
   ↓
   Storage / Platform
   (Models are shared across all layers)

   Why this layout helps maintenance:
   - Each layer has a single, well‑defined responsibility.
   - Dependencies point downward; no layer knows about the layers above it.
   - Swapping a storage backend (e.g., from JSON to SQLite) only requires
     changing the `storage` layer; the core and services remain untouched.
   - The UI can be replaced (e.g., with a web frontend) without touching
     business logic because it only interacts via the stable service
     interface.
   - Unit‑testing the core is trivial because it depends only on pure
     `models` and has no side‑effects.
   - The app layer (`main.rs`) is just a composition root; it can be thin
     and replaced for different deployment targets (desktop, embedded, etc.).
   ------------------------------------------------------------------------- */