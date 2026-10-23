//! Queue manager and yt-dlp worker tasks.
//!
//! Job order in the vector is the priority order. Pause kills the child process;
//! resuming re-spawns yt-dlp, which continues from the `.part` file on disk.
use crate::store::{HistoryEntry, Store};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    process::Stdio,
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::mpsc,
};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Options {
    pub kind: String,         // "video" | "audio"
    pub resolution: u32,      // 0 = best available
    pub prefer_60fps: bool,
    pub container: String,    // mp4 | mkv | webm
    pub audio_format: String, // mp3 | flac
}

#[derive(Clone, Serialize, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Status { Queued, Active, Paused, Done, Failed, Canceled }

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub id: String,
    pub url: String,
    pub title: String,
    pub options: Options,
    pub status: Status,
    pub downloaded: u64,
    pub total: u64,
    pub speed: f64,
    pub eta: Option<u64>,
    pub elapsed: u64,
    pub percent: f64,
    pub error: Option<String>,
    pub dest: String,
    pub file: Option<String>,
}

enum Stop { Pause, Cancel }

struct Inner {
    jobs: Vec<Job>,
    ctl: HashMap<String, mpsc::UnboundedSender<Stop>>,
    seq: u64,
}

pub struct Manager {
    app: AppHandle,
    store: Arc<Store>,
    inner: Mutex<Inner>,
}

