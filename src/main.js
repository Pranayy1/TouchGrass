const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const SECONDS_PER_MINUTE = 60;
const DAILY_GOAL_SECONDS = 8 * 60 * 60;
const ANALYTICS_STORAGE_KEY = "touchgrass_analytics_v2";
const FOCUS_TIMER_STORAGE_KEY = "touchgrass_focus_remaining_seconds";
const FOCUS_TIMER_RUNNING_STORAGE_KEY = "touchgrass_focus_is_running";
const ANALYTICS_WINDOW_DAYS = 7;
const DAILY_ALERT_THRESHOLD_SECONDS = 5 * 60 * 60;
const USAGE_FILL_CLASSES = ["code", "browser", "chat", "music"];
const LAST_UPDATE_CHECK_KEY = "last_update_check";
const UPDATE_CHECK_INTERVAL = 10 * 60 * 60 * 1000;
const FALLBACK_APPS = [
	{ name: "No data yet", seconds: 1, percent: 100 },
	{ name: "Keep app open", seconds: 0, percent: 0 },
	{ name: "Switch apps", seconds: 0, percent: 0 },
	{ name: "Data updates live", seconds: 0, percent: 0 }
];

document.addEventListener("DOMContentLoaded", async () => {
	console.log("App: DOMContentLoaded fired");

	document.addEventListener("dragstart", (event) => event.preventDefault());

	const isFocusPopupWindow = Boolean(window.__FOCUS_POPUP_MODE__);
	const injectedFocusMinutes = Number(window.__FOCUS_POPUP_MINUTES__);
	const injectedFocusRemaining = Number(window.__FOCUS_POPUP_REMAINING__);

	const dom = {
		actualTime: document.getElementById("actual-time"),
		greeting: document.getElementById("greeting-text"),
		heroHours: document.getElementById("hero-hours"),
		progressCircle: document.getElementById("progress-circle"),
		appUsageContainer: document.getElementById("app-usage-container"),
		topAppLabel: document.getElementById("top-app-label"),
		trackingStatusText: document.getElementById("tracking-status-text"),
		startupStatusText: document.getElementById("startup-status-text"),
		closeBehaviorStatusText: document.getElementById("close-behavior-status-text"),
		hourlyNotificationsStatusText: document.getElementById("hourly-notifications-status-text"),
		trackingToggleBtn: document.getElementById("tracking-toggle-btn"),
		startupToggleBtn: document.getElementById("startup-toggle-btn"),
		closeBehaviorToggleBtn: document.getElementById("close-behavior-toggle-btn"),
		hourlyNotificationsToggleBtn: document.getElementById("hourly-notifications-toggle-btn"),
		trackingSummaryText: document.getElementById("tracking-summary-text"),
		topAppInline: document.getElementById("top-app-inline"),
		trackingInline: document.getElementById("tracking-inline"),
		analyticsRangeLabel: document.getElementById("analytics-range-label"),
		analyticsTodayLabel: document.getElementById("analytics-today-label"),
		analyticsTodayDate: document.getElementById("analytics-today-date"),
		analyticsAverageTime: document.getElementById("analytics-average-time"),
		analyticsMostDay: document.getElementById("analytics-most-day"),
		analyticsMostDayTime: document.getElementById("analytics-most-day-time"),
		analyticsLeastDay: document.getElementById("analytics-least-day"),
		analyticsLeastDayTime: document.getElementById("analytics-least-day-time"),
		analyticsRecordCount: document.getElementById("analytics-record-count"),
		analyticsChart: document.getElementById("analytics-chart"),
		analyticsList: document.getElementById("analytics-list"),
		analyticsDays: document.getElementById("analytics-days"),
		musicAudio: document.getElementById("calm-audio"),
		musicToggleBtn: document.getElementById("music-toggle-btn"),
		musicStopBtn: document.getElementById("music-stop-btn"),
		musicVolume: document.getElementById("music-volume"),
		musicStatus: document.getElementById("music-status"),
		focusPopup: document.getElementById("focus-popup"),
		focusMinuteSelect: document.getElementById("focus-minute-select"),
		focusMinuteCustom: document.getElementById("focus-minute-custom"),
		focusDisplay: document.getElementById("focus-time-display"),
		focusBtn: document.getElementById("focus-btn"),
		focusStartBtn: document.getElementById("focus-start-btn"),
		focusPopoutBtn: document.getElementById("focus-popout-btn"),
		focusResetBtn: document.getElementById("focus-reset-btn"),
		notificationBtn: document.getElementById("notification-btn"),
		notificationBadge: document.getElementById("notification-badge"),
		modal: document.getElementById("breathe-modal"),
		breatheCircle: document.getElementById("breathe-circle"),
		breatheInstruction: document.getElementById("breathe-instruction"),
		breatheTimer: document.getElementById("breathe-timer"),
		startBreatheBtn: document.getElementById("start-breathe-btn"),
		openBreatheBtn: document.getElementById("open-breathe-btn"),
		closeBreatheBtn: document.getElementById("close-breathe-btn"),
		focusToast: document.getElementById("focus-toast"),
		usageAlertToast: document.getElementById("usage-alert-toast")
	};

	const state = {
		totalSecondsToday: 0,
		currentApp: "Unknown",
		topApp: "Unknown",
		lastUsageSignature: "",

	// Removed temporary click debug listener
		launchOnStartupEnabled: false,
		hideOnCloseEnabled: true,
		hourlyNotificationsEnabled: true,
		usageApps: [],
		analyticsArchive: loadAnalyticsArchive(),
		activePage: "dashboard",
		unsubscribeUsageListener: null,
		unsubscribeTrackingListener: null,
		unsubscribeResetListener: null,
		unsubscribeAlertListener: null,
		fallbackInterval: null,
		analyticsDirty: true,
		lastFocusRender: -1,
		forceFocusRender: true,
		isPopupWindow: isFocusPopupWindow,
		// Focus timer defaults
		isFocusing: false,
		focusTimeLeft: 25 * SECONDS_PER_MINUTE,
		focusDurationMinutes: 25,
		fiveHourAlertShown: false,
		alertDayKey: getTodayKey(),
		usageAlertTimer: null
	};

	const todayRecord = state.analyticsArchive.days.find((entry) => entry.dateKey === getTodayKey());
	state.totalSecondsToday = todayRecord ? Number(todayRecord.totalSeconds) || 0 : 0;

	if (isFocusPopupWindow) {
		document.body.classList.add("popup-focus-window");
		state.focusDurationMinutes = Number.isFinite(injectedFocusMinutes) ? Math.max(1, Math.min(240, Math.round(injectedFocusMinutes))) : 25;
		if (Number.isFinite(injectedFocusRemaining) && injectedFocusRemaining > 0) {
			state.focusTimeLeft = Math.max(1, Math.round(injectedFocusRemaining));
		} else {
			state.focusTimeLeft = state.focusDurationMinutes * SECONDS_PER_MINUTE;
		}
		state.isFocusing = true;
	}

	if (dom.focusMinuteCustom) {
		dom.focusMinuteCustom.max = "240";
	}



	bindPageNavigation(state, dom);
	renderUsage(dom.appUsageContainer, FALLBACK_APPS);
	renderAnalytics(state, dom);
	if (!isFocusPopupWindow) {
		initializeUsageSync(state, dom);
		initializeTrackingState(state, dom);
		initializeStartupState(state, dom);
		initializeCloseBehaviorState(state, dom);
		initializeHourlyNotificationsState(state, dom);
		await setupUpdateListener();
	}

	// show embedded popup in the main window
	if (!isFocusPopupWindow && dom.focusPopup) dom.focusPopup.classList.remove("hidden");
	applyFocusInputVisibility(dom);
	updateClockAndGreeting(dom);
	renderTime(state, dom);
	renderFocus(state, dom);

	// Start/Pause/Resume button inside the main app
	dom.focusStartBtn?.addEventListener("click", () => {
		if (!dom.focusStartBtn) return;
		if (!state.isFocusing) {
			// start or resume
			if (state.focusTimeLeft <= 0) {
				state.focusTimeLeft = Math.max(1, Number(state.focusDurationMinutes) || 25) * SECONDS_PER_MINUTE;
			}
			state.isFocusing = true;
		} else {
			// pause
			state.isFocusing = false;
		}
		state.forceFocusRender = true;
		renderFocus(state, dom);
	});

	// Pop Out button: only enabled when timer is running
	dom.focusPopoutBtn?.addEventListener("click", async () => {
		if (!state.isFocusing) {
			const minutes = getSelectedFocusMinutes(dom);
			state.focusDurationMinutes = minutes;
			state.focusTimeLeft = minutes * SECONDS_PER_MINUTE;
			state.isFocusing = true;
		}

		const remaining = Math.max(1, Math.round(state.focusTimeLeft));
		try {
			localStorage.setItem(FOCUS_TIMER_STORAGE_KEY, String(remaining));
			localStorage.setItem(FOCUS_TIMER_RUNNING_STORAGE_KEY, "1");
		} catch (error) {
			console.warn("Could not update focus timer storage before pop out:", error);
		}
		state.forceFocusRender = true;
		renderFocus(state, dom);
		await openFocusPopup(remaining);
	});

	dom.focusMinuteSelect?.addEventListener("change", () => {
		const minutes = getSelectedFocusMinutes(dom);
		state.focusDurationMinutes = minutes;
		applyFocusInputVisibility(dom);
		if (dom.focusMinuteCustom && dom.focusMinuteSelect?.value !== "custom") {
			dom.focusMinuteCustom.value = String(minutes);
		}
		if (!state.isFocusing) {
			state.focusTimeLeft = minutes * SECONDS_PER_MINUTE;
			state.forceFocusRender = true;
			renderFocus(state, dom);
		}
	});

	dom.focusMinuteCustom?.addEventListener("input", () => {
		if (dom.focusMinuteSelect?.value !== "custom") return;
		if (dom.focusMinuteCustom) {
			const numericValue = Number(dom.focusMinuteCustom.value);
			if (Number.isFinite(numericValue)) {
				dom.focusMinuteCustom.value = String(Math.max(1, Math.min(240, Math.round(numericValue))));
			}
		}
		const minutes = getSelectedFocusMinutes(dom);
		state.focusDurationMinutes = minutes;
		applyFocusInputVisibility(dom);
		if (!state.isFocusing) {
			state.focusTimeLeft = minutes * SECONDS_PER_MINUTE;
			state.forceFocusRender = true;
			renderFocus(state, dom);
		}
	});

	dom.focusResetBtn?.addEventListener("click", () => {
		const minutes = getSelectedFocusMinutes(dom);
		state.isFocusing = false;
		state.focusDurationMinutes = minutes;
		state.focusTimeLeft = minutes * SECONDS_PER_MINUTE;
		state.forceFocusRender = true;
			renderFocus(state, dom);
	});

	dom.trackingToggleBtn?.addEventListener("click", async () => {
		const next = !state.trackingEnabled;
		const result = await invokeTauri("set_tracking_enabled", { enabled: next });
		if (result && typeof result.tracking_enabled === "boolean") {
			applyTrackingStatus(result.tracking_enabled, state, dom);
		} else {
			applyTrackingStatus(state.trackingEnabled, state, dom);
		}
	});

	dom.startupToggleBtn?.addEventListener("click", async () => {
		const next = !state.launchOnStartupEnabled;
		const result = await invokeTauri("set_launch_on_startup", { enabled: next });
		if (result && typeof result.enabled === "boolean") {
			applyStartupStatus(result.enabled, state, dom);
		} else {
			applyStartupStatus(state.launchOnStartupEnabled, state, dom);
		}
	});

	dom.closeBehaviorToggleBtn?.addEventListener("click", async () => {
		const next = !state.hideOnCloseEnabled;
		const result = await invokeTauri("set_hide_on_close", { enabled: next });
		if (result && typeof result.hide_on_close === "boolean") {
			applyCloseBehaviorStatus(result.hide_on_close, state, dom);
		} else {
			applyCloseBehaviorStatus(state.hideOnCloseEnabled, state, dom);
		}
	});

	dom.hourlyNotificationsToggleBtn?.addEventListener("click", async () => {
		const next = !state.hourlyNotificationsEnabled;
		const result = await invokeTauri("set_hourly_notifications", { enabled: next });
		if (result && typeof result.enabled === "boolean") {
			applyHourlyNotificationsStatus(result.enabled, state, dom);
		} else {
			applyHourlyNotificationsStatus(state.hourlyNotificationsEnabled, state, dom);
		}
	});

	if (dom.musicAudio && dom.musicVolume) {
		dom.musicAudio.volume = Number(dom.musicVolume.value) || 0.35;
	}

	dom.musicToggleBtn?.addEventListener("click", async () => {
		if (!dom.musicAudio) return;
		if (dom.musicAudio.paused) {
			try {
				if (dom.musicStatus) dom.musicStatus.textContent = "Loading";
				await dom.musicAudio.play();
				dom.musicToggleBtn.textContent = "Pause";
				if (dom.musicStatus) dom.musicStatus.textContent = "Playing";
			} catch (error) {
				console.warn("Could not start music:", error);
				dom.musicToggleBtn.textContent = "Play";
				if (dom.musicStatus) dom.musicStatus.textContent = "Error";
			}
		} else {
			dom.musicAudio.pause();
			dom.musicToggleBtn.textContent = "Play";
			if (dom.musicStatus) dom.musicStatus.textContent = "Paused";
		}
	});

	dom.musicStopBtn?.addEventListener("click", () => {
		if (!dom.musicAudio) return;
		dom.musicAudio.pause();
		dom.musicAudio.currentTime = 0;
		if (dom.musicToggleBtn) dom.musicToggleBtn.textContent = "Play";
		if (dom.musicStatus) dom.musicStatus.textContent = "Stopped";
	});

	dom.musicVolume?.addEventListener("input", () => {
		if (!dom.musicAudio) return;
		dom.musicAudio.volume = Math.max(0, Math.min(1, Number(dom.musicVolume.value) || 0));
	});

	dom.musicAudio?.addEventListener("waiting", () => {
		if (dom.musicStatus) dom.musicStatus.textContent = "Loading";
	});

	dom.musicAudio?.addEventListener("playing", () => {
		if (dom.musicToggleBtn) dom.musicToggleBtn.textContent = "Pause";
		if (dom.musicStatus) dom.musicStatus.textContent = "Playing";
	});

	dom.musicAudio?.addEventListener("pause", () => {
		if (dom.musicAudio?.currentTime === 0) return;
		if (dom.musicToggleBtn) dom.musicToggleBtn.textContent = "Play";
		if (dom.musicStatus) dom.musicStatus.textContent = "Paused";
	});

dom.musicAudio?.addEventListener("error", () => {
	console.error("Music playback failed", {
		code: dom.musicAudio?.error?.code,
		source: dom.musicAudio?.currentSrc
	});

	if (dom.musicToggleBtn) dom.musicToggleBtn.textContent = "Play";
	if (dom.musicStatus) dom.musicStatus.textContent = "Music unavailable";
});

	dom.openBreatheBtn?.addEventListener("click", () => openModal(dom));
	dom.closeBreatheBtn?.addEventListener("click", () => {
		state.breatheTime = 0;
		closeModal(dom, state);
	});
	dom.modal?.addEventListener("click", (event) => {
		if (event.target === dom.modal) closeModal(dom, state);
	});

	dom.startBreatheBtn?.addEventListener("click", () => {
		if (state.isBreathing) return;
		state.isBreathing = true;
		state.breatheTime = 60;
		state.breathePhaseSeconds = 0;
		state.isInhale = true;
		dom.startBreatheBtn.classList.add("hidden");
		applyBreathPhase(state, dom);
	});

	const ticker = setInterval(() => {
		tickApp(state, dom);
	}, 1000);

	const clockTicker = setInterval(() => {
		updateClockAndGreeting(dom);
	}, 30000);

	window.addEventListener("beforeunload", () => {
		clearInterval(ticker);
		clearInterval(clockTicker);
		if (state.usageAlertTimer) {
			clearTimeout(state.usageAlertTimer);
		}
		if (state.fallbackInterval) {
			clearInterval(state.fallbackInterval);
		}
		if (typeof state.unsubscribeUsageListener === "function") {
			state.unsubscribeUsageListener();
		}
		if (typeof state.unsubscribeTrackingListener === "function") {
			state.unsubscribeTrackingListener();
		}
		if (typeof state.unsubscribeResetListener === "function") {
			state.unsubscribeResetListener();
		}
		if (typeof state.unsubscribeAlertListener === "function") {
			state.unsubscribeAlertListener();
		}
	});

	if (listen) {
		listen("tauri://exit", () => {
			clearInterval(ticker);
			clearInterval(clockTicker);
			if (state.usageAlertTimer) {
				clearTimeout(state.usageAlertTimer);
			}
			if (state.fallbackInterval) {
				clearInterval(state.fallbackInterval);
			}
		});
	}
});

