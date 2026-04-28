const SECONDS_PER_MINUTE = 60;
const DAILY_GOAL_SECONDS = 8 * 60 * 60;
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
		sessionTimer: document.getElementById("session-timer"),
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
		analyticsTopApp: document.getElementById("analytics-top-app"),
		analyticsTotalTime: document.getElementById("analytics-total-time"),
		analyticsChart: document.getElementById("analytics-chart"),
		analyticsList: document.getElementById("analytics-list"),
		musicAudio: document.getElementById("calm-audio"),
		musicToggleBtn: document.getElementById("music-toggle-btn"),
		musicStopBtn: document.getElementById("music-stop-btn"),
		musicVolume: document.getElementById("music-volume"),
		titleHideBtn: document.getElementById("title-hide-btn"),
		titleQuitBtn: document.getElementById("title-quit-btn"),
		focusDisplay: document.getElementById("focus-time-display"),
		focusBtn: document.getElementById("focus-btn"),
		focusResetBtn: document.getElementById("focus-reset-btn"),
		modal: document.getElementById("breathe-modal"),
		breatheCircle: document.getElementById("breathe-circle"),
		breatheInstruction: document.getElementById("breathe-instruction"),
		breatheTimer: document.getElementById("breathe-timer"),
		startBreatheBtn: document.getElementById("start-breathe-btn"),
		openBreatheBtn: document.getElementById("open-breathe-btn"),
		closeBreatheBtn: document.getElementById("close-breathe-btn")
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
		activePage: "dashboard",
		unsubscribeUsageListener: null,
		unsubscribeTrackingListener: null
	};

	bindPageNavigation(state, dom);
	renderUsage(dom.appUsageContainer, FALLBACK_APPS);
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

	dom.titleHideBtn?.addEventListener("click", () => {
		invokeTauri("hide_to_tray");
	});

	dom.titleQuitBtn?.addEventListener("click", () => {
		invokeTauri("quit_app");
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

function tickApp(state, dom) {
	if (state.focusTimeLeft % 15 === 0) {
		updateClockAndGreeting(dom);
	}

	if (state.isFocusing) {
		state.focusTimeLeft = Math.max(0, state.focusTimeLeft - 1);
		if (state.focusTimeLeft === 0) {
			state.isFocusing = false;
			alert("Focus session complete! Great job.");
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
	if (dom.sessionTimer) {
		dom.sessionTimer.textContent = formatClockDuration(state.totalSecondsToday);
	}

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
	state.totalSecondsToday = Number(snapshot.total_seconds) || 0;
	state.currentApp = snapshot.current_app || "Unknown";
	state.topApp = snapshot.top_app || "Unknown";
	state.usageApps = Array.isArray(snapshot.apps) ? snapshot.apps : [];
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
	if (!dom.analyticsChart || !dom.analyticsList) return;

	const apps = Array.isArray(state.usageApps) && state.usageApps.length > 0
		? state.usageApps
		: FALLBACK_APPS;

	const chartItems = apps.slice(0, 6);
	dom.analyticsChart.innerHTML = "";
	dom.analyticsList.innerHTML = "";

	chartItems.forEach((item) => {
		const percent = Math.max(0, Math.min(100, Number(item.percent) || 0));
		const row = document.createElement("div");
		row.className = "analytics-bar";
		row.innerHTML = `
			<div class="analytics-bar-head">
				<span>${item.name}</span>
				<span>${formatDurationShort(Number(item.seconds) || 0)}</span>
			</div>
			<div class="analytics-bar-track"><div class="analytics-bar-fill" data-fill="${percent}"></div></div>
		`;
		dom.analyticsChart.appendChild(row);
	});

	chartItems.forEach((item, index) => {
		const row = document.createElement("div");
		row.className = "analytics-list-item";
		row.innerHTML = `
			<div>
				<strong>${index + 1}. ${item.name}</strong>
				<div class="settings-subtitle">${formatDurationShort(Number(item.seconds) || 0)} tracked</div>
			</div>
			<div class="tag">${Math.round(Number(item.percent) || 0)}%</div>
		`;
		dom.analyticsList.appendChild(row);
	});

	requestAnimationFrame(() => {
		dom.analyticsChart.querySelectorAll(".analytics-bar-fill").forEach((bar) => {
			const fill = bar.getAttribute("data-fill") || "0";
			bar.style.width = `${fill}%`;
		});
	});

	if (dom.analyticsTopApp) dom.analyticsTopApp.textContent = state.topApp || "Unknown";
	if (dom.analyticsTotalTime) dom.analyticsTotalTime.textContent = formatClockDuration(state.totalSecondsToday);
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
