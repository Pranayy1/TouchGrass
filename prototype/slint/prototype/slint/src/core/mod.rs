use chrono::Local;
use crate::models::{TrackerState, UsageSnapshot, UsageEntry};

/// Returns the current day key in YYYY-MM-DD format.
pub fn current_day_key() -> String {
    Local::now().date_naive().format("%Y-%m-%d").to_string()
}

/// Resets the daily state fields in the provided TrackerState.
pub fn reset_daily_state(state: &mut TrackerState) {
    state.total_millis = 0;
    state.per_app_millis.clear();
    state.processed_hours = 0;
    state.five_hour_alert_sent = false;
    state.hourly_notifications_sent.clear();
    state.current_app = if state.tracking_enabled {
        "Waiting for user app".to_string()
    } else {
        "Tracking paused".to_string()
    };
    state.day_key = current_day_key();
}

/// Ensures daily rollover: if the day has changed, resets daily state and returns true.
/// Returns false if the day has not changed.
pub fn ensure_daily_rollover(state: &mut TrackerState) -> bool {
    let today = current_day_key();
    if state.day_key != today {
        reset_daily_state(state);
        true
    } else {
        false
    }
}

/// Converts milliseconds to seconds.
pub fn millis_to_seconds(ms: u64) -> u64 {
    ms / 1000
}

/// Creates a usage snapshot from the given tracker state.
pub fn snapshot_from_state(state: &TrackerState) -> UsageSnapshot {
    let total_seconds = millis_to_seconds(state.total_millis);
    let mut apps: Vec<UsageEntry> = state
        .per_app_millis
        .iter()
        .map(|(name, ms)| {
            let seconds = millis_to_seconds(*ms);
            UsageEntry {
                name: name.clone(),
                seconds,
                percent: if total_seconds == 0 {
                    0.0
                } else {
                    (seconds as f64 / total_seconds as f64) * 100.0
                },
            }
        })
        .collect();

    apps.sort_by(|a, b| b.seconds.cmp(&a.seconds));
    apps.truncate(8);

    let top_app = apps
        .first()
        .map(|e| e.name.clone())
        .unwrap_or_else(|| "No tracked app".to_string());

    UsageSnapshot {
        total_seconds,
        current_app: state.current_app.clone(),
        top_app,
        tracking_enabled: state.tracking_enabled,
        apps,
    }
}

/// Strip the leading 'v' from a version string if present.
pub fn strip_v_prefix(version: &str) -> String {
    version.strip_prefix('v').unwrap_or(version).to_string()
}

/// Convert a version string into a tuple of u32 components.
pub fn version_tuple(version: &str) -> Vec<u32> {
    strip_v_prefix(version)
        .split('.')
        .map(|s| s.parse::<u32>().unwrap_or(0))
        .collect()
}