fn num(s: &str) -> Option<f64> {
    s.trim().parse().ok()
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn build_args(job: &Job) -> Vec<String> {
    let o = &job.options;
    let mut a: Vec<String> = [
        "--newline", "--no-playlist", "--no-colors", "--progress", "--no-simulate",
        "--progress-template",
        "download:PROG|%(progress.downloaded_bytes)s|%(progress.total_bytes)s|%(progress.total_bytes_estimate)s|%(progress.speed)s|%(progress.eta)s",
        "--print", "before_dl:TITLE|%(title)s",
        "--print", "after_move:FILE|%(filepath)s",
        "-P", &job.dest,
        "-o", "%(title).150B [%(id)s].%(ext)s",
    ].iter().map(|s| s.to_string()).collect();

    if o.kind == "audio" {
        a.extend(["-f", "ba/b", "-x", "--audio-format", &o.audio_format, "--audio-quality", "0"].map(String::from));
    } else {
        let fmt = match o.container.as_str() {
            "mp4" => "bv*[ext=mp4]+ba[ext=m4a]/bv*+ba/b",
            "webm" => "bv*[ext=webm]+ba[ext=webm]/bv*+ba/b",
            _ => "bv*+ba/b",
        };
        a.extend(["-f", fmt, "--merge-output-format", &o.container].map(String::from));
        let mut sort = if o.resolution > 0 { format!("res:{}", o.resolution) } else { "res".into() };
        if o.prefer_60fps { sort.push_str(",fps"); }
        a.extend(["-S".to_string(), sort]);
    }
    a
}

impl Manager {
    pub fn new(app: AppHandle, store: Arc<Store>) -> Self {
        Manager { app, store, inner: Mutex::new(Inner { jobs: vec![], ctl: HashMap::new(), seq: 0 }) }
    }

    pub fn list(&self) -> Vec<Job> {
        self.inner.lock().unwrap().jobs.clone()
    }

    fn notify(&self) {
        let _ = self.app.emit("jobs-updated", self.list());
    }

    fn update(&self, id: &str, f: impl FnOnce(&mut Job)) {
        if let Some(j) = self.inner.lock().unwrap().jobs.iter_mut().find(|j| j.id == id) {
            f(j);
        }
    }

    pub fn add(self: &Arc<Self>, url: String, options: Options) {
        let dest = self.store.settings().download_dir;
        {
            let mut g = self.inner.lock().unwrap();
            g.seq += 1;
            let id = format!("j{}", g.seq);
            g.jobs.push(Job {
                id, title: url.clone(), url, options, status: Status::Queued,
                downloaded: 0, total: 0, speed: 0.0, eta: None, elapsed: 0, percent: 0.0,
                error: None, dest, file: None,
            });
        }
        self.schedule();
    }

    /// Start as many queued jobs (in priority order) as the concurrency limit allows.
    pub fn schedule(self: &Arc<Self>) {
        let max = self.store.settings().max_concurrent.max(1);
        let mut started = vec![];
        {
            let mut g = self.inner.lock().unwrap();
            let mut active = g.jobs.iter().filter(|j| j.status == Status::Active).count();
            let queued: Vec<String> =
                g.jobs.iter().filter(|j| j.status == Status::Queued).map(|j| j.id.clone()).collect();
            for id in queued {
                if active >= max { break; }
                let (tx, rx) = mpsc::unbounded_channel();
                g.ctl.insert(id.clone(), tx);
                if let Some(j) = g.jobs.iter_mut().find(|j| j.id == id) {
                    j.status = Status::Active;
                    j.error = None;
                }
                active += 1;
                started.push((id, rx));
            }
        }
        for (id, rx) in started {
            let me = self.clone();
            tauri::async_runtime::spawn(async move { me.run(id, rx).await });
        }
        self.notify();
    }

    fn signal(&self, id: &str, s: Stop) -> bool {
        self.inner.lock().unwrap().ctl.get(id).map(|t| t.send(s).is_ok()).unwrap_or(false)
    }

    pub fn pause(&self, id: &str) {
        if !self.signal(id, Stop::Pause) {
            self.update(id, |j| if j.status == Status::Queued { j.status = Status::Paused });
            self.notify();
        }
    }

    pub fn resume(self: &Arc<Self>, id: &str) {
        self.update(id, |j| {
            if matches!(j.status, Status::Paused | Status::Failed | Status::Canceled) {
                j.status = Status::Queued;
            }
        });
        self.schedule();
    }

    pub fn cancel(&self, id: &str) {
        if !self.signal(id, Stop::Cancel) {
            self.update(id, |j| {
                if matches!(j.status, Status::Queued | Status::Paused) { j.status = Status::Canceled }
            });
            self.notify();
        }
    }

    pub fn remove(&self, id: &str) {
        self.inner.lock().unwrap().jobs.retain(|j| !(j.id == id && j.status != Status::Active));
        self.notify();
    }

    pub fn clear_finished(&self) {
        self.inner.lock().unwrap().jobs
            .retain(|j| matches!(j.status, Status::Queued | Status::Active | Status::Paused));
        self.notify();
    }

    /// Move a job up (negative delta) or down (positive) in the queue.
    pub fn move_job(&self, id: &str, delta: i32) {
        {
            let mut g = self.inner.lock().unwrap();
            if let Some(i) = g.jobs.iter().position(|j| j.id == id) {
                let n = (i as i32 + delta).clamp(0, g.jobs.len() as i32 - 1) as usize;
                let j = g.jobs.remove(i);
                g.jobs.insert(n, j);
            }
        }
        self.notify();
    }

    fn finish(self: &Arc<Self>, id: &str, status: Status, err: Option<String>) {
        let mut entry = None;
        {
            let mut g = self.inner.lock().unwrap();
            g.ctl.remove(id);
            if let Some(j) = g.jobs.iter_mut().find(|j| j.id == id) {
                j.status = status.clone();
                j.speed = 0.0;
                j.eta = None;
                j.error = err;
                if status == Status::Done {
                    j.percent = 100.0;
                    if j.total == 0 { j.total = j.downloaded; }
                    entry = Some(HistoryEntry {
                        id: format!("{}-{}", j.id, now()),
                        title: j.title.clone(),
                        url: j.url.clone(),
                        file: j.file.clone(),
                        dir: j.dest.clone(),
                        size: j.total,
                        finished_at: now(),
                    });
                }
            }
        }
        if let Some(e) = entry { self.store.push_history(e); }
        self.schedule();
    }

    async fn run(self: Arc<Self>, id: String, mut rx: mpsc::UnboundedReceiver<Stop>) {
        let job = match self.list().into_iter().find(|j| j.id == id) {
            Some(j) => j,
            None => return,
        };
        let mut cmd = Command::new("yt-dlp");
        cmd.args(build_args(&job)).arg(&job.url)
            .stdout(Stdio::piped()).stderr(Stdio::piped()).kill_on_drop(true);
        #[cfg(windows)]
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                self.finish(&id, Status::Failed,
                    Some(format!("Could not start yt-dlp: {e}. Install it and make sure it is on PATH.")));
                return;
            }
        };
        let mut out = BufReader::new(child.stdout.take().unwrap()).lines();
        let mut err = BufReader::new(child.stderr.take().unwrap()).lines();
        let last_err = Arc::new(Mutex::new(String::new()));
        let le = last_err.clone();
        tauri::async_runtime::spawn(async move {
            while let Ok(Some(l)) = err.next_line().await {
                if l.contains("ERROR") { *le.lock().unwrap() = l; }
            }
        });

        let base = job.elapsed;
        let start = Instant::now();
        let mut last_emit = Instant::now();

        let stop = loop {
            tokio::select! {
                line = out.next_line() => match line {
                    Ok(Some(l)) => {
                        if let Some(t) = l.strip_prefix("TITLE|") {
                            let t = t.to_string();
                            self.update(&id, |j| j.title = t);
                            self.notify();
                        } else if let Some(f) = l.strip_prefix("FILE|") {
                            let f = f.to_string();
                            self.update(&id, |j| j.file = Some(f));
                        } else if let Some(p) = l.strip_prefix("PROG|") {
                            let f: Vec<&str> = p.split('|').collect();
                            if f.len() >= 5 {
                                let done = num(f[0]).unwrap_or(0.0) as u64;
                                let total = num(f[1]).or(num(f[2])).unwrap_or(0.0) as u64;
                                let speed = num(f[3]).unwrap_or(0.0);
                                let eta = num(f[4]).map(|v| v as u64);
                                let elapsed = base + start.elapsed().as_secs();
                                self.update(&id, |j| {
                                    j.downloaded = done;
                                    if total > 0 { j.total = total; }
                                    j.speed = speed;
                                    j.eta = eta;
                                    j.elapsed = elapsed;
                                    j.percent = if j.total > 0 {
                                        (done as f64 / j.total as f64 * 100.0).min(100.0)
                                    } else { 0.0 };
                                });
                                if last_emit.elapsed() > Duration::from_millis(120) {
                                    last_emit = Instant::now();
                                    self.notify();
                                }
                            }
                        }
                    }
                    _ => break None,
                },
                s = rx.recv() => break s,
            }
        };

        match stop {
            Some(Stop::Pause) => {
                let _ = child.kill().await;
                self.finish(&id, Status::Paused, None);
            }
            Some(Stop::Cancel) => {
                let _ = child.kill().await;
                self.finish(&id, Status::Canceled, None);
            }
            None => {
                let ok = child.wait().await.map(|s| s.success()).unwrap_or(false);
                if ok {
                    self.finish(&id, Status::Done, None);
                } else {
                    let msg = last_err.lock().unwrap().clone();
                    let msg = if msg.is_empty() { "yt-dlp exited with an error".into() } else { msg };
                    self.finish(&id, Status::Failed, Some(msg));
                }
            }
        }
    }
}
