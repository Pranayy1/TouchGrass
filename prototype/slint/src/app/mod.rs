use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// Define the struct for dashboard app data to match Slint definition
slint::slint! {
    struct DashboardDataApp {
        name: string,
        usage: string,
        progress: float
    }
}

impl App {
    /// Create a new application instance.
    ///
    /// This method will initialize the tracker service and tray service.
    /// It will not show the window or start the event loop.
    pub fn new() -> Self {
        // Create tracker service with optional storage
        let storage = None;
        let tracker_service = TrackerService::new(None, storage);

        // Create command channel for tray communication
        let (command_tx, command_rx) = mpsc::channel();

        // Create and start tray service
        let tray_service = PlatformTrayService::new(command_tx);
        tray_service.start();

        Self {
            tracker_service,
            tray_service: Some(tray_service),
            command_rx,
            should_quit: Arc::new(Mutex::new(false)),
        }
    }

    /// Run the application: show the window and enter the Slint event loop.
    ///
    /// This method will block until the user closes the window.
    /// After the event loop exits, the tracker service is stopped to ensure
    /// final state persistence.
    pub fn run(mut self) {
        // Import and instantiate the Slint components
        let window = MainWindow::new().unwrap();
        window.show().unwrap();

        // Get references to the dashboard for callbacks
        let dashboard = window.global::<Dashboard>();

        // Get references to the settings for callbacks
        let settings = window.global::<Settings>();

        // Set up close-requested handler for hide-on-close behavior
        let tracker_state = self.tracker_service.state();
        let window_weak = window.as_weak();
        let should_quit_clone = self.should_quit.clone();

        // Try to get the underlying Window object
        let window_obj = window.window();
        window_obj.on_close_requested(move || {
            // Get the current tracker state to check hide_on_close preference
            let state = tracker_state.lock().unwrap();
            let hide_on_close = state.hide_on_close;
            drop(state); // Release the lock

            if hide_on_close {
                // Hide the window but keep the application running
                if let Some(window) = window_weak.clone().upgrade() {
                    window.hide().unwrap();
                }
                // Return KeepWindowShown to prevent the window from being destroyed
                // and keep the event loop running
                slint::CloseRequestResponse::KeepWindowShown
            } else {
                // Allow the window to close and quit the application
                *should_quit_clone.lock().unwrap() = true;
                slint::CloseRequestResponse::HideWindow
            }
        });

        // Set up the dashboard data callback
        if let Some(dashboard) = dashboard {
            let tracker_state = self.tracker_service.state();
            dashboard.on_get_data(move || {
                // Get the current tracker state
                let state = tracker_state.lock().unwrap();

                // Calculate today's usage from total_millis
                let total_seconds = (state.total_millis / 1000) as u64;
                let hours = total_seconds / 3600;
                let minutes = (total_seconds % 3600) / 60;
                let today_usage = format!("{}h {}m", hours, minutes);

                // Focus Time: Not genuinely available in TrackerState
                // The original implementation had focus timer functionality but
                // did not persistently track accumulated focus time
                let focus_time = "--".to_string();

                // Sessions: Not genuinely available as "completed sessions" in TrackerState
                // The original implementation had focus popup/timer functionality but
                // did not persistently track completed focus sessions
                let sessions = "--".to_string();

                // Get top applications sorted by usage
                let mut apps: Vec<(&String, &u64)> = state.per_app_millis.iter().collect();
                apps.sort_by(|a, b| b.1.cmp(a.1)); // Sort by usage descending

                // Create apps data for display (max 4 apps)
                let mut apps_data = Vec::new();
                let mut app_minutes_vec = Vec::new();

                for (app_name, app_millis) in apps.iter().take(4) {
                    let app_seconds = (*app_millis / 1000) as u64;
                    let app_minutes = app_seconds / 60; // Convert to minutes for display
                    let usage_string = format!("{}m", app_minutes);

                    apps_data.push(DashboardDataApp {
                        name: (*app_name).clone().into(),
                        usage: usage_string.into(),
                        progress: app_minutes as float, // Actual usage in minutes
                    });
                    app_minutes_vec.push(app_minutes);
                }

                // Fill remaining slots with empty data if less than 4 apps
                while apps_data.len() < 4 {
                    apps_data.push(DashboardDataApp {
                        name: "".into(),
                        usage: "0m".into(),
                        progress: 0.0,
                    });
                    app_minutes_vec.push(0);
                }

                // Calculate maximum progress value for proportional scaling
                let max_progress = app_minutes_vec.into_iter().fold(0.0, |a, b| a.max(b));

                // Set the dashboard properties
                dashboard.set_today_usage(today_usage.into());
                dashboard.set_focus_time(focus_time.into());
                dashboard.set_sessions(sessions.into());
                dashboard.set_app1_name(apps_data[0].name.clone());
                dashboard.set_app1_usage(apps_data[0].usage.clone());
                dashboard.set_app1_progress(apps_data[0].progress);
                dashboard.set_app2_name(apps_data[1].name.clone());
                dashboard.set_app2_usage(apps_data[1].usage.clone());
                dashboard.set_app2_progress(apps_data[1].progress);
                dashboard.set_app3_name(apps_data[2].name.clone());
                dashboard.set_app3_usage(apps_data[2].usage.clone());
                dashboard.set_app3_progress(apps_data[2].progress);
                dashboard.set_app4_name(apps_data[3].name.clone());
                dashboard.set_app4_usage(apps_data[3].usage.clone());
                dashboard.set_app4_progress(apps_data[3].progress);
                dashboard.set_max_progress(max_progress);
            });
        }

        // Set up the settings data callback
        if let Some(settings) = settings {
            let tracker_state = self.tracker_service.state();
            settings.on_get_settings_data(move || {
                // Get the current tracker state
                let state = tracker_state.lock().unwrap();

                // Set the settings properties
                settings.set_tracking_enabled(state.tracking_enabled);
                settings.set_hide_on_close(state.hide_on_close);
                settings.set_hourly_notifications_enabled(state.hourly_notifications_enabled);
            });
        }

        // Clone values for the tray command handler thread
        let window_clone = window.as_weak();
        let should_quit = self.should_quit.clone();
        let command_rx = Arc::new(Mutex::new(self.command_rx));

        // Spawn a thread to handle tray commands
        let should_quit_clone = should_quit.clone();
        let command_rx_clone = command_rx.clone();
        thread::spawn(move || {
            let command_rx = command_rx_clone.lock().unwrap();
            loop {
                // Check for tray commands
                if let Ok(command) = command_rx.try_recv() {
                    match command {
                        ApplicationCommand::ShowWindow => {
                            // Show the window using Slint's thread-safe invocation
                            if let Some(window) = window_clone.clone().upgrade() {
                                window.show().unwrap();
                            }
                        }
                        ApplicationCommand::Quit => {
                            // Set the quit flag
                            *should_quit_clone.lock().unwrap() = true;
                            // Break out of the loop
                            break;
                        }
                    }
                }

                // Sleep briefly to avoid busy waiting
                thread::sleep(Duration::from_millis(100));
            }
        });

        // Run the Slint event loop (will exit when last window is closed)
        slint::run_event_loop().unwrap();

        // After the event loop exits, stop the tracker service to persist final state
        // and ensure the background thread exits cleanly.
        self.tracker_service.stop();

        // Stop tray service
        if let Some(tray_service) = self.tray_service.take() {
            tray_service.stop();
        }
    }
}