# TouchGrass — Digital Wellbeing for Desktop

A lightweight digital wellbeing app for Windows that tracks your screen time and helps you stay focused.
Built with Tauri, Rust, and vanilla HTML/CSS/JS.

![TouchGrass Screenshot](./src/assets/image.png)

## Features

- Real-time app usage tracking with live progress bars
- 7-day analytics with per-app breakdown
- Focus Mode — 25-minute Pomodoro timer
- Breathing Exercise — 1-minute guided reset
- Calm Music — built-in ambient audio player
- System tray support — runs quietly in the background
- Launch on startup option

## Requirements

- Windows 10 / 11 (64-bit)
- [Microsoft Edge WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (pre-installed on Windows 11)
- [Node.js](https://nodejs.org/) v18+
- [Rust](https://rustup.rs/)
- [Visual Studio](https://visualstudio.microsoft.com/) — required for the **Desktop development with C++** workload, which Tauri needs to compile on Windows. You don't need to write or edit any code in Visual Studio — just install it once and do all your actual development in VS Code.


## Setup & Development

1. Clone the repository
```bash
   git clone https://github.com/Pranayy1/TouchGrass.git
   cd TouchGrass
```

2. Install dependencies
```bash
   npm install
```

3. Run in development mode
```bash
   npm run tauri dev
```

4. Build for production
```bash
   npm run tauri build
```
   The installer will be output to `src-tauri/target/release/bundle/`.

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) is the recommended editor for this project, paired with these extensions:

- [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

> **Note:** Before you start, make sure you have [Visual Studio](https://visualstudio.microsoft.com/) installed with the **Desktop development with C++** workload enabled. This is a one-time setup step required by Tauri on Windows — open the Visual Studio Installer, select that workload, and install it. After that, you can close Visual Studio entirely and work exclusively in VS Code.

## Download

Head to the [Releases](https://github.com/Pranayy1/TouchGrass/releases) or [Website](https://pranayy1.github.io/touchgrass-site/) page to download the latest installer.