/// Determine if the latest version is newer than the current version.
pub fn is_newer(latest: &str, current: &str) -> bool {
    version_tuple(latest) > version_tuple(current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{TrackerState, UsageEntry};
    use std::collections::{HashMap, HashSet};

    #[test]
    fn test_current_day_key_format() {
        let key = current_day_key();
        // Expect format YYYY-MM-DD
        let parts: Vec<&str> = key.split('-').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0].len(), 4); // year
        assert_eq!(parts[1].len(), 2); // month
        assert_eq!(parts[2].len(), 2); // day
        // Ensure they are numbers
        let year = parts[0].parse::<i32>().unwrap();
        let month = parts[1].parse::<u32>().unwrap();
        let day = parts[2].parse::<u32>().unwrap();
        assert!(year > 2000);
        assert!(month >= 1 && month <= 12);
        assert!(day >= 1 && day <= 31);
    }

    #[test]
    fn test_reset_daily_state() {
        let mut state = TrackerState {
            total_millis: 1000,
            current_app: "old_app".to_string(),
            per_app_millis: vec![("old".to_string(), 500)].into_iter().collect(),
            tracking_enabled: false,
            hide_on_close: false,
            day_key: "2024-01-01".to_string(),
            processed_hours: 10,
            five_hour_alert_sent: true,
            hourly_notifications_sent: vec![5].into_iter().collect(),
            hourly_notifications_enabled: false,
            notifications: vec![],
        };

        reset_daily_state(&state);

        assert_eq!(state.total_millis, 0);
        assert!(state.per_app_millis.is_empty());
        assert_eq!(state.processed_hours, 0);
        assert_eq!(state.five_hour_alert_sent, false);
        assert!(state.hourly_notifications_sent.is_empty());
        // current_app should be set based on tracking_enabled (false) => "Tracking paused"
        assert_eq!(state.current_app, "Tracking paused");
        // day_key should be today's date
        assert_eq!(state.day_key, current_day_key());
        // tracking_enabled unchanged
        assert_eq!(state.tracking_enabled, false);
        // hide_on_close unchanged
        assert_eq!(state.hide_on_close, false);
        // hourly_notifications_enabled unchanged
        assert_eq!(state.hourly_notifications_enabled, false);
    }

    #[test]
    fn test_ensure_daily_rollover_no_change() {
        let mut state = TrackerState {
            total_millis: 0,
            current_app: "".to_string(),
            per_app_millis: HashMap::new(),
            tracking_enabled: false,
            hide_on_close: false,
            day_key: current_day_key(), // same as today
            processed_hours: 0,
            five_hour_alert_sent: false,
            hourly_notifications_sent: HashSet::new(),
            hourly_notifications_enabled: true,
            notifications: vec![],
        };

        let changed = ensure_daily_rollover(&state);
        assert!(!changed);
        // State should be unchanged because day_key matches
        assert_eq!(state.day_key, current_day_key());
        assert_eq!(state.total_millis, 0);
        assert!(state.per_app_millis.is_empty());
    }

    #[test]
    fn test_ensure_daily_rollover_change() {
        // Set day_key to yesterday
        let yesterday = chrono::Local::now()
            .date_naive()
            .pred_opt()
            .expect("could not get yesterday")
            .format("%Y-%m-%d")
            .to_string();

        let mut state = TrackerState {
            total_millis: 1000,
            current_app: "app".to_string(),
            per_app_millis: vec![("app".to_string(), 1000)].into_iter().collect(),
            tracking_enabled: true,
            hide_on_close: false,
            day_key: yesterday,
            processed_hours: 0,
            five_hour_alert_sent: false,
            hourly_notifications_sent: HashSet::new(),
            hourly_notifications_enabled: true,
            notifications: vec![],
        };

        let changed = ensure_daily_rollover(&state);
        assert!(changed);
        // After rollover, day_key should be today
        assert_eq!(state.day_key, current_day_key());
        // total_millis reset to 0
        assert_eq!(state.total_millis, 0);
        // per_app_millis cleared
        assert!(state.per_app_millis.is_empty());
        // processed_hours reset
        assert_eq!(state.processed_hours, 0);
        // five_hour_alert_sent reset
        assert_eq!(state.five_hour_alert_sent, false);
        // hourly_notifications_sent cleared
        assert!(state.hourly_notifications_sent.is_empty());
        // current_app set based on tracking_enabled (true) => "Waiting for user app"
        assert_eq!(state.current_app, "Waiting for user app");
        // tracking_enabled unchanged
        assert_eq!(state.tracking_enabled, true);
        // hide_on_close unchanged
        assert_eq!(state.hide_on_close, false);
        // hourly_notifications_enabled unchanged
        assert_eq!(state.hourly_notifications_enabled, true);
    }

    #[test]
    fn test_millis_to_seconds() {
        assert_eq!(millis_to_seconds(0), 0);
        assert_eq!(millis_to_seconds(1000), 1);
        assert_eq!(millis_to_seconds(1500), 1);
        assert_eq!(millis_to_seconds(2000), 2);
    }

    #[test]
    fn test_snapshot_from_state() {
        let mut state = TrackerState::default();
        state.total_millis = 3600000; // 1 hour
        state.current_app = "AppA".to_string();
        state.per_app_millis.insert("AppA".to_string(), 1800000); // 30 minutes
        state.per_app_millis.insert("AppB".to_string(), 1200000); // 20 minutes
        state.per_app_millis.insert("AppC".to_string(), 600000); // 10 minutes
        state.tracking_enabled = true;

        let snap = snapshot_from_state(&state);

        assert_eq!(snap.total_seconds, 3600); // 1 hour = 3600 seconds
        assert_eq!(snap.current_app, "AppA");
        assert_eq!(snap.tracking_enabled, true);
        assert_eq!(snap.apps.len(), 3); // we have three entries, truncated to 8 so all three

        // Check that the apps are sorted by seconds descending
        let mut seconds_vec: Vec<u64> = snap.apps.iter().map(|e| e.seconds).collect();
        seconds_vec.sort_by(|a, b| b.cmp(a));
        assert_eq!(seconds_vec, vec![1800, 1200, 600]); // 30m, 20m, 10m in seconds

        // Check percentages: total 3600 seconds
        // AppA: 1800/3600 = 0.5 => 50%
        // AppB: 1200/3600 = 0.333... => 33.333...
        // AppC: 600/3600 = 0.1666... => 16.666...
        let mut found_a = false;
        let mut found_b = false;
        let mut found_c = false;
        for app in &snap.apps {
            match app.name.as_str() {
                "AppA" => {
                    assert!((app.percent - 50.0).abs() < 0.001);
                    found_a = true;
                }
                "AppB" => {
                    assert!((app.percent - 100.0 * 1200.0 / 3600.0).abs() < 0.001);
                    found_b = true;
                }
                "AppC" => {
                    assert!((app.percent - 100.0 * 600.0 / 3600.0).abs() < 0.001);
                    found_c = true;
                }
                _ => panic!("unexpected app"),
            }
        }
        assert!(found_a && found_b && found_c);
    }

    #[test]
    fn test_strip_v_prefix() {
        assert_eq!(strip_v_prefix("v1.2.3"), "1.2.3");
        assert_eq!(strip_v_prefix("1.2.3"), "1.2.3");
        assert_eq!(strip_v_prefix("v"), "");
        assert_eq!(strip_v_prefix(""), "");
    }

    #[test]
    fn test_version_tuple() {
        assert_eq!(version_tuple("1.2.3"), vec![1, 2, 3]);
        assert_eq!(version_tuple("v1.2.3"), vec![1, 2, 3]);
        assert_eq!(version_tuple("0.0.1"), vec![0, 0, 1]);
        assert_eq!(version_tuple("10.20.30"), vec![10, 20, 30]);
        assert_eq!(version_tuple("1.2"), vec![1, 2, 0]); // missing patch => 0
        assert_eq!(version_tuple("1"), vec![1, 0, 0]); // missing minor and patch => 0
        assert_eq!(version_tuple(""), vec![0, 0, 0]); // empty => 0,0,0
    }

    #[test]
    fn test_is_newer() {
        assert!(is_newer("2.0.0", "1.9.9"));
        assert!(!is_newer("1.9.9", "2.0.0"));
        assert!(is_newer("v2.0.0", "1.9.9"));
        assert!(!is_newer("1.9.9", "v2.0.0"));
        assert!(!is_newer("1.0.0", "1.0.0"));
        assert!(is_newer("1.0.1", "1.0.0")); // 1.0.1 > 1.0.0 => true
        assert!(!is_newer("1.0.0", "1.0.1"));
    }
}