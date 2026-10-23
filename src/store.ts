import { create } from "zustand";
import { listen } from "@tauri-apps/api/event";
import { api } from "./api";
import type { Deps, HistoryEntry, Job, Options, Settings } from "./types";

interface State {
  jobs: Job[];
  history: HistoryEntry[];
  settings: Settings | null;
  deps: Deps | null;
  tab: "queue" | "history";
  options: Options;
  setTab: (t: "queue" | "history") => void;
  setOptions: (o: Partial<Options>) => void;
  saveSettings: (s: Settings) => Promise<void>;
  refreshHistory: () => Promise<void>;
  init: () => Promise<void>;
}

export const useApp = create<State>((set, get) => ({
  jobs: [],
  history: [],
  settings: null,
  deps: null,
  tab: "queue",
  options: { kind: "video", resolution: 1080, prefer60fps: false, container: "mp4", audioFormat: "mp3" },

  setTab: (tab) => set({ tab }),
  setOptions: (o) => set((s) => ({ options: { ...s.options, ...o } })),

  saveSettings: async (settings) => {
    await api.saveSettings(settings);
    set({ settings });
  },

  refreshHistory: async () => set({ history: await api.history() }),

  init: async () => {
    let lastDone = -1;
    await listen<Job[]>("jobs-updated", (e) => {
      set({ jobs: e.payload });
      const done = e.payload.filter((j) => j.status === "done").length;
      if (done !== lastDone) {
        lastDone = done;
        get().refreshHistory();
      }
    });
    const [jobs, settings, deps] = await Promise.all([api.jobs(), api.settings(), api.deps()]);
    set({ jobs, settings, deps });
    await get().refreshHistory();
  },
}));