function shouldCheckForUpdates() {
    const lastCheck = localStorage.getItem(LAST_UPDATE_CHECK_KEY);

    if (!lastCheck) {
        return true;
    }

    return (Date.now() - Number(lastCheck)) > UPDATE_CHECK_INTERVAL;
}

function markUpdateCheckCompleted() {
    localStorage.setItem(LAST_UPDATE_CHECK_KEY, Date.now().toString());
}

let currentUpdate = null;

async function setupUpdateListener() {
    await listen(
        "update://available",
        (event) => {
            const update = event.payload;

            showUpdateBanner(update);
        }
    );

    await listen(
        "update://checked",
        () => {
            markUpdateCheckCompleted();
        }
    );

    await listen(
        "update://progress",
        (event) => {
            const progress = event.payload;
            const fill = document.getElementById("update-progress-fill");
            const text = document.getElementById("update-progress-text");
            if (fill && progress.total) {
                const pct = Math.round((progress.downloaded / progress.total) * 100);
                fill.style.width = pct + "%";
            }
            if (text && progress.total) {
                const dm = (progress.downloaded / 1024 / 1024).toFixed(1);
                const tm = (progress.total / 1024 / 1024).toFixed(1);
                text.textContent = "Downloading " + dm + " / " + tm + " MB";
            }
        }
    );

    await listen(
        "update://downloaded",
        () => {
            const text = document.getElementById("update-progress-text");
            if (text) text.textContent = "Download complete. Installing…";
        }
    );

    await listen(
        "update://installed",
        () => {
            const banner = document.getElementById("update-banner");
            const progressContainer = document.getElementById("update-progress-container");
            if (banner) {
                banner.innerHTML = '<div><strong>Update installed!</strong><p>Restart TouchGrass to apply the update.</p></div><div class="update-banner-actions"><button id="update-restart-btn">Restart Now</button></div>';
                banner.classList.remove("hidden");
            }
            if (progressContainer) progressContainer.classList.add("hidden");
            document.getElementById("update-restart-btn")?.addEventListener("click", async () => {
                try {
                    await invokeTauri("quit_app");
                } catch (e) {
                    window.location.reload();
                }
            });
        }
    );

    await listen(
        "update://error",
        (event) => {
            const msg = event.payload;
            const text = document.getElementById("update-progress-text");
            const btn = document.getElementById("update-btn");
            const dismissBtn = document.getElementById("update-dismiss-btn");
            const progressContainer = document.getElementById("update-progress-container");
            if (text) text.textContent = String(msg);
            if (btn) {
                btn.textContent = "Retry Download";
                btn.disabled = false;
            }
            if (dismissBtn) dismissBtn.classList.remove("hidden");
            if (progressContainer) progressContainer.classList.add("hidden");
        }
    );
}

