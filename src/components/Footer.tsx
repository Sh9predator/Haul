import { open } from "@tauri-apps/plugin-dialog";
import { api } from "../api";
import { useApp } from "../store";

export default function Footer() {
  const { settings, saveSettings } = useApp();
  if (!settings) return <footer className="foot" />;

  const choose = async () => {
    const dir = await open({ directory: true, defaultPath: settings.downloadDir });
    if (typeof dir === "string") saveSettings({ ...settings, downloadDir: dir });
  };

  return (
    <footer className="foot">
      <span className="dim">Save to</span>
      <code className="mono path" title={settings.downloadDir}>{settings.downloadDir}</code>
      <button onClick={choose}>Change</button>
      <button onClick={() => api.openDir(settings.downloadDir)}>Open</button>
      <span className="spacer" />
      <label>Parallel downloads
        <select value={settings.maxConcurrent}
          onChange={(e) => saveSettings({ ...settings, maxConcurrent: +e.target.value })}>
          {[1, 2, 3, 4, 6, 8].map((n) => <option key={n} value={n}>{n}</option>)}
        </select>
      </label>
    </footer>
  );
}
