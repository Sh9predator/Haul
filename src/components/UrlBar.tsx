import { useState } from "react";
import { api } from "../api";
import { extractUrls, platform } from "../format";
import { useApp } from "../store";

const RES = [
  [0, "Best available"], [2160, "2160p (4K)"], [1440, "1440p"],
  [1080, "1080p"], [720, "720p"], [480, "480p"],
] as const;

export default function UrlBar() {
  const [text, setText] = useState("");
  const { options: o, setOptions } = useApp();
  const urls = extractUrls(text);

  const submit = async () => {
    if (!urls.length) return;
    for (const u of urls) await api.add(u, o);
    setText("");
  };

  return (
    <section className="bar">
      <div className="inputrow">
        <textarea
          rows={1}
          value={text}
          placeholder="Paste a video link, or several, one per line"
          spellCheck={false}
          onChange={(e) => setText(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); submit(); }
          }}
          style={{ height: `${Math.min(Math.max(text.split("\n").length, 1), 5) * 22 + 20}px` }}
        />
        <button className="primary" disabled={!urls.length} onClick={submit}>
          {urls.length > 1 ? `Add ${urls.length} to queue` : "Add to queue"}
        </button>
      </div>

      {urls.length > 0 && (
        <ul className="chips">
          {urls.slice(0, 6).map((u) => (
            <li key={u} className="chip mono" title={u}>{platform(u)}</li>
          ))}
          {urls.length > 6 && <li className="chip mono">+{urls.length - 6} more</li>}
        </ul>
      )}

      <div className="opts">
        <div className="seg" role="group" aria-label="Output type">
          {(["video", "audio"] as const).map((k) => (
            <button key={k} aria-pressed={o.kind === k} onClick={() => setOptions({ kind: k })}>
              {k === "video" ? "Video" : "Audio only"}
            </button>
          ))}
        </div>

        {o.kind === "video" ? (
          <>
            <label>Quality
              <select value={o.resolution} onChange={(e) => setOptions({ resolution: +e.target.value })}>
                {RES.map(([v, l]) => <option key={v} value={v}>{l}</option>)}
              </select>
            </label>
            <label className="check">
              <input type="checkbox" checked={o.prefer60fps}
                onChange={(e) => setOptions({ prefer60fps: e.target.checked })} />
              Prefer 60 fps
            </label>
            <label>Container
              <select value={o.container} onChange={(e) => setOptions({ container: e.target.value as typeof o.container })}>
                <option value="mp4">MP4</option><option value="mkv">MKV</option><option value="webm">WebM</option>
              </select>
            </label>
          </>
        ) : (
          <label>Format
            <select value={o.audioFormat} onChange={(e) => setOptions({ audioFormat: e.target.value as typeof o.audioFormat })}>
              <option value="mp3">MP3</option><option value="flac">FLAC (lossless)</option>
            </select>
          </label>
        )}
      </div>
    </section>
  );
}
