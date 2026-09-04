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
use std::collections::{BTreeSet, HashMap};

/// An entry in the notification history.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NotificationEntry {
    pub id: String,
    pub title: String,
    pub message: String,
    pub timestamp: i64,
    #[serde(default)]
    pub read_at: Option<i64>,
}

/// The core state of the application tracker.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct TrackerState {
    pub total_millis: u64,
    pub current_app: String,
    pub per_app_millis: HashMap<String, u64>,
    pub tracking_enabled: bool,
    pub hide_on_close: bool,
    pub day_key: String,
    pub processed_hours: u64,
    pub five_hour_alert_sent: bool,
    pub hourly_notifications_sent: BTreeSet<u64>,
    #[serde(default = "default_hourly_notifications_enabled")]
    pub hourly_notifications_enabled: bool,
    #[serde(default)]
    pub notifications: Vec<NotificationEntry>,
}

impl Default for TrackerState {
    fn default() -> Self {
        Self {
            total_millis: 0,
            current_app: String::new(),
            per_app_millis: HashMap::new(),
            tracking_enabled: false,
            hide_on_close: false,
            day_key: String::new(),
            processed_hours: 0,
            five_hour_alert_sent: false,
            hourly_notifications_sent: BTreeSet::new(),
            hourly_notifications_enabled: true,
            notifications: Vec::new(),
        }
    }
}

fn default_hourly_notifications_enabled() -> bool {
    true
}

/// A single application's usage in a snapshot.
#[derive(Debug, Clone, Serialize)]
pub struct UsageEntry {
    pub name: String,
    pub seconds: u64,
    pub percent: f64,
}

/// A snapshot of current usage statistics.
#[derive(Debug, Clone, Serialize)]
pub struct UsageSnapshot {
    pub total_seconds: u64,
    pub current_app: String,
    pub top_app: String,
    pub tracking_enabled: bool,
    pub apps: Vec<UsageEntry>,
}

/// The current tracking status (enabled/disabled).
#[derive(Debug, Clone, Serialize)]
pub struct TrackingStatus {
    pub tracking_enabled: bool,
}

/// The launch-on-startup status.
#[derive(Debug, Clone, Serialize)]
pub struct StartupStatus {
    pub enabled: bool,
}

/// The close-behavior status (hide on close or not).
#[derive(Debug, Clone, Serialize)]
pub struct CloseBehaviorStatus {
    pub hide_on_close: bool,
}

/// The hourly notifications status (enabled/disabled).
#[derive(Debug, Clone, Serialize)]
pub struct HourlyNotificationsStatus {
    pub enabled: bool,
}

/// An alert about usage levels.
#[derive(Debug, Clone, Serialize)]
pub struct UsageAlert {
    pub level: String,
    pub message: String,
    pub total_hours: u64,
    pub total_seconds: u64,
}

/// Information about an available update.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateInfo {
    pub version: String,
    pub notes: String,
    pub release_url: String,
}

/// A GitHub release (used for decoding update checks).
#[derive(Debug, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub body: String,
    pub html_url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_state_default() {
        let state = TrackerState::default();
        assert_eq!(state.total_millis, 0);
        assert_eq!(state.current_app, "");
        assert!(state.per_app_millis.is_empty());
        assert_eq!(state.tracking_enabled, false);
        assert_eq!(state.hide_on_close, false);
        assert_eq!(state.day_key, "");
        assert_eq!(state.processed_hours, 0);
        assert_eq!(state.five_hour_alert_sent, false);
        assert!(state.hourly_notifications_sent.is_empty());
        assert_eq!(state.hourly_notifications_enabled, true);
        assert!(state.notifications.is_empty());
    }

    #[test]
    fn test_tracker_state_serialization() {
        let mut state = TrackerState::default();
        state.total_millis = 1000;
        state.current_app = "test_app".to_string();
        state.per_app_millis.insert("test_app".to_string(), 500);
        state.tracking_enabled = true;
        state.hide_on_close = true;
        state.day_key = "2024-01-01".to_string();
        state.processed_hours = 1;
        state.five_hour_alert_sent = true;
        state.hourly_notifications_sent.insert(1);
        state.hourly_notifications_enabled = false;
        state.notifications.push(NotificationEntry {
            id: "1".to_string(),
            title: "Test".to_string(),
            message: "Test message".to_string(),
            timestamp: 1000,
            read_at: Some(2000),
        });

        let serialized = serde_json::to_string(&state).unwrap();
        let deserialized: TrackerState = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.total_millis, 1000);
        assert_eq!(deserialized.current_app, "test_app");
        assert_eq!(deserialized.per_app_millis.get("test_app"), Some(&500));
        assert_eq!(deserialized.tracking_enabled, true);
        assert_eq!(deserialized.hide_on_close, true);
        assert_eq!(deserialized.day_key, "2024-01-01");
        assert_eq!(deserialized.processed_hours, 1);
        assert_eq!(deserialized.five_hour_alert_sent, true);
        assert_eq!(deserialized.hourly_notifications_sent.get(&1), Some(&1));
        assert_eq!(deserialized.hourly_notifications_enabled, false);
        assert_eq!(deserialized.notifications.len(), 1);
        let entry = &deserialized.notifications[0];
        assert_eq!(entry.id, "1");
        assert_eq!(entry.title, "Test");
        assert_eq!(entry.message, "Test message");
        assert_eq!(entry.timestamp, 1000);
        assert_eq!(entry.read_at, Some(2000));
    }
}