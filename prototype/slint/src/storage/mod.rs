use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use chrono::Local;
use crate::models::TrackerState;

/// Errors that can occur when interacting with the storage.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON serialization/deserialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("State file not found")]
    NotFound,
}

/// Handle for persisting and restoring TrackerState as JSON.
#[derive(Clone)]
pub struct Storage {
    /// Path to the state file.
    path: PathBuf,
}

impl Storage {
    /// Create a new storage instance that will use the given path for the state file.
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            path: path.into(),
        }
    }

    /// Load the TrackerState from the state file.
    ///
    /// If the file does not exist, returns a default state as the original application would:
    /// tracking enabled, hide on close, and daily state reset for the current day.
    ///
    /// # Errors
    /// Returns `StorageError::Io` if there is an issue reading the file (other than not found),
    /// or `StorageError::Serde` if the file contains invalid JSON.
    pub fn load(&self) -> Result<TrackerState, StorageError> {
        if !self.path.exists() {
            // Return the default state as the original application would when no state file exists.
            let mut state = TrackerState::default();
            state.tracking_enabled = true;
            state.hide_on_close = true;
            // hourly_notifications_enabled is already true by default in TrackerState.

            // Set the day key to today's date and reset the daily state.
            let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
            state.day_key = today;
            // Reset the daily-dependent fields to match the original reset_daily_state behavior.
            state.total_millis = 0;
            state.per_app_millis.clear();
            state.processed_hours = 0;
            state.five_hour_alert_sent = false;
            state.hourly_notifications_sent.clear();
            state.current_app = "Waiting for user app".to_string();

            Ok(state)
        } else {
            let json = fs::read_to_string(&self.path)?;
            let state: TrackerState = serde_json::from_str(&json)?;
            Ok(state)
        }
    }

    /// Save the TrackerState to the state file.
    ///
    /// The save is performed atomically by writing to a temporary file and then renaming it
    /// to the target file. This helps prevent corruption if the process is interrupted during the save.
    ///
    /// # Errors
    /// Returns `StorageError::Io` if there is an issue creating the file, writing to it,
    /// or renaming the temporary file.
    pub fn save(&self, state: &TrackerState) -> Result<(), StorageError> {
        // Ensure the parent directory exists.
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Create a temporary file in the same directory.
        let temp_path = {
            let mut path = self.path.clone();
            path.set_extension("tmp");
            path
        };

        // Serialize the state to JSON.
        let json = serde_json::to_string_pretty(state)?;

        // Write to the temporary file.
        fs::write(&temp_path, json)?;

        // Rename the temporary file to the target file (atomic on most modern filesystems).
        fs::rename(&temp_path, &self.path)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let state_file = temp_dir.path().join("test_state.json");
        let storage = Storage::new(&state_file);

        // Create a test state.
        let mut original = TrackerState::default();
        original.total_millis = 1500;
        original.current_app = "test_app".to_string();
        original.per_app_millis.insert("test_app".to_string(), 1500);
        original.tracking_enabled = true;
        original.hide_on_close = true;
        original.day_key = "2024-01-01".to_string();
        original.processed_hours = 1;
        original.five_hour_alert_sent = true;
        original.hourly_notifications_sent.insert(1);
        original.hourly_notifications_enabled = false;
        original.notifications.push(crate::models::NotificationEntry {
            id: "1".to_string(),
            title: "Test".to_string(),
            message: "Test message".to_string(),
            timestamp: 1000,
            read_at: Some(2000),
        });

        // Save the state.
        storage.save(&original).expect("Failed to save state");

        // Load the state back.
        let loaded = storage.load().expect("Failed to load state");

        // Verify that the loaded state matches the original.
        assert_eq!(loaded.total_millis, original.total_millis);
        assert_eq!(loaded.current_app, original.current_app);
        assert_eq!(loaded.per_app_millis, original.per_app_millis);
        assert_eq!(loaded.tracking_enabled, original.tracking_enabled);
        assert_eq!(loaded.hide_on_close, original.hide_on_close);
        assert_eq!(loaded.day_key, original.day_key);
        assert_eq!(loaded.processed_hours, original.processed_hours);
        assert_eq!(loaded.five_hour_alert_sent, original.five_hour_alert_sent);
        assert_eq!(loaded.hourly_notifications_sent, original.hourly_notifications_sent);
        assert_eq!(loaded.hourly_notifications_enabled, original.hourly_notifications_enabled);
        assert_eq!(loaded.notifications.len(), original.notifications.len());
        if let (Some(original), Some(loaded)) = (
            original.notifications.first(),
            loaded.notifications.first(),
        ) {
            assert_eq!(original.id, loaded.id);
            assert_eq!(original.title, loaded.title);
            assert_eq!(original.message, loaded.message);
            assert_eq!(original.timestamp, loaded.timestamp);
            assert_eq!(original.read_at, loaded.read_at);
        }
    }

    #[test]
    fn test_load_missing_file_returns_default() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let state_file = temp_dir.path().join("nonexistent.json");
        let storage = Storage::new(&state_file);

        // Loading a non-existent file should return the default state as the original would.
        let state = storage.load().expect("Failed to load state");

        // Check that the state has the defaults set by the original application when no file exists.
        assert!(state.tracking_enabled, "tracking should be enabled by default");
        assert!(state.hide_on_close, "hide_on_close should be true by default");
        assert!(state.hourly_notifications_enabled, "hourly notifications should be enabled by default");
        // The day key should be set to today's date.
        let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
        assert_eq!(state.day_key, today);
        // The daily state should be reset.
        assert_eq!(state.total_millis, 0);
        assert!(state.per_app_millis.is_empty());
        assert_eq!(state.processed_hours, 0);
        assert!(!state.five_hour_alert_sent);
        assert!(state.hourly_notifications_sent.is_empty());
        assert_eq!(state.current_app, "Waiting for user app");
    }

    #[test]
    fn test_load_invalid_json() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let state_file = temp_dir.path().join("invalid.json");
        let storage = Storage::new(&state_file);

        // Write invalid JSON to the file.
        fs::write(&state_file, "not json").expect("Failed to write invalid json");

        // Loading should return a Serde error.
        let result = storage.load();
        assert!(result.is_err(), "Expected an error when loading invalid JSON");
        if let Err(StorageError::Serde(_)) = result {
            // Pass
        } else {
            panic!("Expected StorageError::Serde, got an error of another type");
        }
    }

    #[test]
    fn test_atomic_save() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let state_file = temp_dir.path().join("state.json");
        let storage = Storage::new(&state_file);

        let state = TrackerState::default();

        // Save the state.
        storage.save(&state).expect("Failed to save state");

        // The state file should exist.
        assert!(state_file.exists(), "State file should exist after save");

        // There should be no temporary file left.
        let temp_file = state_file.with_extension("tmp");
        assert!(!temp_file.exists(), "Temporary file should not exist after save");
    }
}