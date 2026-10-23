import { useEffect } from "react";
import { useApp } from "./store";
import UrlBar from "./components/UrlBar";
import Queue from "./components/Queue";
import History from "./components/History";
import Footer from "./components/Footer";

export default function App() {
  const { init, tab, setTab, jobs, history, deps } = useApp();
  useEffect(() => { init(); }, [init]);

  const missing = deps && (!deps.ytdlp || !deps.ffmpeg);
  const running = jobs.filter((j) => j.status === "active").length;

  return (
    <div className="app">
      <header className="top">
        <h1>Haul</h1>
        <span className="mono dim">
          {deps?.ytdlp ? `yt-dlp ${deps.ytdlp}` : "yt-dlp not found"}
        </span>
      </header>

      {missing && (
        <div className="banner" role="alert">
          {!deps?.ytdlp && <span>yt-dlp is not on your PATH. Downloads will fail until it is installed. </span>}
          {!deps?.ffmpeg && <span>ffmpeg is not on your PATH. Merging video and audio, and audio extraction, need it.</span>}
        </div>
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
