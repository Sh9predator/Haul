#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod downloader;
mod store;

use downloader::{Job, Manager, Options};
use serde::Serialize;
use std::sync::Arc;
use store::{HistoryEntry, Settings, Store};
use tauri::{AppHandle, Manager as _, State};
use tauri_plugin_opener::OpenerExt;
use tokio::process::Command;

type Mgr<'a> = State<'a, Arc<Manager>>;
type Db<'a> = State<'a, Arc<Store>>;

#[derive(Serialize)]
struct Deps { ytdlp: Option<String>, ffmpeg: Option<String> }

async fn first_line(bin: &str, arg: &str) -> Option<String> {
    let out = Command::new(bin).arg(arg).output().await.ok()?;
    if !out.status.success() { return None; }
    String::from_utf8_lossy(&out.stdout).lines().next().map(|s| s.to_string())
}

#[tauri::command]
async fn check_deps() -> Deps {
    Deps { ytdlp: first_line("yt-dlp", "--version").await, ffmpeg: first_line("ffmpeg", "-version").await }
}

#[tauri::command] fn list_jobs(m: Mgr) -> Vec<Job> { m.list() }
#[tauri::command] fn add_job(m: Mgr, url: String, options: Options) { m.add(url, options) }
#[tauri::command] fn pause_job(m: Mgr, id: String) { m.pause(&id) }
#[tauri::command] fn resume_job(m: Mgr, id: String) { m.resume(&id) }
#[tauri::command] fn cancel_job(m: Mgr, id: String) { m.cancel(&id) }
#[tauri::command] fn remove_job(m: Mgr, id: String) { m.remove(&id) }
#[tauri::command] fn move_job(m: Mgr, id: String, delta: i32) { m.move_job(&id, delta) }
#[tauri::command] fn clear_finished(m: Mgr) { m.clear_finished() }

#[tauri::command] fn get_settings(s: Db) -> Settings { s.settings() }
#[tauri::command]
fn set_settings(m: Mgr, s: Db, settings: Settings) {
    s.save_settings(settings);
    m.schedule(); // a higher concurrency limit may release queued jobs
}

#[tauri::command] fn list_history(s: Db) -> Vec<HistoryEntry> { s.history() }
#[tauri::command] fn clear_history(s: Db) { s.clear_history() }

#[tauri::command]
fn reveal(app: AppHandle, path: String) -> Result<(), String> {
    app.opener().reveal_item_in_dir(path).map_err(|e| e.to_string())
}

#[tauri::command]
fn open_dir(app: AppHandle, path: String) -> Result<(), String> {
    app.opener().open_path(path, None::<&str>).map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data = app.path().app_data_dir()?;
            let dl = app.path().download_dir().unwrap_or_else(|_| data.clone());
            let store = Arc::new(Store::load(data, dl.to_string_lossy().into_owned()));
            app.manage(Arc::new(Manager::new(app.handle().clone(), store.clone())));
            app.manage(store);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            check_deps, list_jobs, add_job, pause_job, resume_job, cancel_job, remove_job,
            move_job, clear_finished, get_settings, set_settings, list_history, clear_history,
            reveal, open_dir
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Haul");
}
