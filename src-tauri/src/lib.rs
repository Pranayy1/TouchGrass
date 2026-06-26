use serde::{Serialize, Deserialize};
use std::{
    collections::{BTreeSet, HashMap},
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use tauri::{
    menu::{Menu, MenuItem},
    webview::WebviewWindowBuilder,
    tray::TrayIconBuilder,
    AppHandle, Emitter, LogicalSize, Manager, State, WindowEvent, WebviewUrl,
};
use tauri_plugin_autostart::ManagerExt as AutostartExt;
use tauri_plugin_notification::NotificationExt;
use chrono::Local;

#[cfg(target_os = "windows")]
use windows_sys::Win32::{
    Foundation::{CloseHandle, HWND},
    System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
        PROCESS_QUERY_LIMITED_INFORMATION,
    },
    UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId},
};

#[derive(Clone, Serialize, Deserialize)]
struct NotificationEntry {
    id: String,
    title: String,
    message: String,
    timestamp: i64,
    #[serde(default)]
    read_at: Option<i64>,
}

impl Default for NotificationEntry {
    fn default() -> Self {
        Self {
            id: String::new(),
            title: String::new(),
            message: String::new(),
            timestamp: 0,
            read_at: None,
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct TrackerState {
    total_millis: u64,
    current_app: String,
    per_app_millis: HashMap<String, u64>,
    tracking_enabled: bool,
    hide_on_close: bool,
    day_key: String,
    processed_hours: u64,
    five_hour_alert_sent: bool,
    hourly_notifications_sent: BTreeSet<u64>,
    #[serde(default = "default_hourly_notifications_enabled")]
    hourly_notifications_enabled: bool,
    #[serde(default)]
    notifications: Vec<NotificationEntry>,
}

fn default_hourly_notifications_enabled() -> bool {
    true
}

#[derive(Clone)]
struct TrackerHandle(Arc<Mutex<TrackerState>>);

#[derive(Clone, Serialize)]
struct UsageEntry {
    name: String,
    seconds: u64,
    percent: f64,
}

#[derive(Clone, Serialize)]
struct UsageSnapshot {
    total_seconds: u64,
    current_app: String,
    top_app: String,
    tracking_enabled: bool,
    apps: Vec<UsageEntry>,
}

#[derive(Clone, Serialize)]
struct TrackingStatus {
    tracking_enabled: bool,
}

#[derive(Clone, Serialize)]
struct StartupStatus {
    enabled: bool,
}

#[derive(Clone, Serialize)]
struct CloseBehaviorStatus {
    hide_on_close: bool,
}

#[derive(Clone, Serialize)]
struct HourlyNotificationsStatus {
    enabled: bool,
}

#[derive(Clone, Serialize)]
struct UsageAlert {
    level: String,
    message: String,
    total_hours: u64,
    total_seconds: u64,
}

fn current_day_key() -> String {
    Local::now().date_naive().format("%Y-%m-%d").to_string()
}

fn reset_daily_state(state: &mut TrackerState) {
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

fn ensure_daily_rollover(state: &mut TrackerState) -> bool {
    let today = current_day_key();
    if state.day_key != today {
        reset_daily_state(state);
        return true;
    }

    false
}

#[tauri::command]
fn get_usage_snapshot(state: State<'_, TrackerHandle>) -> UsageSnapshot {
    match state.0.lock() {
        Ok(guard) => snapshot_from_state(&guard),
        Err(poisoned) => {
            eprintln!("Tracker state mutex poisoned, recovering: {:?}", poisoned);
            snapshot_from_state(&poisoned.into_inner())
        }
    }
}

#[tauri::command]
fn set_tracking_enabled(
    enabled: bool,
    state: State<'_, TrackerHandle>,
    app: AppHandle,
) -> TrackingStatus {
    set_tracking_enabled_internal(&state.0, enabled, &app)
}

#[tauri::command]
fn get_tracking_status(state: State<'_, TrackerHandle>) -> TrackingStatus {
    let enabled = state
        .0
        .lock()
        .map(|s| s.tracking_enabled)
        .unwrap_or(false);

    TrackingStatus {
        tracking_enabled: enabled,
    }
}

#[tauri::command]
fn hide_to_tray(app: AppHandle) {
    hide_main_window(&app);
}

#[tauri::command]
fn show_main_window(app: AppHandle) {
    reveal_main_window(&app);
}

#[tauri::command]
async fn show_focus_popup(app: AppHandle, remaining_seconds: u32) -> tauri::Result<()> {
    let remaining = remaining_seconds.max(1);

    if let Some(window) = app.get_webview_window("focus-popup") {
        window.set_always_on_top(true)?;
        window.set_maximizable(false)?;
        window.set_shadow(false)?;
        window.set_size(LogicalSize::new(172.0, 44.0))?;
        window.show()?;
        window.unminimize()?;
        window.set_focus()?;
        return Ok(());
    }

    let initialization_script = format!("window.__FOCUS_POPUP_REMAINING__ = {};", remaining);

    let popup =
        WebviewWindowBuilder::new(&app, "focus-popup", WebviewUrl::App("popup.html".into()))
            .title("Focus")
            .decorations(false)
            .transparent(true)
            .shadow(false)
            .resizable(false)
            .maximizable(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .focused(true)
            .inner_size(172.0, 44.0)
            .initialization_script(initialization_script)
            .build()?;

    popup.set_focus()?;
    Ok(())
}

#[tauri::command]
fn close_focus_popup(app: AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("focus-popup") {
        window.close()?;
    }

    Ok(())
}

#[tauri::command]
fn quit_app(app: AppHandle) {
    app.exit(0);
}

#[tauri::command]
fn get_launch_on_startup(app: AppHandle) -> StartupStatus {
    let enabled = app
        .autolaunch()
        .is_enabled()
        .unwrap_or(false);
    StartupStatus { enabled }
}

#[tauri::command]
fn set_launch_on_startup(enabled: bool, app: AppHandle) -> StartupStatus {
    let manager = app.autolaunch();
    let result = if enabled {
        manager.enable()
    } else {
        manager.disable()
    };

    if let Err(error) = result {
        eprintln!("failed to update autostart: {error}");
    }

    let current = manager.is_enabled().unwrap_or(false);
    StartupStatus { enabled: current }
}

#[tauri::command]
fn get_close_behavior(state: State<'_, TrackerHandle>) -> CloseBehaviorStatus {
    let hide_on_close = state
        .0
        .lock()
        .map(|s| s.hide_on_close)
        .unwrap_or(true);

    CloseBehaviorStatus { hide_on_close }
}

#[tauri::command]
fn set_hide_on_close(
    enabled: bool,
    state: State<'_, TrackerHandle>,
    app: AppHandle,
) -> CloseBehaviorStatus {
    if let Ok(mut tracker) = state.0.lock() {
        tracker.hide_on_close = enabled;

        let _ = save_state(&tracker, &app);
    }

    CloseBehaviorStatus {
        hide_on_close: enabled,
    }
}

#[tauri::command]
fn get_hourly_notifications(state: State<'_, TrackerHandle>) -> HourlyNotificationsStatus {
    let enabled = state
        .0
        .lock()
        .map(|s| s.hourly_notifications_enabled)
        .unwrap_or(true);

    HourlyNotificationsStatus { enabled }
}

#[tauri::command]
fn set_hourly_notifications(
    enabled: bool,
    state: State<'_, TrackerHandle>,
    app: AppHandle,
) -> HourlyNotificationsStatus {
    if let Ok(mut tracker) = state.0.lock() {
        tracker.hourly_notifications_enabled = enabled;
        let _ = save_state(&tracker, &app);
    }

    HourlyNotificationsStatus { enabled }
}

#[tauri::command]
fn get_notifications(state: State<'_, TrackerHandle>) -> Vec<NotificationEntry> {
    let Ok(tracker) = state.0.lock() else {
        return Vec::new();
    };

    let mut notifications = tracker.notifications.clone();
    notifications.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    notifications
}

#[tauri::command]
fn delete_notification(id: String, state: State<'_, TrackerHandle>, app: AppHandle) {
    let Ok(mut tracker) = state.0.lock() else {
        return;
    };

    tracker.notifications.retain(|n| n.id != id);
    let _ = save_state(&tracker, &app);
}

#[tauri::command]
fn clear_notifications(state: State<'_, TrackerHandle>, app: AppHandle) {
    let Ok(mut tracker) = state.0.lock() else {
        return;
    };

    tracker.notifications.clear();
    let _ = save_state(&tracker, &app);
}

#[tauri::command]
fn timer_completed(
    minutes: u32,
    state: State<'_, TrackerHandle>,
    app: AppHandle,
) {
    if minutes == 0 {
        return;
    }

    notify_and_store(
        &state.0,
        &app,
        "Focus Timer Completed",
        &format!(
            "You successfully completed a {} minute focus session.",
            minutes
        ),
    );
}

#[tauri::command]
fn mark_notification_read(id: String, state: State<'_, TrackerHandle>, app: AppHandle) {
    let Ok(mut tracker) = state.0.lock() else {
        return;
    };

    let now = chrono::Utc::now().timestamp();
    let mut changed = false;
    for notification in &mut tracker.notifications {
        if notification.id == id && notification.read_at.is_none() {
            notification.read_at = Some(now);
            changed = true;
        }
    }

    if !changed {
        return;
    }

    let _ = save_state(&tracker, &app);
    let _ = app.emit("notification://read", ());
}

#[tauri::command]
fn mark_all_notifications_read(state: State<'_, TrackerHandle>, app: AppHandle) {
    let Ok(mut tracker) = state.0.lock() else {
        return;
    };

    let now = chrono::Utc::now().timestamp();
    let mut changed = false;
    for notification in &mut tracker.notifications {
        if notification.read_at.is_none() {
            notification.read_at = Some(now);
            changed = true;
        }
    }

    if !changed {
        return;
    }

    let _ = save_state(&tracker, &app);
    let _ = app.emit("notification://read", ());
}

#[derive(Deserialize, Debug)]
struct GitHubRelease {
    tag_name: String,
    body: String,
    html_url: String,
}

#[derive(Clone, Serialize)]
struct UpdateInfo {
    version: String,
    notes: String,
    release_url: String,
}

fn strip_v_prefix(version: &str) -> String {
    version.strip_prefix('v').unwrap_or(version).to_string()
}

fn version_tuple(version: &str) -> Vec<u32> {
    strip_v_prefix(version)
        .split('.')
        .map(|s| s.parse::<u32>().unwrap_or(0))
        .collect()
}

fn is_newer(latest: &str, current: &str) -> bool {
    version_tuple(latest) > version_tuple(current)
}

#[tauri::command]
async fn check_for_updates(app: AppHandle) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .user_agent("TouchGrass")
        .build()
        .map_err(|e| format!("HTTP client build failed: {e}"))?;

    let release = client
        .get("https://api.github.com/repos/Pranayy1/TouchGrass/releases/latest")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("GitHub API returned error: {e}"))?
        .json::<GitHubRelease>()
        .await
        .map_err(|e| format!("JSON parse failed: {e}"))?;

    let current = env!("CARGO_PKG_VERSION");
    if is_newer(&release.tag_name, current) {
        let info = UpdateInfo {
            version: strip_v_prefix(&release.tag_name),
            notes: release.body,
            release_url: release.html_url,
        };
        let _ = app.emit("update://available", info);
    } else {
        let _ = app.emit("update://checked", ());
    }

    Ok(())
}

fn millis_to_seconds(ms: u64) -> u64 {
    ms / 1000
}

fn snapshot_from_state(state: &TrackerState) -> UsageSnapshot {
    let total = millis_to_seconds(state.total_millis);
    let mut apps: Vec<UsageEntry> = state
        .per_app_millis
        .iter()
        .map(|(name, ms)| {
            let seconds = millis_to_seconds(*ms);
            UsageEntry {
                name: name.clone(),
                seconds,
                percent: if total == 0 {
                    0.0
                } else {
                    (seconds as f64 / total as f64) * 100.0
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
        total_seconds: total,
        current_app: state.current_app.clone(),
        top_app,
        tracking_enabled: state.tracking_enabled,
        apps,
    }
}

fn is_running_in_tray(app: &AppHandle) -> bool {
    if let Some(window) = app.get_webview_window("main") {
        return window.is_visible().map(|visible| !visible).unwrap_or(true);
    }

    true
}

fn send_system_notification(app: &AppHandle, title: &str, body: &str) {
    let _ = app
        .notification()
        .builder()
        .title(title)
        .body(body)
        .show();
}

fn notify_and_store(
    tracker: &Arc<Mutex<TrackerState>>,
    app: &AppHandle,
    title: &str,
    message: &str,
) {
    let entry = NotificationEntry {
        id: uuid::Uuid::new_v4().to_string(),
        title: title.to_string(),
        message: message.to_string(),
        timestamp: chrono::Utc::now().timestamp(),
        read_at: None,
    };

    if let Ok(mut state) = tracker.lock() {
        state.notifications.push(entry.clone());
        if let Err(error) = save_state(&state, app) {
            eprintln!("failed to save notification: {error}");
        }
        let _ = app.emit("notification://added", entry);
    }

    send_system_notification(app, title, message);
}

fn process_usage_alerts(
    state: &mut TrackerState,
    app: AppHandle,
    tracker: Arc<Mutex<TrackerState>>,
) {
    let total_seconds = millis_to_seconds(state.total_millis);
    let completed_hours = total_seconds / 3600;

    if completed_hours > state.processed_hours {
        let tray_mode = is_running_in_tray(&app);

        for hour in (state.processed_hours + 1)..=completed_hours {
            if state.hourly_notifications_enabled
                && tray_mode
                && !state.hourly_notifications_sent.contains(&hour)
            {
                notify_and_store(
                    &tracker,
                    &app,
                    "TouchGrass Hourly Usage",
                    &format!("You have used your PC for {} hour{} today.", hour, if hour == 1 { "" } else { "s" }),
                );
                state.hourly_notifications_sent.insert(hour);
            }
        }

        state.processed_hours = completed_hours;
    }

    if total_seconds >= 5 * 3600 && !state.five_hour_alert_sent {
        state.five_hour_alert_sent = true;

        let alert = UsageAlert {
            level: "critical".to_string(),
            message: "Alert: You have used your PC for 5 hours today. Please take a break.".to_string(),
            total_hours: completed_hours,
            total_seconds,
        };

        let _ = app.emit("usage://alert", alert.clone());
        notify_and_store(&tracker, &app, "TouchGrass Usage Alert", &alert.message);
    }
}

fn track_foreground_apps(shared: Arc<Mutex<TrackerState>>, app: AppHandle) {
    thread::spawn(move || {
        let mut sampling_ms = 2500u64;
        let mut last_tick = Instant::now();
        let mut last_emit = Instant::now();
        let mut last_save = Instant::now();
        let mut last_app = String::new();
        let mut tick_count = 0u32;

        loop {
            thread::sleep(Duration::from_millis(sampling_ms));
            let now = Instant::now();
          let elapsed = now.saturating_duration_since(last_tick).as_millis() as u64;
last_tick = now;

if elapsed > 30_000 {
    continue;
}

tick_count += 1;

            // Only call expensive Windows API every 3rd tick, reuse last_app for time accumulation
            let detected_active = if tick_count % 3 == 0 {
                detect_active_user_app()
            } else {
                None
            };

            if let Ok(mut state) = shared.lock() {
                let rolled_over = ensure_daily_rollover(&mut state);
                if rolled_over {
                    last_app.clear();
                }

                if !state.tracking_enabled {
                    state.current_app = "Tracking paused".to_string();
                    sampling_ms = 8000;
                } else if let Some(active_name) = detected_active.or_else(|| {
                    if last_app.is_empty() {
                        None
                    } else {
                        Some(last_app.clone())
                    }
                }) {
                    let changed = rolled_over || active_name != last_app;
                    let credited_app = if changed && !last_app.is_empty() {
                        last_app.clone()
                    } else {
                        active_name.clone()
                    };

                    if active_name != last_app {
                        last_app = active_name.clone();
                        sampling_ms = 1200;
                        tick_count = 0;
                    } else {
                        sampling_ms = (sampling_ms + 180).min(5000);
                    }

                    state.current_app = active_name.clone();
                    if !credited_app.is_empty() {
                        state.total_millis += elapsed;
                        let counter = state.per_app_millis.entry(credited_app).or_insert(0);
                        *counter += elapsed;
                    }
                    process_usage_alerts(&mut state, app.clone(), shared.clone());
                  

                    if changed || last_emit.elapsed() >= Duration::from_secs(8) {
                        let snapshot = snapshot_from_state(&state);
                        let _ = app.emit("usage://snapshot", snapshot);
                        last_emit = Instant::now();
                    }
                } else if tick_count % 3 == 0 {
                    state.current_app = "No tracked user app".to_string();
                    sampling_ms = (sampling_ms + 400).min(7000);
                }

                // Save state every 30 seconds
                if last_save.elapsed() >= Duration::from_secs(30) {
                    let _ = save_state(&state, &app);
                    last_save = Instant::now();
                }
            }
        }
    });
}

fn set_tracking_enabled_internal(
    tracker: &Arc<Mutex<TrackerState>>,
    enabled: bool,
    app: &AppHandle,
) -> TrackingStatus {
    if let Ok(mut state) = tracker.lock() {
        ensure_daily_rollover(&mut state);
        state.tracking_enabled = enabled;
        if !enabled {
            state.current_app = "Tracking paused".to_string();
        }

        let _ = app.emit(
            "tracking://status",
            TrackingStatus {
                tracking_enabled: enabled,
            },
        );
        let _ = app.emit("usage://snapshot", snapshot_from_state(&state));
        let _ = save_state(&state, app);
    }

    TrackingStatus {
        tracking_enabled: enabled,
    }
}

fn hide_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

fn reveal_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn reset_usage_data(tracker: &Arc<Mutex<TrackerState>>, app: &AppHandle) {
    if let Ok(mut state) = tracker.lock() {
        reset_daily_state(&mut state);
        let _ = app.emit("usage://reset", ());
        let _ = app.emit("usage://snapshot", snapshot_from_state(&state));
        let _ = save_state(&state, app);
    }
}

fn setup_main_window_behavior(app: &AppHandle, tracker: Arc<Mutex<TrackerState>>) {
    if let Some(window) = app.get_webview_window("main") {
        let app_handle = app.clone();
        let tracker_state = tracker.clone();
        window.on_window_event(move |event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let should_hide = tracker_state
                    .lock()
                    .map(|state| state.hide_on_close)
                    .unwrap_or(true);

                if should_hide {
                    api.prevent_close();
                    hide_main_window(&app_handle);
                }
            }
        });
    }
}

fn get_state_file_path(app: &AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("./data"))
        .join("touchgrass_state.json")
}

fn save_state(state: &TrackerState, app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let path = get_state_file_path(app);
    
    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    let json = serde_json::to_string_pretty(state)?;
    fs::write(&path, json)?;
    Ok(())
}

fn load_state(app: &AppHandle) -> Result<TrackerState, Box<dyn std::error::Error>> {
    let path = get_state_file_path(app);
    
    if !path.exists() {
        let mut state = TrackerState::default();
        state.hide_on_close = true;
        state.tracking_enabled = true;
        state.hourly_notifications_enabled = true;
        reset_daily_state(&mut state);
        return Ok(state);
    }
    
    let json = fs::read_to_string(&path)?;
    let mut state: TrackerState = serde_json::from_str(&json)?;

    let now = chrono::Utc::now().timestamp();
    let max_age = 86400;
    let original_len = state.notifications.len();
    state.notifications
        .retain(|n| now - n.timestamp <= max_age);
    if state.notifications.len() != original_len {
        save_state(&state, app)?;
    }

    let calculated_total: u64 =
    state.per_app_millis.values().sum();

if state.total_millis != calculated_total {
    eprintln!(
        "Repairing corrupted total_millis: {} -> {}",
        state.total_millis,
        calculated_total
    );

    state.total_millis = calculated_total;
}
    if state.day_key.is_empty() {
        reset_daily_state(&mut state);
    } else {
        ensure_daily_rollover(&mut state);
    }

    let hours_from_total = millis_to_seconds(state.total_millis) / 3600;
    state.processed_hours = state.processed_hours.max(hours_from_total);

    Ok(state)
}

fn setup_tray(app: &AppHandle, tracker: Arc<Mutex<TrackerState>>) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "tray_show", "Open TouchGrass", true, None::<&str>)?;
    let stop_tracking =
        MenuItem::with_id(app, "tray_stop_tracking", "Stop Tracking", true, None::<&str>)?;
    let start_tracking =
        MenuItem::with_id(app, "tray_start_tracking", "Start Tracking", true, None::<&str>)?;
    let reset_data = MenuItem::with_id(app, "tray_reset_data", "Reset Usage Data", true, None::<&str>)?;
    let close = MenuItem::with_id(app, "tray_close", "Close App", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[&show, &stop_tracking, &start_tracking, &reset_data, &close],
    )?;

    let tracker_for_menu = tracker.clone();
    
 let mut builder = TrayIconBuilder::new()
    .menu(&menu)
    .tooltip("TouchGrass")
    .on_tray_icon_event(|tray, event| {
        if let tauri::tray::TrayIconEvent::Click {
            button: tauri::tray::MouseButton::Left, ..
        } = event {
            reveal_main_window(tray.app_handle());
        }
    })
    .on_menu_event(move |app, event| match event.id().as_ref() {
        "tray_show" => reveal_main_window(app),
        "tray_stop_tracking" => {
            let _ = set_tracking_enabled_internal(&tracker_for_menu, false, app);
        }
        "tray_start_tracking" => {
            let _ = set_tracking_enabled_internal(&tracker_for_menu, true, app);
        }
        "tray_reset_data" => {
            reset_usage_data(&tracker_for_menu, app);
        }
        "tray_close" => app.exit(0),
        _ => {}
    });

if let Some(icon) = app.default_window_icon().cloned() {
    builder = builder.icon(icon);
}
builder.build(app)?;

    Ok(())
}

#[cfg(target_os = "windows")]
fn detect_active_user_app() -> Option<String> {
    let hwnd: HWND = unsafe { GetForegroundWindow() };
    if hwnd.is_null() {
        return None;
    }

    let mut pid: u32 = 0;
    unsafe {
        GetWindowThreadProcessId(hwnd, &mut pid);
    }
    if pid == 0 {
        return None;
    }

    let path = get_process_image_path(pid)?;
    let app = extract_app_name(&path)?;

    if is_system_app(&app, &path) {
        return None;
    }

    Some(app)
}

#[cfg(target_os = "windows")]
fn get_process_image_path(pid: u32) -> Option<String> {
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if handle.is_null() {
        return None;
    }

    let mut size: u32 = 1024;
    let mut buffer = vec![0u16; size as usize];

    let ok = unsafe {
        QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            buffer.as_mut_ptr(),
            &mut size,
        )
    };