async function initializeTrackingState(state, dom) {
	const status = await invokeTauri("get_tracking_status");
	if (status && typeof status.tracking_enabled === "boolean") {
		applyTrackingStatus(status.tracking_enabled, state, dom);
	}

	const tauriEvent = { listen: listen };
	

	try {
		state.unsubscribeTrackingListener = await tauriEvent.listen("tracking://status", (event) => {
			const payload = event?.payload;
			if (!payload || typeof payload.tracking_enabled !== "boolean") return;
			applyTrackingStatus(payload.tracking_enabled, state, dom);
		});
	} catch (error) {
		console.warn("Could not subscribe to tracking status:", error);
	}
}

async function initializeStartupState(state, dom) {
	const status = await invokeTauri("get_launch_on_startup");
	if (status && typeof status.enabled === "boolean") {
		applyStartupStatus(status.enabled, state, dom);
	}
}

async function initializeCloseBehaviorState(state, dom) {
	const status = await invokeTauri("get_close_behavior");
	if (status && typeof status.hide_on_close === "boolean") {
		applyCloseBehaviorStatus(status.hide_on_close, state, dom);
	}
}

async function initializeHourlyNotificationsState(state, dom) {
	const status = await invokeTauri("get_hourly_notifications");
	if (status && typeof status.enabled === "boolean") {
		applyHourlyNotificationsStatus(status.enabled, state, dom);
	}
}

