import type { SyncedLine } from "./types";

/// Parse LRC text into time-stamped lines, sorted ascending by time. Lines
/// that don't start with `[mm:ss(.cc)]` (header lines like `[ar:]`/`[ti:]`)
/// are skipped. Repeated timestamp lines on a single line (e.g. `[00:01.00][00:32.50]Hello`)
/// are expanded into multiple entries.
export function parseSynced(lrc: string): SyncedLine[] {
  const out: SyncedLine[] = [];
  for (const raw of lrc.split("\n")) {
    const line = raw.trim();
    if (!line.startsWith("[")) continue;
    // Collect all leading [stamp] groups, then the residual text.
    let cursor = 0;
    const stamps: number[] = [];
    while (cursor < line.length && line[cursor] === "[") {
      const close = line.indexOf("]", cursor);
      if (close === -1) break;
      const inner = line.slice(cursor + 1, close);
      const t = parseTimestamp(inner);
      if (t == null) break;
      stamps.push(t);
      cursor = close + 1;
    }
    if (stamps.length === 0) continue;
    const text = line.slice(cursor).trim();
    for (const t of stamps) out.push({ time: t, text });
  }
  out.sort((a, b) => a.time - b.time);
  return out;
}

function parseTimestamp(s: string): number | null {
  // Accept "mm:ss" or "mm:ss.cc" or "hh:mm:ss(.cc)".
  const parts = s.split(":");
  if (parts.length < 2 || parts.length > 3) return null;
  const nums = parts.map((p) => Number(p));
  if (nums.some((n) => Number.isNaN(n))) return null;
  if (parts.length === 2) return nums[0] * 60 + nums[1];
  return nums[0] * 3600 + nums[1] * 60 + nums[2];
}

/// Find the index of the active synced line for a given elapsed time.
/// Returns -1 if no line has started yet.
export function activeLineIndex(lines: SyncedLine[], elapsed: number): number {
  if (lines.length === 0) return -1;
  // Binary search for the last line whose time <= elapsed.
  let lo = 0, hi = lines.length - 1, ans = -1;
  while (lo <= hi) {
    const mid = (lo + hi) >> 1;
    if (lines[mid].time <= elapsed) {
      ans = mid;
      lo = mid + 1;
    } else {
      hi = mid - 1;
    }
  }
  return ans;
}
