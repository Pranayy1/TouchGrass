const SECONDS_PER_MINUTE = 60;
const DAILY_GOAL_SECONDS = 8 * 60 * 60;
const ANALYTICS_STORAGE_KEY = "touchgrass_analytics_v1";
const ANALYTICS_WINDOW_DAYS = 7;
const USAGE_FILL_CLASSES = ["code", "browser", "chat", "music"];
const FALLBACK_APPS = [
	{ name: "No data yet", seconds: 1, percent: 100 },
	{ name: "Keep app open", seconds: 0, percent: 0 },
	{ name: "Switch apps", seconds: 0, percent: 0 },
	{ name: "Data updates live", seconds: 0, percent: 0 }
];

document.addEventListener("DOMContentLoaded", () => {
	document.addEventListener("dragstart", (event) => event.preventDefault());

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
		trackingToggleBtn: document.getElementById("tracking-toggle-btn"),
		startupToggleBtn: document.getElementById("startup-toggle-btn"),
		closeBehaviorToggleBtn: document.getElementById("close-behavior-toggle-btn"),
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
		focusDisplay: document.getElementById("focus-time-display"),
		focusBtn: document.getElementById("focus-btn"),
		focusResetBtn: document.getElementById("focus-reset-btn"),
		modal: document.getElementById("breathe-modal"),
		breatheCircle: document.getElementById("breathe-circle"),
		breatheInstruction: document.getElementById("breathe-instruction"),
		breatheTimer: document.getElementById("breathe-timer"),
		startBreatheBtn: document.getElementById("start-breathe-btn"),
		openBreatheBtn: document.getElementById("open-breathe-btn"),
		closeBreatheBtn: document.getElementById("close-breathe-btn"),
		focusToast: document.getElementById("focus-toast")
	};

	const state = {
		totalSecondsToday: 0,
		currentApp: "Unknown",
		topApp: "Unknown",
		lastUsageSignature: "",
		focusTimeLeft: 25 * SECONDS_PER_MINUTE,
		isFocusing: false,
		breatheTime: 60,
		isBreathing: false,
		breathePhaseSeconds: 0,
		isInhale: true,
		lastClockMinute: -1,
		trackingEnabled: true,
		launchOnStartupEnabled: false,
		hideOnCloseEnabled: true,
		usageApps: [],
		analyticsArchive: loadAnalyticsArchive(),
		activePage: "dashboard",
		unsubscribeUsageListener: null,
		unsubscribeTrackingListener: null
	};

	const todayRecord = state.analyticsArchive.days.find((entry) => entry.dateKey === getTodayKey());
	state.totalSecondsToday = todayRecord ? Number(todayRecord.totalSeconds) || 0 : 0;

	bindPageNavigation(state, dom);
	renderUsage(dom.appUsageContainer, FALLBACK_APPS);
	renderAnalytics(state, dom);
	initializeUsageSync(state, dom);
	initializeTrackingState(state, dom);
	initializeStartupState(state, dom);
	initializeCloseBehaviorState(state, dom);
	updateClockAndGreeting(dom);
	renderTime(state, dom);
	renderFocus(state, dom.focusDisplay, dom.focusBtn);

	dom.focusBtn?.addEventListener("click", () => {
		state.isFocusing = !state.isFocusing;
		renderFocus(state, dom.focusDisplay, dom.focusBtn);
	});

	dom.focusResetBtn?.addEventListener("click", () => {
		state.isFocusing = false;
		state.focusTimeLeft = 25 * SECONDS_PER_MINUTE;
		renderFocus(state, dom.focusDisplay, dom.focusBtn);
	});

	dom.trackingToggleBtn?.addEventListener("click", async () => {
		const next = !state.trackingEnabled;
		const result = await invokeTauri("set_tracking_enabled", { enabled: next });
		if (result && typeof result.tracking_enabled === "boolean") {
			applyTrackingStatus(result.tracking_enabled, state, dom);
		}
	});

	dom.startupToggleBtn?.addEventListener("click", async () => {
		const next = !state.launchOnStartupEnabled;
		const result = await invokeTauri("set_launch_on_startup", { enabled: next });
		if (result && typeof result.enabled === "boolean") {
			applyStartupStatus(result.enabled, state, dom);
		}
	});

	dom.closeBehaviorToggleBtn?.addEventListener("click", async () => {
		const next = !state.hideOnCloseEnabled;
		const result = await invokeTauri("set_hide_on_close", { enabled: next });
		if (result && typeof result.hide_on_close === "boolean") {
			applyCloseBehaviorStatus(result.hide_on_close, state, dom);
		}
	});

	dom.musicToggleBtn?.addEventListener("click", async () => {
		if (!dom.musicAudio) return;
		if (dom.musicAudio.paused) {
			try {
				await dom.musicAudio.play();
				dom.musicToggleBtn.textContent = "Pause";
			} catch (error) {
				console.warn("Could not start music:", error);
			}
		} else {
			dom.musicAudio.pause();
			dom.musicToggleBtn.textContent = "Play";
		}
	});

	dom.musicStopBtn?.addEventListener("click", () => {
		if (!dom.musicAudio) return;
		dom.musicAudio.pause();
		dom.musicAudio.currentTime = 0;
		if (dom.musicToggleBtn) dom.musicToggleBtn.textContent = "Play";
	});

	dom.musicVolume?.addEventListener("input", () => {
		if (dom.musicAudio) dom.musicAudio.volume = Number(dom.musicVolume.value);
	});

	dom.openBreatheBtn?.addEventListener("click", () => openModal(dom));
	dom.closeBreatheBtn?.addEventListener("click", () => closeModal(dom, state));
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
		if (typeof state.unsubscribeUsageListener === "function") {
			state.unsubscribeUsageListener();
		}
		if (typeof state.unsubscribeTrackingListener === "function") {
			state.unsubscribeTrackingListener();
		}
	});
});