async function initializeUsageSync(state, dom) {
	await refreshUsageSnapshot(state, dom);

	const tauriEvent = { listen: listen };
	if (!tauriEvent?.listen) {
		state.fallbackInterval = setInterval(() => {
			refreshUsageSnapshot(state, dom);
		}, 15000);
		return;
	}

	try {
		state.unsubscribeUsageListener = await tauriEvent.listen("usage://snapshot", (event) => {
			const snapshot = event?.payload;
			if (!snapshot || !Array.isArray(snapshot.apps)) return;
			applyUsageSnapshot(snapshot, state, dom);
		});
		state.unsubscribeAlertListener = await tauriEvent.listen("usage://alert", (event) => {
			const payload = event?.payload;
			if (!payload) return;

			if (payload.level === "critical") {
				state.fiveHourAlertShown = true;
				applyNotificationAlertState(true, dom);
				showUsageAlertToast(payload.message || "Usage alert", true, state, dom);
			}
		});
		state.unsubscribeResetListener = await tauriEvent.listen("usage://reset", () => {
			state.analyticsArchive = { days: [], lastSnapshot: null };
			saveAnalyticsArchive(state.analyticsArchive);
			state.totalSecondsToday = 0;
			state.fiveHourAlertShown = false;
			state.alertDayKey = getTodayKey();
			applyNotificationAlertState(false, dom);
			renderAnalytics(state, dom);
			renderTime(state, dom);
		});
	} catch (error) {
		console.warn("Could not subscribe to usage updates:", error);
	}
}

function applyNotificationAlertState(active, dom) {
	if (dom.notificationBtn) {
		dom.notificationBtn.classList.toggle("is-alert", active);
	}

	if (dom.notificationBadge) {
		dom.notificationBadge.classList.toggle("hidden", !active);
	}
}

function showUsageAlertToast(message, critical, state, dom) {
	if (!dom.usageAlertToast) return;

	if (state.usageAlertTimer) {
		clearTimeout(state.usageAlertTimer);
		state.usageAlertTimer = null;
	}

	dom.usageAlertToast.textContent = message;
	dom.usageAlertToast.classList.toggle("critical", Boolean(critical));
	dom.usageAlertToast.classList.remove("hidden");

	requestAnimationFrame(() => {
		dom.usageAlertToast?.classList.add("show");
	});

	state.usageAlertTimer = setTimeout(() => {
		dom.usageAlertToast?.classList.remove("show");
		state.usageAlertTimer = setTimeout(() => {
			dom.usageAlertToast?.classList.add("hidden");
			state.usageAlertTimer = null;
		}, 220);
	}, 4200);
}

function showUpdateBanner(update) {
    const key = "update_seen_" + update.version;

    if (localStorage.getItem(key)) {
        return;
    }

    currentUpdate = update;

    const banner =
        document.getElementById(
            "update-banner"
        );

    const version =
        document.getElementById(
            "update-version"
        );

    const notes =
        document.getElementById(
            "update-notes"
        );

    version.textContent =
        "Version " + update.version;

    const cleanNotes = update.notes
        .replaceAll("##", "")
        .replaceAll("-", "✓");

    notes.textContent = cleanNotes;

    banner.classList.remove(
        "hidden"
    );

    const dismissBtn =
        document.getElementById(
            "update-dismiss-btn"
        );

    if (dismissBtn) {
        dismissBtn.addEventListener(
            "click",
            () => {
                banner.classList.add("hidden");
                localStorage.setItem(key, "true");
            }
        );
    }

    const downloadBtn =
        document.getElementById(
            "update-btn"
        );

    if (downloadBtn) {
        downloadBtn.addEventListener(
            "click",
            async () => {
                if (!currentUpdate) return;

                downloadBtn.textContent = "Downloading…";
                downloadBtn.disabled = true;
                dismissBtn.classList.add("hidden");

                const progressContainer =
                    document.getElementById(
                        "update-progress-container"
                    );

                progressContainer.classList.remove("hidden");

                try {
                    await invokeTauri(
                        "download_and_install_update",
                        {
                            version: currentUpdate.version,
                            notes: currentUpdate.notes,
                        }
                    );
                } catch (error) {
                    console.warn("Update download failed:", error);
                    downloadBtn.textContent = "Retry Download";
                    downloadBtn.disabled = false;
                    dismissBtn.classList.remove("hidden");
                    progressContainer.classList.add("hidden");
                }
            }
        );
    }
}

function loadAnalyticsArchive() {
	const raw = localStorage.getItem(ANALYTICS_STORAGE_KEY);
	if (!raw) return { days: [], lastSnapshot: null };

	try {
		const parsed = JSON.parse(raw);
		if (!parsed || typeof parsed !== 'object') throw new Error('Invalid archive format');
		const archive = normalizeAnalyticsArchive(parsed);
		const trimmed = trimAnalyticsArchive(archive);
		saveAnalyticsArchive(trimmed);
		return trimmed;
	} catch (error) {
		console.warn("Could not parse analytics archive:", error);
		localStorage.removeItem(ANALYTICS_STORAGE_KEY);
		return { days: [], lastSnapshot: null };
	}
}

function saveAnalyticsArchive(archive) {
	localStorage.setItem(ANALYTICS_STORAGE_KEY, JSON.stringify(archive));
}

function normalizeAnalyticsArchive(archive) {
	const days = Array.isArray(archive?.days) ? archive.days : [];
	return {
		days: days
			.map((entry) => normalizeDayRecord(entry))
			.filter(Boolean)
			.sort((left, right) => left.dateKey.localeCompare(right.dateKey))
			.slice(-ANALYTICS_WINDOW_DAYS),
		lastSnapshot: normalizeSnapshotRecord(archive?.lastSnapshot)
	};
}

