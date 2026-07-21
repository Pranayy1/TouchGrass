//! # Storage / Persistence Layer
//!
//! ## Responsibility
//! The storage layer abstracts persistent data storage. It provides a clean,
//! synchronous interface for creating, reading, updating, and deleting data.
//! The rest of the codebase (especially `core` and `services`) should depend
//! only on the traits defined here, not on concrete implementations.
//!
//! ## What belongs here
//! - Trait definitions for repositories, stores, or data access objects (DAOs).
//!   Examples: `UserRepository`, `SettingsStore`, `EventLog`.
//! - Concrete implementations for various back‑ends:
//!   * Local file system (JSON, TOML, SQLite, etc.)
//!   * Remote sync endpoints (when applicable)
//!   * In‑memory stores for testing
//! - Translation layers that convert between domain models (from `core`/`models`) and the
//!   storage format.
//! - Migration utilities or schema versioning helpers.
//! - Error types specific to storage operations.
//!
//! ## What must NOT belong here
//! - Business rules: when to save, what data is valid, or how to interpret stored data.
//! - UI concerns: widgets, themes, or event handling.
//! - Core domain logic: use‑case orchestration or domain services.
//! - Platform‑specific APIs: direct file‑system or syscalls should be wrapped by the
//!   `platform` layer; storage should only depend on the abstractions it defines.
//! - Networking details beyond generic I/O (those belong to `platform` or a dedicated
//!   `network` module if one existed).
//!
//! ## Dependencies
//! - `models` (for shared data structures).
//! - `platform` (only if the storage implementation needs to locate platform‑specific
//!   directories – e.g., via `platform::dirs::data_dir()`).
//! - General‑purpose, pure Rust crates (e.g., `serde`).
//!
//! It must NOT depend on:
//! - `core`, `services`, `app`, `ui`.
//! - Any crate that would pull in UI or business‑logic concerns.
//!
//! Example placeholder: a simple in‑memory key‑value store trait and implementation.
//! For the prototype we make the store cloneable for convenience.
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// A generic key‑value store trait.
pub trait KvStore: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, Self::Error>;
    fn set(&self, key: &str, value: Vec<u8>) -> Result<(), Self::Error>;
    fn delete(&self, key: &str) -> Result<(), Self::Error>;
}

/// Simple in‑memory implementation useful for tests and early prototypes.
// We derive Clone so that we can easily copy the store (the inner Arc is cheap to clone).
#[derive(Clone, Debug, Default)]
pub struct MemStore {
    inner: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl KvStore for MemStore {
    type Error = std::io::Error;

    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, Self::Error> {
        // If the lock is poisoned, we treat it as an I/O error.
        let guard = self.inner.read().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::Other, "RwLock poisoned")
        })?;
        Ok(guard.get(key).cloned())
    }

    fn set(&self, key: &str, value: Vec<u8>) -> Result<(), Self::Error> {
        let mut guard = self.inner.write().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::Other, "RwLock poisoned")
        })?;
        guard.insert(key.to_string(), value);
        Ok(())
    }

    fn delete(&self, key: &str) -> Result<(), Self::Error> {
        let mut guard = self.inner.write().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::Other, "RwLock poisoned")
        })?;
        guard.remove(key);
        Ok(())
    }
}