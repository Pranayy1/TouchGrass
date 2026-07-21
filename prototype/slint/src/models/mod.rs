//! # Shared Models
//!
//! ## Responsibility
//! The models layer contains the shared data structures that are used across
//! layer boundaries. These are primarily plain data transfer objects (DTOs),
//! value objects, enums, and simple structs that contain no behavior.
//! They serve as the lingua franca between layers, allowing changes in one
//! layer to be expressed in a stable format for another.
//!
//! ## What belongs here
//! - Pure data structs/enums with `#[derive(Debug, Clone, PartialEq, Eq, ...)]`.
//! - Value objects that encapsulate domain primitives (e.g., `UserId`,
//!   `EmailAddress`) and may contain validation logic but **no** business
//!   rules.
//! - Data transfer objects used for communication between layers (e.g.,
//!   between `services` and `ui`, or between `storage` and `core`).
//! - Serialization/deserialization aids (e.g., `serde` attributes) if the
//!   project adopts a serialization format for storage or IPC.
//! - Marker types or enums that represent state shared across layers.
//!
//! ## What must **never** belong here
//! - Business logic: methods that implement rules, workflows, or domain
//!   operations.
//! - Dependencies on other layers: a model must not know whether it came
//!   from UI, core, or storage. Keeping it pure prevents coupling.
//! - Async types, futures, or reactor‑specific handles (unless the type is
//!   purely a wrapper over an OS handle that is inert).
//! - UI‑specific widgets, styles, or Slint component types.
//! - Persistence‑concerns: table names, column mappings, ORM annotations.
//!   (These belong in `storage` or `services`.)
//!
//! ## Dependencies
//! This layer must have **zero** dependencies on any other local module.
//! It may depend on widely‑used, pure utility crates (e.g., `serde`,
//! `uuid`, `chrono`) as long as those dependencies do not pull in I/O or
//! runtime requirements that would leak concerns.
//!
//! Example placeholder: a simple user identifier.
//! For the prototype we use a u64; in a real project you might replace it
//! with a UUID or a database‑generated ID.
//!
use serde::{Deserialize, Serialize};

/// An opaque, type‑safe user identifier.
/// Using a simple u64 for the prototype; replace with UUID or similar later.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub u64);

impl UserId {
    /// Generate a new user ID.
    /// In a real implementation, this would be a proper UUID or
    /// database‑generated ID.
    #[must_use]
    pub fn new() -> Self {
        // Placeholder: static counter for demo purposes only.
        // NOTE: Not thread‑safe; acceptable for a placeholder.
        static mut COUNTER: u64 = 0;
        unsafe {
            let id = COUNTER;
            COUNTER = COUNTER.wrapping_add(1);
            Self(id)
        }
    }
}

/// A simple display name with basic validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisplayName(String);

impl DisplayName {
    /// Creates a new `DisplayName` after validating the input.
    ///
    /// Returns `None` if the string is empty or longer than 32 characters.
    pub fn new(name: impl Into<String>) -> Option<Self> {
        let name = name.into();
        if !name.is_empty() && name.len() <= 32 {
            Some(Self(name))
        } else {
            None
        }
    }

    /// Access the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}