    unsafe {
        CloseHandle(handle);
    }

    if ok == 0 || size == 0 {
        return None;
    }

    let path = String::from_utf16_lossy(&buffer[..size as usize]);
    Some(path)
}

#[cfg(target_os = "windows")]
fn extract_app_name(path: &str) -> Option<String> {
    let name = path
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or(path)
        .trim_end_matches(".exe")
        .trim();

    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

#[cfg(target_os = "windows")]
fn is_system_app(app_name: &str, path: &str) -> bool {
    let app = app_name.to_ascii_lowercase();
    let p = path.to_ascii_lowercase();

    let blocked = [
        "taskmgr",
        "dwm",
        "idle",
        "system",
        "svchost",
        "explorer",
        "sihost",
        "searchhost",
        "searchapp",
        "startmenuexperiencehost",
        "textinputhost",
        "runtimebroker",
        "ctfmon",
        "fontdrvhost",
        "applicationframehost",
        "lockapp",
        "shellexperiencehost",
    ];

    if blocked.contains(&app.as_str()) {
        return true;
    }

    p.starts_with("c:\\windows\\") || p.contains("\\windowsapps\\")
}

#[cfg(not(target_os = "windows"))]
fn detect_active_user_app() -> Option<String> {
    None
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let tracker = Arc::new(Mutex::new(TrackerState {
        tracking_enabled: true,
        hide_on_close: true,
        day_key: current_day_key(),
        ..Default::default()
    }));

    tauri::Builder::default()
        .manage(TrackerHandle(tracker.clone()))
        .setup(|app| {
            let handle = app.handle().clone();
            let tracker = app.state::<TrackerHandle>().0.clone();

            if let Ok(mut state) = tracker.lock() {
                if let Ok(loaded_state) = load_state(&handle) {
                    *state = loaded_state;
                }

                state.current_app = if state.tracking_enabled {
                    "Waiting for user app".to_string()
                } else {
                    "Tracking paused".to_string()
                };
            }

            setup_main_window_behavior(&handle, tracker.clone());
            setup_tray(&handle, tracker.clone())?;
            track_foreground_apps(tracker, handle.clone());

            Ok(())
        })
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_usage_snapshot,
            get_tracking_status,
            set_tracking_enabled,
            hide_to_tray,
            show_main_window,
            show_focus_popup,
            close_focus_popup,
            quit_app,
            get_launch_on_startup,
            set_launch_on_startup,
            get_close_behavior,
            set_hide_on_close,
            get_hourly_notifications,
            set_hourly_notifications,
            get_notifications,
            delete_notification,
            clear_notifications,
            timer_completed,
            mark_notification_read,
            mark_all_notifications_read,
            check_for_updates
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
