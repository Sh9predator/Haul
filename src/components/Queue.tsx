import { api } from "../api";
import { bytes, clock, platform, speed } from "../format";
import { useApp } from "../store";
import type { Job } from "../types";

const LABEL: Record<Job["status"], string> = {
  queued: "Queued", active: "Downloading", paused: "Paused",
  done: "Complete", failed: "Failed", canceled: "Canceled",
};

function Row({ j, first, last }: { j: Job; first: boolean; last: boolean }) {
  const o = j.options;
  const spec = o.kind === "audio" ? o.audioFormat.toUpperCase()
    : `${o.resolution ? o.resolution + "p" : "Best"}${o.prefer60fps ? " 60fps" : ""} ${o.container.toUpperCase()}`;
  const live = j.status === "active";
  const finished = ["done", "failed", "canceled"].includes(j.status);

  return (
    <li className={`job ${j.status}`}>
      <div className="head">
        <div className="titles">
          <span className="title" title={j.title}>{j.title}</span>
          <span className="meta mono">{platform(j.url)} · {spec}</span>
        </div>
        <span className={`state ${j.status}`}>{LABEL[j.status]}</span>
      </div>

      <div className="track" role="progressbar" aria-valuenow={Math.round(j.percent)} aria-valuemin={0} aria-valuemax={100}>
        <div className="fill" style={{ width: `${j.percent}%` }} />
      </div>

      <div className="stats mono">
        <span>{j.percent.toFixed(1)}%</span>
        <span>{bytes(j.downloaded)} / {j.total ? bytes(j.total) : "?"}</span>
        <span>{live ? speed(j.speed) : "–"}</span>
        <span>ETA {live ? clock(j.eta) : "–"}</span>
        <span>Elapsed {clock(j.elapsed)}</span>
      </div>

      {j.error && <p className="err mono">{j.error}</p>}

      <div className="actions">
        {live && <button onClick={() => api.pause(j.id)}>Pause</button>}
        {j.status === "queued" && <button onClick={() => api.pause(j.id)}>Pause</button>}
        {["paused", "failed", "canceled"].includes(j.status) &&
          <button onClick={() => api.resume(j.id)}>{j.status === "paused" ? "Resume" : "Retry"}</button>}
        {!finished && <button className="danger" onClick={() => api.cancel(j.id)}>Cancel</button>}
        {j.status === "done" && j.file && <button onClick={() => api.reveal(j.file!)}>Show in folder</button>}
        {!live && <button onClick={() => api.remove(j.id)}>Remove</button>}
        <span className="spacer" />
        <button aria-label="Move up in queue" disabled={first} onClick={() => api.move(j.id, -1)}>↑</button>
        <button aria-label="Move down in queue" disabled={last} onClick={() => api.move(j.id, 1)}>↓</button>
      </div>
    </li>
  );
}

export default function Queue() {
  const jobs = useApp((s) => s.jobs);
  if (!jobs.length)
    return <p className="empty">The queue is empty. Paste a link above and press Enter.</p>;
  return (
    <>
      <div className="toolbar">
        <button onClick={() => api.clearFinished()}>Clear finished</button>
      </div>
      <ul>{jobs.map((j, i) => <Row key={j.id} j={j} first={i === 0} last={i === jobs.length - 1} />)}</ul>
    </>
  );
}