function trimAnalyticsArchive(archive, referenceDate = new Date()) {
	const normalized = new Date(referenceDate);
	normalized.setHours(0, 0, 0, 0);
	const windowStart = new Date(normalized);
	windowStart.setDate(windowStart.getDate() - (ANALYTICS_WINDOW_DAYS - 1));
	const windowStartKey = getTodayKey(windowStart);

	const days = archive.days
		.filter((day) => day.dateKey >= windowStartKey)
		.sort((left, right) => left.dateKey.localeCompare(right.dateKey));

	const lastSnapshot = archive.lastSnapshot && archive.lastSnapshot.dateKey >= windowStartKey
		? archive.lastSnapshot
		: null;

	return {
		days,
		lastSnapshot
	};
}

function normalizeDayRecord(day) {
	if (!day?.dateKey) return null;
	return {
		dateKey: String(day.dateKey),
		weekday: day.weekday || getWeekdayLabel(new Date(`${day.dateKey}T00:00:00`)),
		displayDate: day.displayDate || formatLongDate(day.dateKey),
		totalSeconds: Number(day.totalSeconds) || 0,
		apps: normalizeAppMap(day.apps)
	};
}

function normalizeSnapshotRecord(snapshot) {
	if (!snapshot?.dateKey) return null;
	return {
		dateKey: String(snapshot.dateKey),
		totalSeconds: Number(snapshot.totalSeconds) || 0,
		apps: normalizeAppMap(snapshot.apps)
	};
}

function normalizeAppMap(apps) {
	if (Array.isArray(apps)) {
		return apps.reduce((accumulator, app) => {
			if (!app?.name) return accumulator;
			accumulator[app.name] = Number(app.seconds) || 0;
			return accumulator;
		}, {});
	}

	if (apps && typeof apps === "object") {
		return Object.entries(apps).reduce((accumulator, [name, seconds]) => {
			accumulator[name] = Number(seconds) || 0;
			return accumulator;
		}, {});
	}

	return {};
}

function getTodayKey(referenceDate = new Date()) {
	const year = referenceDate.getFullYear();
	const month = String(referenceDate.getMonth() + 1).padStart(2, "0");
	const day = String(referenceDate.getDate()).padStart(2, "0");
	return `${year}-${month}-${day}`;
}

function getWeekdayLabel(referenceDate) {
	return referenceDate.toLocaleDateString([], { weekday: "long" });
}

function formatLongDate(dateKey) {
	const date = new Date(`${dateKey}T00:00:00`);
	return date.toLocaleDateString([], { month: "short", day: "numeric", year: "numeric" });
}

function formatDurationShort(seconds) {
	const s = Math.max(0, Number(seconds) || 0);
	if (s >= 3600) {
		const hrs = Math.floor(s / 3600);
		const mins = Math.floor((s % 3600) / 60);
		return mins > 0 ? `${hrs}h ${mins}m` : `${hrs}h`;
	}
	if (s >= 60) {
		const mins = Math.floor(s / 60);
		const secs = s % 60;
		return secs > 0 ? `${mins}m ${secs}s` : `${mins}m`;
	}
	return `${s}s`;
}

function addOrUpdateDay(archive, dateKey) {
	let day = archive.days.find((entry) => entry.dateKey === dateKey);
	if (!day) {
		day = {
			dateKey,
			weekday: getWeekdayLabel(new Date(`${dateKey}T00:00:00`)),
			displayDate: formatLongDate(dateKey),
			totalSeconds: 0,
			apps: {}
		};
		archive.days.push(day);
	}
	return day;
}

function calculateAppDeltas(currentApps, previousApps) {
	const deltas = {};
	const appNames = new Set([...Object.keys(currentApps), ...Object.keys(previousApps)]);
	for (const name of appNames) {
		const delta = Math.max(0, (Number(currentApps[name]) || 0) - (Number(previousApps[name]) || 0));
		if (delta > 0) deltas[name] = delta;
	}
	return deltas;
}

function mergeAppDeltas(target, deltas) {
	for (const [name, seconds] of Object.entries(deltas)) {
		target[name] = (Number(target[name]) || 0) + seconds;
	}
}

function updateAnalyticsArchiveFromSnapshot(snapshot, state) {
	const archive = normalizeAnalyticsArchive(state.analyticsArchive);
	const todayKey = getTodayKey();

	const currentApps = normalizeAppMap(snapshot.apps);
	const currentTotal = Number(snapshot.total_seconds) || 0;

	const day = addOrUpdateDay(archive, todayKey);

	// Rust is source of truth
	day.totalSeconds = currentTotal;

	day.apps = {};
	for (const [name, seconds] of Object.entries(currentApps)) {
		day.apps[name] = seconds;
	}

	archive.lastSnapshot = {
		dateKey: todayKey,
		totalSeconds: currentTotal,
		apps: currentApps
	};

	const finalArchive = trimAnalyticsArchive(archive);

	state.totalSecondsToday = currentTotal;
	state.analyticsArchive = finalArchive;
	state.analyticsDirty = true;

	saveAnalyticsArchive(finalArchive);
}

function renderUsage(container, usageItems) {
	if (!container) return;
	container.innerHTML = "";

	const fragment = document.createDocumentFragment();
	usageItems.forEach((item, index) => {
		const fillClass = USAGE_FILL_CLASSES[index % USAGE_FILL_CLASSES.length];
		const timeText = formatDurationShort(item.seconds || 0);
		const shortName = getBadgeLabel(item.name);
		const safePercent = Math.max(0, Math.min(100, Math.round(item.percent || 0)));

		const row = document.createElement("div");
		row.className = "usage-item";
		row.innerHTML = `
			<div class="usage-top">
				<div class="usage-app">
					<span class="usage-icon" aria-hidden="true">${shortName}</span>
					<span>${item.name}</span>
				</div>
				<span class="usage-time">${timeText}</span>
			</div>
			<div class="usage-track">
				<div class="usage-fill ${fillClass}" data-target="${safePercent}"></div>
			</div>
		`;
		fragment.appendChild(row);
	});

	container.appendChild(fragment);

	requestAnimationFrame(() => {
		const bars = container.querySelectorAll(".usage-fill");
		for (const bar of bars) {
			const target = bar.getAttribute("data-target") || "0";
			bar.style.width = `${target}%`;
		}
	});
}

function showFocusToast(dom) {
  if (!dom.focusToast) return;
  dom.focusToast.classList.remove("hidden");
  requestAnimationFrame(() => dom.focusToast.classList.add("show"));
  setTimeout(() => {
    dom.focusToast.classList.remove("show");
    setTimeout(() => dom.focusToast.classList.add("hidden"), 300);
  }, 3500);
}

