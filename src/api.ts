import { invoke } from "@tauri-apps/api/core";
import type { Deps, HistoryEntry, Job, Options, Settings } from "./types";

export const api = {
  deps: () => invoke<Deps>("check_deps"),
  jobs: () => invoke<Job[]>("list_jobs"),
  add: (url: string, options: Options) => invoke("add_job", { url, options }),
  pause: (id: string) => invoke("pause_job", { id }),
  resume: (id: string) => invoke("resume_job", { id }),
  cancel: (id: string) => invoke("cancel_job", { id }),
  remove: (id: string) => invoke("remove_job", { id }),
  move: (id: string, delta: number) => invoke("move_job", { id, delta }),
  clearFinished: () => invoke("clear_finished"),
  settings: () => invoke<Settings>("get_settings"),
  saveSettings: (settings: Settings) => invoke("set_settings", { settings }),
  history: () => invoke<HistoryEntry[]>("list_history"),
  clearHistory: () => invoke("clear_history"),
  reveal: (path: string) => invoke("reveal", { path }),
  openDir: (path: string) => invoke("open_dir", { path }),
};