async function initializeTrackingState(state, dom) {
	const status = await invokeTauri("get_tracking_status");
	if (status && typeof status.tracking_enabled === "boolean") {
		applyTrackingStatus(status.tracking_enabled, state, dom);
	}

	const tauriEvent = window.__TAURI__?.event;
	if (!tauriEvent?.listen) return;

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

async function initializeUsageSync(state, dom) {
	await refreshUsageSnapshot(state, dom);

	const tauriEvent = window.__TAURI__?.event;
	if (!tauriEvent?.listen) {
		setInterval(() => {
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
	} catch (error) {
		console.warn("Could not subscribe to usage updates:", error);
	}
}

function loadAnalyticsArchive() {
	const raw = localStorage.getItem(ANALYTICS_STORAGE_KEY);
	if (!raw) return { days: [], lastSnapshot: null };

	try {
		const archive = normalizeAnalyticsArchive(JSON.parse(raw));
		const trimmed = trimAnalyticsArchive(archive);
		saveAnalyticsArchive(trimmed);
		return trimmed;
	} catch (error) {
		console.warn("Could not parse analytics archive:", error);
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
	const windowStart = new Date(referenceDate);
	windowStart.setHours(0, 0, 0, 0);
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
	const previousSnapshot = normalizeSnapshotRecord(archive.lastSnapshot);
	const day = addOrUpdateDay(archive, todayKey);

	let deltaTotal = currentTotal;
	let deltaApps = currentApps;

	if (previousSnapshot) {
		if (currentTotal < previousSnapshot.totalSeconds) {
			deltaTotal = currentTotal;
			deltaApps = currentApps;
		} else {
			deltaTotal = Math.max(0, currentTotal - previousSnapshot.totalSeconds);
			deltaApps = calculateAppDeltas(currentApps, previousSnapshot.apps);
		}
	}

	day.totalSeconds += deltaTotal;
	mergeAppDeltas(day.apps, deltaApps);
	archive.lastSnapshot = {
		dateKey: todayKey,
		totalSeconds: currentTotal,
		apps: currentApps
	};
	const trimmed = trimAnalyticsArchive(archive);
state.totalSecondsToday = day.totalSeconds;
state.analyticsArchive = trimmed;
saveAnalyticsArchive(trimmed);    
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
		requestAnimationFrame(() => {
			const bars = container.querySelectorAll(".usage-fill");
			for (const bar of bars) {
				const target = bar.getAttribute("data-target") || "0";
				bar.style.width = `${target}%`;
			}
		});
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
			state.focusTimeLeft = 25 * SECONDS_PER_MINUTE;
		}
	}

	if (state.isBreathing) {
		state.breatheTime -= 1;
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

	renderFocus(state, dom.focusDisplay, dom.focusBtn);
}

function renderTime(state, dom) {
	// if (dom.sessionTimer) {
	// 	dom.sessionTimer.textContent = formatClockDuration(state.totalSecondsToday);
	// }

	const totalHours = Math.floor(state.totalSecondsToday / 3600);
	const totalMins = Math.floor((state.totalSecondsToday % 3600) / SECONDS_PER_MINUTE);

	if (dom.heroHours) {
		dom.heroHours.textContent = `${totalHours}h ${totalMins}m`;
	}

	if (dom.progressCircle) {
		const progressPercent = Math.min((state.totalSecondsToday / DAILY_GOAL_SECONDS) * 100, 100);
		dom.progressCircle.setAttribute("stroke-dasharray", `${progressPercent} 100`);
	}
}

async function refreshUsageSnapshot(state, dom) {
	const snapshot = await invokeTauri("get_usage_snapshot");
	if (!snapshot || !Array.isArray(snapshot.apps)) return;
	applyUsageSnapshot(snapshot, state, dom);
}

function applyUsageSnapshot(snapshot, state, dom) {
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

	renderAnalytics(state, dom);

	renderTime(state, dom);
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

function bindPageNavigation(state, dom) {
	const pageButtons = document.querySelectorAll("[data-page-btn]");
	const pageViews = document.querySelectorAll("[data-page]");

	const activatePage = (pageName) => {
		state.activePage = pageName;
		pageButtons.forEach((button) => {
			const active = button.getAttribute("data-page-btn") === pageName;
			button.classList.toggle("active", active);
		});
		pageViews.forEach((view) => {
			view.classList.toggle("page-active", view.getAttribute("data-page") === pageName);
		});
	};

	pageButtons.forEach((button) => {
		button.addEventListener("click", () => activatePage(button.getAttribute("data-page-btn") || "dashboard"));
	});

	activatePage("dashboard");
}

function renderAnalytics(state, dom) {
	if (!dom.analyticsChart || !dom.analyticsList || !dom.analyticsDays) return;

	const archive = normalizeAnalyticsArchive(state.analyticsArchive);
	state.analyticsArchive = archive;
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
		container.querySelectorAll(".analytics-day-fill").forEach((bar) => {
			bar.style.width = `${bar.getAttribute("data-fill") || "0"}%`;
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
		container.querySelectorAll(".analytics-day-fill").forEach((bar) => {
			bar.style.width = `${bar.getAttribute("data-fill") || "0"}%`;
		});
	});
}

async function invokeTauri(command, args = {}) {
	const tauriCore = window.__TAURI__?.core;
	if (!tauriCore?.invoke) return null;

	try {
		return await tauriCore.invoke(command, args);
	} catch (error) {
		console.warn(`Tauri invoke failed for ${command}:`, error);
		return null;
	}
}

function formatDurationShort(seconds) {
	if (seconds < 60) return `${seconds}s`;
	if (seconds < 3600) return `${Math.floor(seconds / 60)}m`;
	const h = Math.floor(seconds / 3600);
	const m = Math.floor((seconds % 3600) / 60);
	return `${h}h ${m}m`;
}

function formatClockDuration(seconds) {
	const safe = Math.max(0, seconds);
	const h = Math.floor(safe / 3600);
	const m = Math.floor((safe % 3600) / 60)
		.toString()
		.padStart(2, "0");
	const s = (safe % 60).toString().padStart(2, "0");
	if (h > 0) return `${h}:${m}:${s}`;
	return `${m}:${s}`;
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

function renderFocus(state, focusDisplay, focusButton) {
	if (focusDisplay) {
		const mins = Math.floor(state.focusTimeLeft / SECONDS_PER_MINUTE)
			.toString()
			.padStart(2, "0");
		const secs = (state.focusTimeLeft % SECONDS_PER_MINUTE).toString().padStart(2, "0");
		focusDisplay.textContent = `${mins}:${secs}`;
	}

	if (focusButton) {
		focusButton.textContent = state.isFocusing ? "Pause" : state.focusTimeLeft < 25 * 60 ? "Resume Focus" : "Start Focus";
	}
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
