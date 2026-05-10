export function fmtTime(secs: number | null | undefined): string {
  if (!secs || !isFinite(secs) || secs < 0) return "0:00";
  const total = Math.floor(secs);
  const m = Math.floor(total / 60);
  const s = total % 60;
  if (m >= 60) {
    const h = Math.floor(m / 60);
    const mm = m % 60;
    return `${h}:${String(mm).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
  }
  return `${m}:${String(s).padStart(2, "0")}`;
}

export function fmtMs(ms: number | null | undefined): string {
  return fmtTime(ms ? ms / 1000 : 0);
}

export function fmtAudio(a: { rate: number; bits: number; channels: number } | null): string {
  if (!a) return "—";
  const khz = (a.rate / 1000).toFixed(1);
  return `${khz} kHz · ${a.bits} bit · ${a.channels} ch`;
}

export function backendOf(uri: string | null | undefined): string {
  if (!uri) return "other";
  if (uri.startsWith("tidal:")) return "tidal";
  if (uri.startsWith("local:")) return "local";
  if (uri.startsWith("file:")) return "file";
  if (uri.startsWith("spotify:")) return "spotify";
  if (uri.startsWith("podcast:")) return "podcast";
  if (uri.startsWith("youtube:")) return "youtube";
  return "other";
}

export function artistsOf(track: { artists: { name: string }[] }): string {
  return track.artists.map((a) => a.name).join(", ");
}

export function albumOf(track: { album: { name: string } | null }): string {
  return track.album?.name ?? "";
}

export function truncate(s: string, max: number): string {
  if (s.length <= max) return s;
  return s.slice(0, max - 1) + "…";
}

const EXT_LABEL: Record<string, string> = {
  flac: "FLAC",
  wav: "WAV",
  aif: "AIFF",
  aiff: "AIFF",
  alac: "ALAC",
  m4a: "M4A",
  aac: "AAC",
  mp3: "MP3",
  ogg: "OGG",
  oga: "OGG",
  opus: "OPUS",
  dsf: "DSF",
  dff: "DFF",
  ape: "APE",
  wv: "WAVPACK",
};

/**
 * Best-effort source format extracted from a URI. Currently only handles
 * file/local URIs by parsing the extension; Tidal/Spotify/etc. return null
 * (their source codec needs a server-side lookup we don't have yet).
 */
export function sourceFormat(uri: string | null | undefined): string | null {
  if (!uri) return null;
  if (!uri.startsWith("local:") && !uri.startsWith("file:")) return null;
  // URIs are URL-encoded; the extension survives decode.
  let path: string;
  try {
    path = decodeURIComponent(uri);
  } catch {
    path = uri;
  }
  // Strip trailing query/fragment if any.
  path = path.split(/[?#]/)[0];
  const dot = path.lastIndexOf(".");
  if (dot < 0) return null;
  const ext = path.slice(dot + 1).toLowerCase();
  return EXT_LABEL[ext] ?? null;
}
