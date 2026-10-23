THIS IS HAUL 

Features
	
Link detection	Paste one link or a whole list. Haul extracts every URL and labels the source (YouTube, Vimeo, Twitch, X, TikTok, direct streams, and anything else yt-dlp supports).
Format control	4K down to 480p, optional 60 fps preference, MP4 / MKV / WebM containers, or audio-only extraction to MP3 / FLAC.
Live dashboard	Percent, downloaded / total size, speed, ETA and elapsed time per job, updated about 8 times a second.
Queue	Parallel download limit, pause, resume, cancel, retry, and move jobs up or down to change priority.
History	Completed downloads are logged to a local JSON file. Reveal the file or open its folder in one click.
Small footprint	Native webview through Tauri. The release binary is a few MB.
Prerequisites

Haul orchestrates external tools; it does not bundle them. Both must be on your PATH.

Tool	Why
yt-dlp	Extraction and downloading
ffmpeg	Merging video and audio, audio conversion

Windows (PowerShell or cmd)

powershell
winget install yt-dlp.yt-dlp Gyan.FFmpeg

macOS

bash
brew install yt-dlp ffmpeg

Linux

bash
sudo apt install ffmpeg      # or your distro's equivalent
pipx install yt-dlp

Open a new terminal afterwards and confirm with yt-dlp --version and ffmpeg -version. Haul checks at startup and shows a notice if either is missing. Keep yt-dlp current with yt-dlp -U, since sites change often.

To build from source you also need:

Rust (stable)
Node.js 18 or newer
The Tauri platform dependencies for your OS (WebView2 on Windows, webkit2gtk on Linux, Xcode tools on macOS)
Build and run
bash
git clone https://github.com/<you>/haul.git
cd haul
npm install

# Only needed if src-tauri/icons is missing: generate it from a square PNG
npm run tauri icon icon.png

npm run tauri dev      # development, with hot reload
npm run tauri build    # release bundle in src-tauri/target/release/bundle
Architecture
haul/
├── src/                      React + TypeScript frontend
│   ├── components/           UrlBar, Queue, History, Footer
│   ├── store.ts              Zustand store; subscribes to backend events
│   ├── api.ts                Typed wrappers over Tauri commands
│   └── styles.css            Design tokens and component styles
└── src-tauri/                Rust backend
    └── src/
        ├── main.rs           Tauri setup and command handlers
        ├── downloader.rs     Queue manager and per-job worker tasks
        └── store.rs          JSON persistence (settings, history)

Download flow. Each active job is a tokio task that spawns yt-dlp with a custom --progress-template, reads its stdout line by line and emits a throttled jobs-updated event with a snapshot of the queue. The vector order of jobs is the priority order, and a scheduler starts queued jobs whenever a slot is free.

Pause and resume. Pausing stops the yt-dlp process. Resuming starts it again, and yt-dlp continues from the partial file on disk.

Data location. Settings and history are stored as JSON in the platform app-data directory (see below).

Where files are stored
What	Location
Downloads	Your system Downloads folder by default. Change it in the footer.
Settings and history	App data folder: %APPDATA%\dev.haul.app\ on Windows, ~/Library/Application Support/dev.haul.app/ on macOS, ~/.local/share/dev.haul.app/ on Linux
Troubleshooting
A notice says yt-dlp or ffmpeg is missing. Each PATH entry must be a folder (for example C:\Tools, or ...\ffmpeg\bin), not the .exe file. Close and reopen your terminal, or sign out and back in if you launch Haul from the Start menu, so the new PATH is picked up.
A download fails. The error from yt-dlp is shown on the job. Updating with yt-dlp -U fixes most extractor errors.
Blank window in dev. Make sure nothing else is using port 1420, and that the src/ folder is next to index.html.
Limitations
The queue lives in memory. Unfinished jobs are not restored after you quit; partial files stay on disk and a re-added link resumes from them.
Playlists are not expanded; each link downloads a single video.
Sites that require login are not supported yet.
Responsible use

Download only content you own, that is in the public domain, or that you have permission to download. You are responsible for complying with the terms of the sites you use and the laws where you live.

License

MIT