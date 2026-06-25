const timerElement = document.getElementById("timer");
const shellElement = document.getElementById("popup-shell");
const closeButton = document.getElementById("close-popup");
const STORAGE_KEY = "touchgrass_focus_remaining_seconds";
const RUNNING_STORAGE_KEY = "touchgrass_focus_is_running";
const injectedRemaining = Number(window.__FOCUS_POPUP_REMAINING__);
const hashRemaining = Number(new URLSearchParams(window.location.hash.replace(/^#/, "")).get("remaining"));
let lastKnownRemaining = Number.isFinite(injectedRemaining) && injectedRemaining > 0
  ? Math.floor(injectedRemaining)
  : Number.isFinite(hashRemaining) && hashRemaining > 0
    ? Math.floor(hashRemaining)
    : 0;
let isRunning = lastKnownRemaining > 0;
const injectedMinutes = Number(window.__FOCUS_POPUP_MINUTES__);
const originalMinutes = Number.isFinite(injectedMinutes) && injectedMinutes > 0
  ? Math.round(injectedMinutes)
  : 0;
let completionNotified = false;

function formatTime(totalSeconds) {
  const safeSeconds = Math.max(0, Math.floor(totalSeconds));
  const minutes = Math.floor(safeSeconds / 60).toString().padStart(2, "0");
  const seconds = (safeSeconds % 60).toString().padStart(2, "0");
  return `${minutes}:${seconds}`;
}

function render() {
  if (!timerElement) return;
  timerElement.textContent = formatTime(lastKnownRemaining);
}

function writeSharedTimer() {
  try {
    localStorage.setItem(STORAGE_KEY, String(lastKnownRemaining));
    localStorage.setItem(RUNNING_STORAGE_KEY, isRunning ? "1" : "0");
  } catch (error) {
    console.warn("Could not persist popup timer state:", error);
  }
}

function readSharedTimer() {
  const stored = Number(localStorage.getItem(STORAGE_KEY));
  const storedIsRunning = localStorage.getItem(RUNNING_STORAGE_KEY) === "1";
  if (Number.isFinite(stored) && stored >= 0) {
    if (storedIsRunning || lastKnownRemaining > 0) {
      lastKnownRemaining = Math.floor(stored);
    }
  }
  isRunning = storedIsRunning && lastKnownRemaining > 0;
}

function tick() {
  if (!isRunning || lastKnownRemaining <= 0) {
    render();
    return;
  }

  lastKnownRemaining = Math.max(0, lastKnownRemaining - 1);
  if (lastKnownRemaining <= 0) {
    isRunning = false;
    if (!completionNotified && originalMinutes > 0) {
      completionNotified = true;
      invokeTauri("timer_completed", { minutes: originalMinutes }).catch((error) => {
        console.warn("Failed to notify timer completion:", error);
      });
    }
  }
  writeSharedTimer();
  render();
}

async function invokeTauri(command, args = {}) {
  const invoke = window.__TAURI__?.core?.invoke;
  if (!invoke) return false;

  try {
    await invoke(command, args);
    return true;
  } catch (error) {
    console.warn(`Tauri invoke failed for ${command}:`, error);
    return false;
  }
}

async function openMainWindow() {
  await invokeTauri("show_main_window");
}

async function closePopup() {
  const closed = await invokeTauri("close_focus_popup");
  if (!closed) {
    window.close();
  }
}

readSharedTimer();
render();
const countdownTimer = window.setInterval(tick, 1000);

shellElement?.addEventListener("dblclick", async (event) => {
  event.preventDefault();
  event.stopPropagation();
  await openMainWindow();
});

closeButton?.addEventListener("click", async (event) => {
  event.preventDefault();
  event.stopPropagation();
  await closePopup();
});

closeButton?.addEventListener("dblclick", (event) => {
  event.preventDefault();
  event.stopPropagation();
});

window.addEventListener("storage", (event) => {
  if (event.key !== STORAGE_KEY && event.key !== RUNNING_STORAGE_KEY) return;
  readSharedTimer();
  render();
});

window.addEventListener("focus", () => {
  readSharedTimer();
  render();
});

window.addEventListener("beforeunload", () => {
  window.clearInterval(countdownTimer);
});