function tickApp(state, dom) {
	if (state.isFocusing && state.focusTimeLeft % 15 === 0) {
		updateClockAndGreeting(dom);
	}

	if (state.isFocusing) {
		state.focusTimeLeft = Math.max(0, state.focusTimeLeft - 1);
		if (state.focusTimeLeft === 0) {
			state.isFocusing = false;
			showFocusToast(dom);
			setTimeout(() => {
				state.focusTimeLeft = state.focusDurationMinutes * SECONDS_PER_MINUTE;
				state.forceFocusRender = true;
					renderFocus(state, dom);
			}, 2000);
		}
	}

	if (state.isBreathing) {
		state.breatheTime = Math.max(0, state.breatheTime - 1);
		state.breathePhaseSeconds += 1;

		if (dom.breatheTimer) {
			dom.breatheTimer.textContent = state.breatheTime.toString();
		}

		if (state.breathePhaseSeconds >= 4) {
			state.breathePhaseSeconds = 0;
			state.isInhale = !state.isInhale;
			applyBreathPhase(state, dom);
		}

		if (state.breatheTime <= 0) {
			closeModal(dom, state);
		}
	}

	renderFocus(state, dom);
}

function renderTime(state, dom) {
	const totalSeconds = Number(
		state.snapshot?.total_seconds ??
		state.totalSeconds ??
		0
	);

	const totalHours = Math.floor(totalSeconds / 3600);
	const totalMins = Math.floor((totalSeconds % 3600) / SECONDS_PER_MINUTE);

	if (dom.heroHours) {
		dom.heroHours.textContent = `${totalHours}h ${totalMins}m`;
	}

	if (dom.progressCircle) {
		const progressPercent = Math.min(
			(totalSeconds / DAILY_GOAL_SECONDS) * 100,
			100
		);

		dom.progressCircle.setAttribute(
			"stroke-dasharray",
			`${progressPercent} 100`
		);
	}
}

function getSelectedFocusMinutes(dom) {
	const selected = dom.focusMinuteSelect?.value || "25";
	if (selected === "custom") {
		const customValue = Number(dom.focusMinuteCustom?.value) || 25;
		const clamped = Math.max(1, Math.min(240, Math.round(customValue)));
		if (dom.focusMinuteCustom && String(clamped) !== dom.focusMinuteCustom.value) {
			dom.focusMinuteCustom.value = String(clamped);
		}
		return clamped;
	}

	const preset = Number(selected);
	return Math.max(1, Math.min(240, Number.isFinite(preset) ? preset : 25));
}

async function refreshUsageSnapshot(state, dom) {
	const snapshot = await invokeTauri("get_usage_snapshot");
	if (!snapshot || !Array.isArray(snapshot.apps)) return;
	applyUsageSnapshot(snapshot, state, dom);
}

function applyUsageSnapshot(snapshot, state, dom) {
	state.snapshot = snapshot;
	const todayKey = getTodayKey();
	if (state.alertDayKey !== todayKey) {
		state.alertDayKey = todayKey;
		state.fiveHourAlertShown = false;
		applyNotificationAlertState(false, dom);
	}

	state.currentApp = snapshot.current_app || "Unknown";
	state.topApp = snapshot.top_app || "Unknown";
	state.usageApps = Array.isArray(snapshot.apps) ? snapshot.apps : [];
	updateAnalyticsArchiveFromSnapshot(snapshot, state);
	if (typeof snapshot.tracking_enabled === "boolean") {
		applyTrackingStatus(snapshot.tracking_enabled, state, dom);
	}

	const signature = snapshot.apps
		.map((entry) => `${entry.name}:${entry.seconds}`)
		.join("|");

	if (signature !== state.lastUsageSignature) {
		renderUsage(dom.appUsageContainer, snapshot.apps);
		state.lastUsageSignature = signature;
	}

	if (dom.topAppLabel) {
		dom.topAppLabel.textContent = `Top app: ${state.topApp}`;
	}

	if (dom.topAppInline) {
		dom.topAppInline.textContent = state.topApp;
	}

	if (dom.trackingInline) {
		dom.trackingInline.textContent = state.trackingEnabled ? "On" : "Off";
	}

	renderTime(state, dom);

	const totalSeconds = Number(snapshot.total_seconds) || 0;
	const overLimit = totalSeconds >= DAILY_ALERT_THRESHOLD_SECONDS;
	applyNotificationAlertState(overLimit, dom);
	if (overLimit && !state.fiveHourAlertShown) {
		state.fiveHourAlertShown = true;
		showUsageAlertToast("Alert: You have used your PC for 5 hours today. Please take a break.", true, state, dom);
	}

	if (state.analyticsDirty && state.activePage === "analytics") {
		renderAnalytics(state, dom);
	}
}

function applyTrackingStatus(enabled, state, dom) {
	state.trackingEnabled = enabled;

	if (dom.trackingToggleBtn) {
		dom.trackingToggleBtn.textContent = enabled ? "Tracking ON" : "Tracking OFF";
		dom.trackingToggleBtn.classList.toggle("on", enabled);
		dom.trackingToggleBtn.classList.toggle("off", !enabled);
	}

	if (dom.trackingStatusText) {
		dom.trackingStatusText.textContent = enabled
			? "Tracking is active"
			: "Tracking is paused";
	}

	if (dom.trackingSummaryText) {
		dom.trackingSummaryText.textContent = enabled
			? "Tracking active PC session"
			: "Tracking paused";
	}
}

function applyStartupStatus(enabled, state, dom) {
	state.launchOnStartupEnabled = enabled;

	if (dom.startupToggleBtn) {
		dom.startupToggleBtn.textContent = enabled ? "Startup ON" : "Startup OFF";
		dom.startupToggleBtn.classList.toggle("on", enabled);
		dom.startupToggleBtn.classList.toggle("off", !enabled);
	}

	if (dom.startupStatusText) {
		dom.startupStatusText.textContent = enabled
			? "Launch on startup is enabled"
			: "Launch on startup is disabled";
	}
}

function applyCloseBehaviorStatus(enabled, state, dom) {
	state.hideOnCloseEnabled = enabled;

	if (dom.closeBehaviorToggleBtn) {
		dom.closeBehaviorToggleBtn.textContent = enabled ? "Hide On Close" : "Close Exits";
		dom.closeBehaviorToggleBtn.classList.toggle("on", enabled);
		dom.closeBehaviorToggleBtn.classList.toggle("off", !enabled);
	}

	if (dom.closeBehaviorStatusText) {
		dom.closeBehaviorStatusText.textContent = enabled
			? "Close action hides app to tray"
			: "Close action exits the app";
	}
}

