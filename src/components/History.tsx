import { api } from "../api";
import { bytes, when } from "../format";
import { useApp } from "../store";

export default function History() {
  const { history, refreshHistory } = useApp();
  if (!history.length) return <p className="empty">Finished downloads appear here.</p>;
  return (
    <>
      <div className="toolbar">
        <button className="danger" onClick={async () => { await api.clearHistory(); refreshHistory(); }}>
          Clear history
        </button>
      </div>
      <ul>
        {history.map((h) => (
          <li key={h.id} className="job done hist">
            <div className="head">
              <div className="titles">
                <span className="title" title={h.title}>{h.title}</span>
                <span className="meta mono">{when(h.finishedAt)} · {bytes(h.size)}</span>
              </div>
              <div className="actions">
                {h.file && <button onClick={() => api.reveal(h.file!)}>Show file</button>}
                <button onClick={() => api.openDir(h.dir)}>Open folder</button>
              </div>
            </div>
            <p className="meta mono path" title={h.file ?? h.dir}>{h.file ?? h.dir}</p>
          </li>
        ))}
      </ul>
    </>
  );
}
