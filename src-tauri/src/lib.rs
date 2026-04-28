use serde::Serialize;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager, State, WindowEvent,
};
use tauri_plugin_autostart::ManagerExt as AutostartExt;

#[cfg(target_os = "windows")]
use windows_sys::Win32::{
    Foundation::{CloseHandle, HWND},
    System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
        PROCESS_QUERY_LIMITED_INFORMATION,
    },
    UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId},
};

#[derive(Default)]
struct TrackerState {
    total_millis: u64,
    current_app: String,
    per_app_millis: HashMap<String, u64>,
    tracking_enabled: bool,
    hide_on_close: bool,
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

#[tauri::command]
fn get_usage_snapshot(state: State<'_, TrackerHandle>) -> UsageSnapshot {
    let guard = match state.0.lock() {
        Ok(g) => g,
        Err(_) => {
            return UsageSnapshot {
                total_seconds: 0,
                current_app: "Unknown".to_string(),
                top_app: "Unknown".to_string(),
                tracking_enabled: false,
                apps: vec![],
            }
        }
    };

    let total = millis_to_seconds(guard.total_millis);
    let mut items: Vec<UsageEntry> = guard
        .per_app_millis
        .iter()
        .map(|(name, seconds)| UsageEntry {
            name: name.clone(),
            seconds: millis_to_seconds(*seconds),
            percent: if total == 0 {
                0.0
            } else {
                (millis_to_seconds(*seconds) as f64 / total as f64) * 100.0
            },
        })
        .collect();

    items.sort_by(|a, b| b.seconds.cmp(&a.seconds));
    items.truncate(8);

    let top_app = items
        .first()
        .map(|entry| entry.name.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    UsageSnapshot {
        total_seconds: total,
        current_app: guard.current_app.clone(),
        top_app,
        tracking_enabled: guard.tracking_enabled,
        apps: items,
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
fn set_hide_on_close(enabled: bool, state: State<'_, TrackerHandle>) -> CloseBehaviorStatus {
    if let Ok(mut tracker) = state.0.lock() {
        tracker.hide_on_close = enabled;
    }

    CloseBehaviorStatus {
        hide_on_close: enabled,
    }
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

fn track_foreground_apps(shared: Arc<Mutex<TrackerState>>, app: AppHandle) {
    thread::spawn(move || {
        let mut sampling_ms = 1800u64;
        let mut last_tick = Instant::now();
        let mut last_emit = Instant::now();
        let mut last_app = String::new();

        loop {
            thread::sleep(Duration::from_millis(sampling_ms));
            let now = Instant::now();
            let elapsed = now.saturating_duration_since(last_tick).as_millis() as u64;
            last_tick = now;

            let active = detect_active_user_app();
            let mut changed = false;
            let mut should_emit = false;

            if let Ok(mut state) = shared.lock() {
                if !state.tracking_enabled {
                    state.current_app = "Tracking paused".to_string();
                    sampling_ms = 7000;
                } else if let Some(active_name) = active {
                    if active_name != last_app {
                        changed = true;
                        last_app = active_name.clone();
                        sampling_ms = 900;
                    } else {
                        sampling_ms = (sampling_ms + 250).min(4500);
                    }

                    state.current_app = active_name.clone();
                    state.total_millis += elapsed;
                    let counter = state.per_app_millis.entry(active_name).or_insert(0);
                    *counter += elapsed;
                } else {
                    state.current_app = "No tracked user app".to_string();
                    sampling_ms = (sampling_ms + 350).min(6500);
                }

                if changed || last_emit.elapsed() >= Duration::from_secs(5) {
                    let snapshot = snapshot_from_state(&state);
                    let _ = app.emit("usage://snapshot", snapshot);
                    last_emit = Instant::now();
                    should_emit = true;
                }
            }

            if !should_emit && last_emit.elapsed() >= Duration::from_secs(10) {
                if let Ok(state) = shared.lock() {
                    let snapshot = snapshot_from_state(&state);
                    let _ = app.emit("usage://snapshot", snapshot);
                    last_emit = Instant::now();
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
        state.total_millis = 0;
        state.per_app_millis.clear();
        state.current_app = if state.tracking_enabled {
            "Waiting for user app".to_string()
        } else {
            "Tracking paused".to_string()
        };
        let _ = app.emit("usage://snapshot", snapshot_from_state(&state));
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

fn setup_tray(app: &AppHandle, tracker: Arc<Mutex<TrackerState>>) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "tray_show", "Open Arise", true, None::<&str>)?;
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
    TrayIconBuilder::new()
        .menu(&menu)
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
        })
        .build(app)?;

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
        ..Default::default()
    }));

    tauri::Builder::default()
        .manage(TrackerHandle(tracker.clone()))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle().clone();
            let tracker = app.state::<TrackerHandle>().0.clone();

            setup_main_window_behavior(&handle, tracker.clone());
            setup_tray(&handle, tracker.clone())?;
            track_foreground_apps(tracker, handle);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_usage_snapshot,
            get_tracking_status,
            set_tracking_enabled,
            hide_to_tray,
            show_main_window,
            quit_app,
            get_launch_on_startup,
            set_launch_on_startup,
            get_close_behavior,
            set_hide_on_close
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
