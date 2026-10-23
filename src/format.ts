export function bytes(n: number): string {
  if (!n) return "0 MB";
  const mb = n / 1048576;
  return mb >= 1024 ? `${(mb / 1024).toFixed(2)} GB` : `${mb.toFixed(1)} MB`;
}

export const speed = (n: number) => (n ? `${(n / 1048576).toFixed(2)} MB/s` : "–");

export function clock(sec: number | null): string {
  if (sec == null) return "–";
  const h = Math.floor(sec / 3600), m = Math.floor((sec % 3600) / 60), s = sec % 60;
  const p = (v: number) => String(v).padStart(2, "0");
  return h ? `${h}:${p(m)}:${p(s)}` : `${m}:${p(s)}`;
}

const PLATFORMS: [RegExp, string][] = [
  [/(youtube\.com|youtu\.be)/i, "YouTube"],
  [/vimeo\.com/i, "Vimeo"],
  [/twitch\.tv/i, "Twitch"],
  [/(twitter\.com|x\.com)/i, "X"],
  [/tiktok\.com/i, "TikTok"],
  [/instagram\.com/i, "Instagram"],
  [/facebook\.com|fb\.watch/i, "Facebook"],
  [/reddit\.com|redd\.it/i, "Reddit"],
  [/dailymotion\.com/i, "Dailymotion"],
  [/soundcloud\.com/i, "SoundCloud"],
  [/bilibili\.com/i, "Bilibili"],
];

export function platform(url: string): string {
  for (const [re, name] of PLATFORMS) if (re.test(url)) return name;
  if (/\.(mp4|mkv|webm|mov|flv|m3u8|mpd|ts)(\?|#|$)/i.test(url)) return "Direct stream";
  return "Other site";
}

export const extractUrls = (text: string): string[] =>
  Array.from(new Set(text.match(/https?:\/\/[^\s"'<>]+/g) ?? []));

export const when = (secs: number) =>
  new Date(secs * 1000).toLocaleString(undefined, { dateStyle: "medium", timeStyle: "short" });
