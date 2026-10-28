# Haul

A fast, lightweight desktop interface for `yt-dlp` and `ffmpeg`. Built with Tauri, Rust, React, and TypeScript.

---

## Features

* **Link Detection:** Paste single URLs or bulk link lists. Haul automatically parses supported sources (YouTube, Vimeo, Twitch, X, TikTok, direct streams, etc.).
* **Format Control:** Choose resolutions from 4K down to 480p, set a 60 FPS preference, pick container formats (MP4, MKV, WebM), or extract audio directly to MP3 or FLAC.
* **Live Dashboard:** Real-time updates (~8 Hz) tracking progress percentage, downloaded size, speed, ETA, and elapsed time per job.
* **Queue Management:** Configurable parallel download limits, pause, resume, cancel, retry, and reorder jobs to adjust priority.
* **History Log:** Completed downloads are logged locally to JSON. Open the file or reveal its parent folder in one click.
* **Minimal Resource Footprint:** Native Webview via Tauri with a compact release binary size.

---

## Prerequisites

Haul orchestrates local CLI tools and does not bundle them. Both `yt-dlp` and `ffmpeg` must be installed and available on your system `PATH`.

### Installing Dependencies

* **Windows (PowerShell / Command Prompt):**
  ```powershell
  winget install yt-dlp.yt-dlp Gyan.FFmpeg
  ```

* **macOS (Homebrew):**
  ```bash
  brew install yt-dlp ffmpeg
  ```

* **Linux (Debian/Ubuntu example):**
  ```bash
  sudo apt install ffmpeg
  pipx install yt-dlp
  ```

Verify installation by running `yt-dlp --version` and `ffmpeg -version` in a new terminal session. Keep `yt-dlp` updated (`yt-dlp -U`) to ensure extractor compatibility.

### Build Prerequisites

To compile Haul from source, you will need:
1. **Rust** (stable toolchain)
2. **Node.js** (v18 or newer)
3. Platform-specific Tauri dependencies:
   * **Windows:** WebView2
   * **Linux:** `webkit2gtk-4.1` / `libgtk-3-dev`
   * **macOS:** Xcode Command Line Tools

---

## Build & Run

1. **Clone the repository:**
   ```bash
   git clone https://github.com/<your-username>/haul.git
   cd haul
   ```

2. **Install frontend dependencies:**
   ```bash
   npm install
   ```

3. **Generate app icons (optional if `src-tauri/icons` exists):**
   ```bash
   npm run tauri icon icon.png
   ```

4. **Development server (hot reload):**
   ```bash
   npm run tauri dev
   ```

5. **Compile release binary:**
   ```bash
   npm run tauri build
   ```
   The compiled bundles will be output to `src-tauri/target/release/bundle/`.

---

## Architecture

```text
haul/
├── src/                  React + TypeScript frontend
│   ├── components/       UrlBar, Queue, History, Footer
│   ├── store.ts          Zustand state management & Tauri event subscribers
│   ├── api.ts            Typed invocation wrappers for Tauri IPC commands
│   └── styles.css        Design tokens and layout styles
└── src-tauri/            Rust backend
    └── src/
        ├── main.rs       Tauri setup and command handlers
        ├── downloader.rs Queue manager & async tokio worker tasks
        └── store.rs      JSON persistence (settings and history)
```

### Core Execution Flow

* **Download Flow:** Active jobs run as asynchronous `tokio` tasks that spawn `yt-dlp` with a custom `--progress-template`. Standard output is read line-by-line and emitted as a throttled `jobs-updated` event payload to the frontend.
* **Scheduling:** Job execution follows the vector order of the queue. The internal scheduler automatically claims free slots based on your max concurrency setting.
* **Pause & Resume:** Pausing terminates the active `yt-dlp` process. Resuming re-spawns the task, allowing `yt-dlp` to pick up progress from the partial file on disk.

---

## Storage Paths

| Data Type | Location |
| :--- | :--- |
| **Downloads** | Default system `Downloads` folder (configurable via UI footer). |
| **Settings & History** | **Windows:** `%APPDATA%\dev.haul.app\`<br>**macOS:** `~/Library/Application Support/dev.haul.app/`<br>**Linux:** `~/.local/share/dev.haul.app/` |

---

## Troubleshooting

* **Missing Dependency Warning:** Ensure that directory paths (e.g., `C:\Tools\` or `/usr/bin/`), not executable filenames, are assigned to your system `PATH`. Restart your terminal or re-log after changing environment variables.
* **Failed Downloads:** Inspect the error message output directly on the failed job card. Most extraction errors are resolved by running `yt-dlp -U`.
* **Blank Screen in Dev Mode:** Verify that port `1420` is free and that `index.html` is located directly at the root of `src/`.

---

## Known Limitations

* **In-Memory Queue:** Unfinished job queues are cleared on application exit. Partial download files remain intact on disk, allowing re-added links to resume progress.
* **Single Items Only:** Playlists are not automatically expanded; submitted links download a single target item.
* **Authentication:** Sites requiring user authentication or cookies are not currently supported.

---

## Responsible Use

Download only content you own, content in the public domain, or materials you have explicit permission to retrieve. You are responsible for complying with the terms of service of source websites and applicable copyright laws.

---

## License

[MIT](LICENSE)