function applyHourlyNotificationsStatus(enabled, state, dom) {
	state.hourlyNotificationsEnabled = enabled;

	if (dom.hourlyNotificationsToggleBtn) {
		dom.hourlyNotificationsToggleBtn.textContent = enabled ? "Hourly ON" : "Hourly OFF";
		dom.hourlyNotificationsToggleBtn.classList.toggle("on", enabled);
		dom.hourlyNotificationsToggleBtn.classList.toggle("off", !enabled);
	}

	if (dom.hourlyNotificationsStatusText) {
		dom.hourlyNotificationsStatusText.textContent = enabled
			? "Hourly background notifications are enabled"
			: "Hourly background notifications are disabled";
	}
}

function bindPageNavigation(state, dom) {
	const pageButtons = document.querySelectorAll("[data-page-btn]");
	const pageViews = document.querySelectorAll("[data-page]");

	const activatePage = (pageName) => {
		console.log("Navigation: activatePage ->", pageName);
		state.activePage = pageName;
		document.activeElement?.blur();
		if (state.isBreathing) {
			closeModal(dom, state);
		}
		const pageViews = document.querySelectorAll(".page-view");
		const pageButtons = document.querySelectorAll("[data-page-btn]");

		pageViews.forEach((view) => {
			view.classList.toggle("page-active", view.getAttribute("data-page") === pageName);
		});

		pageButtons.forEach((button) => {
			button.classList.toggle("active", button.getAttribute("data-page-btn") === pageName);
		});

		if (pageName === "analytics" && state.analyticsDirty) {
			renderAnalytics(state, dom);
		}
	};

	pageButtons.forEach((button) => {
		button.addEventListener("click", () => activatePage(button.getAttribute("data-page-btn") || "dashboard"));
	});

	activatePage("dashboard");
}

function renderAnalytics(state, dom) {
	if (!dom.analyticsChart || !dom.analyticsList || !dom.analyticsDays) return;
	if (!state.analyticsDirty) return;

	const archive = state.analyticsArchive;
	const days = archive.days.slice().sort((left, right) => left.dateKey.localeCompare(right.dateKey));
	const totalSeconds = days.reduce((sum, day) => sum + (Number(day.totalSeconds) || 0), 0);
	const averageSeconds = days.length ? Math.round(totalSeconds / days.length) : 0;
	const mostUsedDay = days.length ? days.reduce((best, day) => ((Number(day.totalSeconds) || 0) > (Number(best.totalSeconds) || 0) ? day : best), days[0]) : null;
	const leastUsedDay = days.length ? days.reduce((worst, day) => ((Number(day.totalSeconds) || 0) < (Number(worst.totalSeconds) || 0) ? day : worst), days[0]) : null;
	const appTotals = aggregateAppTotals(days);
	const topApps = Object.entries(appTotals)
		.sort((left, right) => right[1] - left[1])
		.slice(0, 5)
		.map(([name, seconds]) => ({ name, seconds, percent: totalSeconds ? (seconds / totalSeconds) * 100 : 0 }));

	if (dom.analyticsRangeLabel) dom.analyticsRangeLabel.textContent = `${days.length}/7 days`;
	if (dom.analyticsTodayLabel) dom.analyticsTodayLabel.textContent = getWeekdayLabel(new Date());
	if (dom.analyticsTodayDate) dom.analyticsTodayDate.textContent = formatLongDate(getTodayKey());
	if (dom.analyticsAverageTime) dom.analyticsAverageTime.textContent = formatDurationShort(averageSeconds);
	if (dom.analyticsMostDay) dom.analyticsMostDay.textContent = mostUsedDay ? `${mostUsedDay.weekday} (${mostUsedDay.displayDate})` : "-";
	if (dom.analyticsMostDayTime) dom.analyticsMostDayTime.textContent = mostUsedDay ? formatDurationShort(Number(mostUsedDay.totalSeconds) || 0) : "-";
	if (dom.analyticsLeastDay) dom.analyticsLeastDay.textContent = leastUsedDay ? `${leastUsedDay.weekday} (${leastUsedDay.displayDate})` : "-";
	if (dom.analyticsLeastDayTime) dom.analyticsLeastDayTime.textContent = leastUsedDay ? formatDurationShort(Number(leastUsedDay.totalSeconds) || 0) : "-";
	if (dom.analyticsRecordCount) dom.analyticsRecordCount.textContent = `${days.length} / 7 days stored`;

	renderAnalyticsBars(dom.analyticsChart, days);
	renderTopApps(dom.analyticsList, topApps);
	renderArchiveRows(dom.analyticsDays, days);

	state.analyticsDirty = false;
}

function aggregateAppTotals(days) {
	return days.reduce((totals, day) => {
		for (const [name, seconds] of Object.entries(day.apps || {})) {
			totals[name] = (Number(totals[name]) || 0) + (Number(seconds) || 0);
		}
		return totals;
	}, {});
}

function renderAnalyticsBars(container, days) {
	if (!container) return;
	container.innerHTML = "";
	const maxSeconds = Math.max(1, ...days.map((day) => Number(day.totalSeconds) || 0));
	days.forEach((day) => {
		const percent = Math.max(0, Math.min(100, ((Number(day.totalSeconds) || 0) / maxSeconds) * 100));
		const row = document.createElement("div");
		row.className = "analytics-bar";
		row.innerHTML = `
			<div class="analytics-bar-head">
				<span>${day.weekday}</span>
				<span>${formatDurationShort(Number(day.totalSeconds) || 0)}</span>
			</div>
			<div class="analytics-day-meta">${day.displayDate}</div>
			<div class="analytics-day-track"><div class="analytics-day-fill" data-fill="${percent}"></div></div>
		`;
		container.appendChild(row);
	});
	requestAnimationFrame(() => {
		requestAnimationFrame(() => {
			container.querySelectorAll(".analytics-day-fill").forEach((bar) => {
				bar.style.width = `${bar.getAttribute("data-fill") || "0"}%`;
			});
		});
	});
}

function renderTopApps(container, apps) {
	if (!container) return;
	container.innerHTML = "";
	if (apps.length === 0) {
		container.innerHTML = '<div class="analytics-list-item"><div><strong>No data yet</strong><div class="settings-subtitle">Track activity locally for seven days to unlock rankings</div></div><div class="tag">Offline</div></div>';
		return;
	}
	apps.forEach((item, index) => {
		const row = document.createElement("div");
		row.className = "analytics-list-item";
		row.innerHTML = `
			<div>
				<strong>${index + 1}. ${item.name}</strong>
				<div class="settings-subtitle">${formatDurationShort(Number(item.seconds) || 0)} tracked</div>
			</div>
			<div class="tag">${Math.round(Number(item.percent) || 0)}%</div>
		`;
		container.appendChild(row);
	});
}

