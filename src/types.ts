export type Status = "queued" | "active" | "paused" | "done" | "failed" | "canceled";

export interface Options {
  kind: "video" | "audio";
  resolution: number; // 0 = best
  prefer60fps: boolean;
  container: "mp4" | "mkv" | "webm";
  audioFormat: "mp3" | "flac";
  codec: "compat" | "best";
}

export interface Job {
  id: string;
  url: string;
  title: string;
  options: Options;
  status: Status;
  downloaded: number;
  total: number;
  speed: number;
  eta: number | null;
  elapsed: number;
  percent: number;
  error: string | null;
  dest: string;
  file: string | null;
  quality: string | null;
  warning: string | null;
}

export interface HistoryEntry {
  id: string;
  title: string;
  url: string;
  file: string | null;
  dir: string;
  size: number;
  finishedAt: number;
}

export interface Settings { downloadDir: string; maxConcurrent: number }
export interface Deps { ytdlp: string | null; ffmpeg: string | null; deno: string | null }
