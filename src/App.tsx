import { useEffect, useState } from "react";
import { api } from "./api";
import { useApp } from "./store";
import UrlBar from "./components/UrlBar";
import Queue from "./components/Queue";
import History from "./components/History";
import Footer from "./components/Footer";

export default function App() {
  const { init, tab, setTab, jobs, history, deps, refreshDeps } = useApp();
  const [upd, setUpd] = useState("");
  const [busy, setBusy] = useState(false);
  useEffect(() => { init(); }, [init]);

  const update = async () => {
    setBusy(true);
    try { setUpd(await api.updateYtdlp()); } catch (e) { setUpd(String(e)); }
    setBusy(false);
    refreshDeps();
  };

  const missing = deps && (!deps.ytdlp || !deps.ffmpeg);
  const running = jobs.filter((j) => j.status === "active").length;

  return (
    <div className="app">
      <header className="top">
        <h1>Haul</h1>
        <span className="mono dim">
          {deps?.ytdlp ? `yt-dlp ${deps.ytdlp}` : "yt-dlp not found"}{" "}
          <button disabled={busy || !deps?.ytdlp} onClick={update}>{busy ? "Updating…" : "Update yt-dlp"}</button>
        </span>
      </header>

      {missing && (
        <div className="banner" role="alert">
          {!deps?.ytdlp && <span>yt-dlp is not on your PATH. Downloads will fail until it is installed. </span>}
          {!deps?.ffmpeg && <span>ffmpeg is not on your PATH. Merging video and audio, and audio extraction, need it.</span>}
        </div>
      )}

      {upd && <p className="note mono">{upd}</p>}
      {deps?.ytdlp && !deps.deno && (
        <p className="note">
          Recent yt-dlp versions need a JavaScript runtime such as Deno for YouTube. Without one,
          YouTube downloads can fail or come out very small.
        </p>
      )}

      <UrlBar />

      <nav className="tabs" role="tablist">
        <button role="tab" aria-selected={tab === "queue"} onClick={() => setTab("queue")}>
          Queue <span className="mono count">{jobs.length}{running ? ` · ${running} active` : ""}</span>
        </button>
        <button role="tab" aria-selected={tab === "history"} onClick={() => setTab("history")}>
          History <span className="mono count">{history.length}</span>
        </button>
      </nav>

      <main className="list">{tab === "queue" ? <Queue /> : <History />}</main>
      <Footer />
    </div>
  );
}