function renderArchiveRows(container, days) {
	if (!container) return;
	container.innerHTML = "";
	if (days.length === 0) {
		container.innerHTML = '<div class="analytics-list-item"><div><strong>No archive yet</strong><div class="settings-subtitle">Daily usage records will appear here once tracking runs</div></div><div class="tag">7 day window</div></div>';
		return;
	}
	const maxSeconds = Math.max(1, ...days.map((day) => Number(day.totalSeconds) || 0));
	days.forEach((day) => {
		const percent = Math.max(0, Math.min(100, ((Number(day.totalSeconds) || 0) / maxSeconds) * 100));
		const row = document.createElement("div");
		row.className = "analytics-day-row";
		row.innerHTML = `
			<div class="analytics-day-head">
				<span>${day.weekday}</span>
				<span>${formatDurationShort(Number(day.totalSeconds) || 0)}</span>
			</div>
			<div class="analytics-day-meta">${day.displayDate}</div>
			<div class="analytics-day-track"><div class="analytics-day-fill" data-fill="${percent}"></div></div>
		`;
		container.appendChild(row);
	});
	requestAnimationFrame(() => {
		requestAnimationFrame(() => {
			container.querySelectorAll(".analytics-day-fill").forEach((bar) => {
				bar.style.width = `${bar.getAttribute("data-fill") || "0"}%`;
			});
		});
	});
}

async function invokeTauri(command, args = {}) {
	try {
		return await invoke(command, args);
	} catch (error) {
		console.warn(`Tauri invoke failed for ${command}:`, error);
		return null;
	}
}

function getBadgeLabel(name) {
	const words = String(name || "?")
		.trim()
		.split(/\s+/)
		.filter(Boolean);

	if (words.length === 0) return "?";
	if (words.length === 1) return words[0].slice(0, 2).toUpperCase();
	return `${words[0][0]}${words[1][0]}`.toUpperCase();
}

function renderFocus(state, dom) {
	if (state.lastFocusRender === state.focusTimeLeft && !state.forceFocusRender) return;
	state.lastFocusRender = state.focusTimeLeft;
	state.forceFocusRender = false;

	const selectedMinutes = Math.max(1, Number(state.focusDurationMinutes) || 25);
	const selectedSeconds = selectedMinutes * SECONDS_PER_MINUTE;

	if (dom.focusDisplay) {
		const mins = Math.floor(state.focusTimeLeft / SECONDS_PER_MINUTE)
			.toString()
			.padStart(2, "0");
		const secs = (state.focusTimeLeft % SECONDS_PER_MINUTE).toString().padStart(2, "0");
		dom.focusDisplay.textContent = `${mins}:${secs}`;
	}

	try {
		localStorage.setItem(FOCUS_TIMER_STORAGE_KEY, String(Math.max(0, Math.round(state.focusTimeLeft))));
		localStorage.setItem(FOCUS_TIMER_RUNNING_STORAGE_KEY, state.isFocusing ? "1" : "0");
	} catch (error) {
		console.warn("Could not persist focus timer state:", error);
	}

	if (dom.focusStartBtn) {
		dom.focusStartBtn.textContent = state.isFocusing ? "Pause" : state.focusTimeLeft < selectedSeconds ? "Resume" : "Start";
	}

	if (dom.focusPopoutBtn) {
		dom.focusPopoutBtn.classList.toggle("disabled", !state.isFocusing);
		dom.focusPopoutBtn.setAttribute("aria-disabled", state.isFocusing ? "false" : "true");
	}

	if (dom.focusPopup) {
		dom.focusPopup.classList.toggle("running", state.isFocusing);
	}
}

async function openFocusPopup(remainingSeconds) {
	const safeRemaining = Math.max(1, Math.round(remainingSeconds));
	const tauriInvoke = window.__TAURI__?.core?.invoke;

	if (tauriInvoke) {
		try {
			await tauriInvoke("show_focus_popup", { remainingSeconds: safeRemaining });
			return true;
		} catch (error) {
			console.warn("Could not open native focus popup:", error);
			return false;
		}
	}

	return openFocusPopupFallback(safeRemaining);
}

function openFocusPopupFallback(remainingSeconds) {
	const url = `./popup.html#remaining=${encodeURIComponent(String(Math.max(1, Math.round(remainingSeconds))))}`;
	const popup = window.open(url, "touchgrass-focus", "popup,width=180,height=80,left=40,top=40");
	if (!popup) {
		console.warn("Popup window was blocked by the browser.");
		return false;
	}

	return true;
}

function applyFocusInputVisibility(dom) {
	if (!dom.focusMinuteSelect || !dom.focusMinuteCustom) return;
	dom.focusMinuteCustom.classList.toggle("hidden", dom.focusMinuteSelect.value !== "custom");
}

function updateClockAndGreeting(dom) {
	const now = new Date();
	const hours = now.getHours();
	let greeting = "Good Night!";

	if (hours >= 5 && hours < 12) greeting = "Good Morning!";
	else if (hours >= 12 && hours < 17) greeting = "Good Afternoon!";
	else if (hours >= 17 && hours < 21) greeting = "Good Evening!";

	if (dom.greeting && dom.greeting.textContent !== greeting) {
		dom.greeting.textContent = greeting;
	}

	if (dom.actualTime) {
		dom.actualTime.textContent = now.toLocaleTimeString([], {
			hour: "2-digit",
			minute: "2-digit"
		});
	}
}

function openModal(dom) {
	if (!dom.modal) return;
	dom.modal.classList.remove("hidden");
	requestAnimationFrame(() => {
		dom.modal.classList.add("show");
	});
}

function closeModal(dom, state) {
	if (!dom.modal) return;
	dom.modal.classList.remove("show");
	setTimeout(() => {
		dom.modal?.classList.add("hidden");
	}, 250);

	state.isBreathing = false;
	state.breatheTime = 60;
	state.breathePhaseSeconds = 0;
	state.isInhale = true;

	if (dom.breatheCircle) {
		dom.breatheCircle.classList.remove("inhale");
	}

	if (dom.breatheTimer) {
		dom.breatheTimer.textContent = "60";
	}

	if (dom.breatheInstruction) {
		dom.breatheInstruction.textContent = "Take a moment to center yourself.";
	}

	dom.startBreatheBtn?.classList.remove("hidden");
}

function applyBreathPhase(state, dom) {
	if (dom.breatheInstruction) {
		dom.breatheInstruction.textContent = state.isInhale
			? "Breathe in deeply..."
			: "Breathe out slowly...";
	}

	if (dom.breatheCircle) {
		dom.breatheCircle.classList.toggle("inhale", state.isInhale);
	}
}
