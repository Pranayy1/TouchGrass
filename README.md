# 🌿 TouchGrass

**A privacy-first Windows desktop app that helps you build healthier digital habits.**

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows-0078D4.svg)](https://github.com/Pranayy1/TouchGrass/releases)
[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%20v2-FFC131.svg)](https://v2.tauri.app/)
[![Rust](https://img.shields.io/badge/Rust-1.76+-dea584.svg)](https://www.rust-lang.org/)

TouchGrass runs quietly in your system tray, tracking your screen time and app usage — entirely locally. No cloud. No account. No telemetry.

---

## Why TouchGrass?

Most screen-time tools require accounts, send your data to the cloud, or bundle telemetry by default. TouchGrass takes the opposite approach:

- Everything stays on your machine.
- Your data is yours.
- The app does one thing well: help you understand and improve your digital habits.

Whether you are a developer, student, or knowledge worker, TouchGrass gives you honest insight into where your time goes — without compromising your privacy.

---

## ✨ Features

### Dashboard
- Today's total screen time at a glance
- Current and top application detection
- Live tracking status indicator

### Analytics
- Per-application usage breakdown
- 7-day historical archive
- Daily rollover with automatic reset

### Focus
- Configurable focus timer (15 / 25 / 45 minutes or custom)
- Floating popup timer window
- Timer completion notification
- Pause, resume, and reset controls

### Notifications
- Persistent notification center with live sync
- Unread badge indicator
- Mark individual or all notifications as read
- Automatic cleanup of notifications older than 24 hours
- Hourly usage reminders
- 5-hour critical usage alert

### Productivity
- Breathing exercise with guided inhale/exhale phases
- Calm background music with volume control
- System tray integration with quick access
- Launch on startup support

### Privacy
- Zero network calls for tracking data
- No accounts, no logins, no telemetry
- All state stored in a single local JSON file
- Fully offline-capable

### Settings
- Toggle tracking on/off
- Toggle hourly notifications
- Toggle hide-on-close behavior
- Automatic update checker against GitHub Releases

---

## 📸 Screenshots

> Screenshots will be added in a future update.

---

## 🎥 Demo

> A demo GIF/video will be added in a future update.

---

## 📥 Download

| Source | Link |
|--------|------|
| GitHub Releases | [github.com/Pranayy1/TouchGrass/releases](https://github.com/Pranayy1/TouchGrass/releases) |
| Website | [TouchGrass](https://pranayy1.github.io/touchgrass-site/) |

Download the latest `.msi` installer from the Releases page. Run it like any other Windows application.

---

## 🛠 Requirements

### For Users
- Windows 10 (1903+) or Windows 11
- No additional dependencies

### For Development
- [Node.js](https://nodejs.org/) 18+
- [Rust](https://www.rust-lang.org/tools/install) 1.76+
- [Tauri CLI](https://v2.tauri.app/start/prerequisites/)
- Windows SDK (for Tauri build tools)

---

## 🚀 Development Setup

```bash
# Clone the repository
git clone https://github.com/Pranayy1/TouchGrass.git
cd TouchGrass

# Install frontend dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

The compiled installer will be located in `src-tauri/target/release/bundle/nsis/`.

---

## 📁 Project Structure

```
TouchGrass/
├── src/                          # Frontend (vanilla JS + HTML + CSS)
│   ├── assets/                   # Static assets
│   ├── index.html                # Main window markup
│   ├── main.js                   # Main window logic
│   ├── popup.html                # Focus timer popup markup
│   ├── popup.js                  # Focus timer popup logic
│   └── styles.css                # Global stylesheet
├── src-tauri/                    # Backend (Rust + Tauri)
│   ├── src/
│   │   ├── lib.rs                # Core application logic
│   │   └── main.rs               # Tauri entry point
│   ├── Cargo.toml                # Rust dependencies
│   └── tauri.conf.json           # Tauri configuration
├── package.json                  # Node.js dependencies
└── README.md                     # This file
```

---

## 🧱 Architecture

```
┌─────────────────────────────────────────┐
│              Frontend                   │
│   Vanilla JS · HTML · CSS              │
└──────────────────┬──────────────────────┘
                   │ Tauri IPC
┌──────────────────▼──────────────────────┐
│              Rust Backend               │
│   TrackerState · Commands · Listeners   │
└──────────────────┬──────────────────────┘
                   │
        ┌──────────┼──────────┐
        ▼          ▼          ▼
  Windows APIs  Local JSON  Tauri Plugin
  (foreground   Storage     (Notification,
   window)                  Autostart)
```

The frontend communicates with the Rust backend exclusively through Tauri's `invoke` mechanism. State is persisted to a single `touchgrass_state.json` file in the platform's app data directory.

---

## 🔒 Privacy

TouchGrass is built privacy-first:

- **No account** — nothing to sign up for.
- **No login** — no credentials stored or transmitted.
- **No telemetry** — zero analytics or tracking pixels.
- **Local storage** — all data lives in a local JSON file on your machine.
- **Offline** — the app works without an internet connection. The only network call is an optional manual update check against GitHub Releases.

---

## ⚡ Tech Stack

| Layer | Technology |
|-------|------------|
| Backend | Rust |
| Framework | Tauri v2 |
| Frontend | Vanilla JavaScript |
| Styling | Vanilla CSS |
| Storage | Local JSON (`touchgrass_state.json`) |
| OS Integration | Windows APIs (Win32) |
| Notifications | Tauri Notification Plugin |
| Auto-update | GitHub Releases API |

---

## 🛣 Roadmap

### Completed
- Screen time tracking
- Per-application usage tracking
- Daily dashboard
- 7-day analytics
- Focus timer with floating popup
- Timer completion notifications
- Persistent notification center
- Unread/read tracking
- Breathing exercise
- Calm music
- System tray support
- Launch on startup
- Automatic update checker
- Settings page
- Local JSON storage

### Planned
- Screenshots and demo media in README
- Linux support
- macOS support
- Notification history search
- Export usage data to CSV

---

## 🤝 Contributing

Contributions are welcome. Here is how to get started:

1. **Fork** the repository.
2. **Create a branch** for your feature or fix:
   ```bash
   git checkout -b feature/my-feature
   ```
3. **Make your changes** and ensure the project builds:
   ```bash
   cd src-tauri && cargo check
   ```
4. **Commit** with a clear message:
   ```bash
   git commit -m "feat: add notification search"
   ```
5. **Push** and open a Pull Request.

Please keep changes focused and avoid introducing unnecessary dependencies.

---

## ❓ FAQ

**Does TouchGrass send my data anywhere?**
No. All data is stored locally in `touchgrass_state.json`. The only network request is an optional manual update check.

**Does it work offline?**
Yes. Tracking, analytics, focus timer, and all core features work without an internet connection.

**Can I run it on Linux or macOS?**
Currently, TouchGrass targets Windows only. Linux and macOS support is planned.

**Where is my data stored?**
In the platform's app data directory:
- Windows: `%APPDATA%\TouchGrass\touchgrass_state.json`

**How do I reset my data?**
Delete `touchgrass_state.json` from the app data directory. The app will start fresh on next launch.

---

## 📄 License

This project is licensed under the MIT License. A `LICENSE` file will be added to the repository.

---

## 👨‍💻 Author

**Pranay Pandey**

- GitHub: [@Pranayy1](https://github.com/Pranayy1)
- Email: pranaypandey.dev@gmail.com
