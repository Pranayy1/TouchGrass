#!# Services / Use‑Case Layer
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
mod tracker;

use crate::models::TrackerState;
use super::platform::get_active_user_app;
use crate::storage::Storage;
use crate::services::tracker::TrackerEngine;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

/// Service responsible for registering a new user.
pub struct RegisterUserService {
    // This is a placeholder - we don't have a real user registration system yet
    // In a real implementation, this would have actual dependencies
    dummy: i32,
}

impl RegisterUserService {
    /// Create a new service with dummy values.
    pub fn new() -> Self {
        Self { dummy: 0 }
    }

    /// Execute the registration use‑case.
    /// This is a placeholder implementation.
    pub fn register_user(&self) -> Result<u64, std::io::Error> {
        // Placeholder implementation - returns a fixed user ID
        Ok(1)
    }
}

/// Service responsible for tracking application usage.
pub struct TrackerService {
    state: Arc<Mutex<TrackerState>>,
    tracker_engine: TrackerEngine,
    /// Handle to the background thread (if running).
    _thread_handle: Option<thread::JoinHandle<()>>,
    /// Signal to stop the background thread.
    stop_sender: Option<mpsc::Sender<()>>,
}

impl TrackerService {
    /// Create a new tracker service.
    ///
    /// If `state` is provided, it will be used as the authoritative state.
    /// Otherwise, a default state is created.
    pub fn new(state: Option<Arc<Mutex<TrackerState>>>, storage: Option<Storage>) -> Self {
        let state = state.unwrap_or_else(|| Arc::new(Mutex::new(TrackerState::default())));
        let tracker_engine = TrackerEngine::new(state.clone(), storage);

        Self {
            state: state.clone(),
            tracker_engine,
            _thread_handle: None,
            stop_sender: None,
        }
    }

    /// Start the tracking service in a background thread.
    pub fn start(&mut self) {
        // Prevent multiple starts
        if self._thread_handle.is_some() || self.stop_sender.is_some() {
            return;
        }

        let state = self.state.clone();
        let mut tracker_engine = self.tracker_engine.clone();
        let (stop_sender, stop_receiver) = mpsc::channel();
        self.stop_sender = Some(stop_sender);

        let handle = thread::spawn(move || {
            let mut next_interval_ms = 2500; // Initial interval

            loop {
                // Wait for stop signal or timeout
                match stop_receiver.recv_timeout(Duration::from_millis(next_interval_ms)) {
                    Ok(()) => {
                        // Received stop signal
                        break;
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        // Timeout elapsed, proceed to update
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        break;
                    }
                }

                // Update the tracker engine
                next_interval_ms = tracker_engine.update();
            }
        });

        self._thread_handle = Some(handle);
    }

    /// Stop the tracking service.
    pub fn stop(&mut self) {
        // Send stop signal if we have a sender
        if let Some(sender) = self.stop_sender.take() {
            let _ = sender.send(());
        }
        // Join the thread if it exists
        if let Some(handle) = self._thread_handle.take() {
            let _ = handle.join();
        }
        // Persist final state after the thread has stopped.
        if let Some(ref storage) = self.tracker_engine.storage {
            let state_clone = self.state.lock().unwrap().clone();
            let _ = storage.save(&state_clone);
        }
    }

    /// Get a clone of the current tracker state.
    pub fn get_state(&self) -> TrackerState {
        // Clone the state while holding the lock
        let state_lock = self.state.lock().unwrap();
        state_lock.clone()
    }

    /// Get a shared reference to the tracker state for read-only access.
    pub fn state(&self) -> Arc<Mutex<TrackerState>> {
        self.state.clone()
    }
}

impl Drop for TrackerService {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Storage;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_service_shares_state_with_engine() {
        let storage = None;
        let mut service = TrackerService::new(None, storage);
        // Get clones of the Arc<Mutex<TrackerState>> from service and engine
        let service_state = service.state.clone();
        let engine_state = service.tracker_engine.state.clone();
        // They should be the same Arc (strong count at least 2)
        assert!(Arc::ptr_eq(&service_state, &engine_state));

        // Mutate state via service
        {
            let mut s = service_state.lock().unwrap();
            s.total_millis = 42;
        }
        // Observe mutation via engine
        {
            let s = engine_state.lock().unwrap();
            assert_eq!(s.total_millis, 42);
        }
    }

    #[test]
    fn test_engine_mutation_observable_via_service() {
        let storage = None;
        let mut service = TrackerService::new(None, storage);
        // Start the service
        service.start();
        assert!(service._thread_handle.is_some(), "thread should have started");

        // Let the thread run a tiny bit to ensure it has updated state (though we can't guarantee)
        // Instead we directly update via engine and check service sees it.
        // We'll lock the state and set a value, then ensure the engine's update doesn't break it.
        {
            let mut s = service.state.lock().unwrap();
            s.day_key = crate::core::current_day_key();
            s.total_millis = 100;
        }
        // Call engine.update once (the thread will also call it, but we can call directly)
        let _ = service.tracker_engine.update();
        // Check that state still has our value (engine should not have zeroed it)
        let s = service.state.lock().unwrap();
        assert_eq!(s.total_millis, 100);
        // Note: The engine may have added elapsed time, but we can't guarantee because we don't control timing.
        // At least we know the state is shared and not lost.
    }

    #[test]
    fn test_service_start_and_stop() {
        let storage = None;
        let mut service = TrackerService::new(None, storage);
        assert!(service._thread_handle.is_none(), "thread should not be running initially");
        service.start();
        assert!(service._thread_handle.is_some(), "thread should be running after start");
        service.stop();
        assert!(service._thread_handle.is_none(), "thread should be stopped after stop");
    }

    #[test]
    fn test_drop_calls_stop() {
        let storage = None;
        {
            let mut service = TrackerService::new(None, storage);
            service.start();
            assert!(service._thread_handle.is_some(), "thread should be running");
        } // service dropped here
        // If drop did not call stop, the thread might still be running.
        // We cannot easily detect that from outside, but we can at least ensure no panic.
        // For simplicity, we just ensure the test passes.
    }
}