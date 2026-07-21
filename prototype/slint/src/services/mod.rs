//! # Services / Use‑Case Layer
//!
//! ## Responsibility
//! The services layer (sometimes called the application or use‑case layer)
//! contains application‑specific business rules that orchestrate the flow
//! of data to and from the core domain, but does not contain the domain
//! logic itself. It coordinates multiple core domain objects, persists
//! results via the storage layer, and translates external events (from UI
//! or platform) into actions the core can understand.
//!
//! ## What belongs here
//! - Use‑case / application service objects that represent a single user
//!   goal or system event (e.g., `CreateUserService`,
//!   `UpdateSettingsService`, `SyncDataService`).
//! - Orchestration logic: fetch data from repositories, invoke domain
//!   methods in `core`, persist results, and possibly publish events.
//! - Validation that crosses aggregate boundaries (while simple field
//!   validation may live in `models` or `core`, complex cross‑aggregate
//!   checks belong here).
//! - Transaction‑like workflows that span multiple storage calls.
//! - Mapping between DTOs from `models` and domain entities from `core`
//!   when needed.
//! - Emission of domain events that the `ui` or `platform` layers may
//!   subscribe to.
//!
//! ## What must NOT belong here
//! - UI rendering details: widget state, animations, or layout.
//! - Platform‑specific calls: file system, networking, timers (unless
//!   wrapped via abstractions in `platform` and injected).
//! - Core domain invariants: those belong strictly in `core`.
//! - Low‑level storage details: SQL queries, file formats; those belong
//!   in `storage` implementations.
//! - Concurrency primitives that are not part of the use‑case contract
//!   (e.g., global thread pools); prefer injecting executors or runtime
//!   handles.
//!
//! ## Dependencies
//! - `core` (to invoke domain behavior).
//! - `storage` (via repository traits).
//! - `models` (for DTOs shared with UI or persistence).
//! - `platform` only through abstract interfaces (e.g., a `Clock` trait
//!   for testable time‑keeping).
//! - External, pure‑rust crates that provide utility (e.g., `chrono`,
//!   `uuid`).
//!
//! It must NOT depend on:
//! - `app`, `ui` (to avoid UI‑specific coupling).
//! - Any UI framework types (e.g., Slint widgets) directly.
//!
//! Example placeholder: a service that registers a new user.
//!
use crate::core::CoreEngine;
use crate::core::CoreLogic;
use crate::models::UserId;
use crate::storage::{KvStore, MemStore};

/// Service responsible for registering a new user.
pub struct RegisterUserService {
    store: MemStore,
    core: CoreEngine,
}

impl RegisterUserService {
    /// Create a new service with the given store and core.
    pub fn new(store: MemStore, core: CoreEngine) -> Self {
        Self { store, core }
    }

    /// Execute the registration use‑case.
    ///
    /// Steps:
    /// 1. Generate a new user ID.
    /// 2. Ask the core to validate/initialize the user (placeholder).
    /// 3. Persist the user record.
    pub fn register_user(&self) -> Result<UserId, std::io::Error> {
        let user_id = UserId::new();

        // Delegate any domain validation to core (though core currently
        // does nothing).
        self.core.execute();

        // Persist a minimal representation: just the ID as UTF‑8 bytes.
        let payload = user_id.0.to_string().into_bytes();
        self.store
            .set(&format!("user:{}", user_id.0), payload)?;

        Ok(user_id)
    }
}