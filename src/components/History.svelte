<script lang="ts">
  import { store } from "../lib/store.svelte";
  import { fmtMs, backendOf } from "../lib/format";
  import BackendBadge from "./BackendBadge.svelte";
  import Cover from "./Cover.svelte";
  import Icon from "./Icon.svelte";

  type Tab = "stats" | "recent" | "most";
  let tab = $state<Tab>("stats");

  $effect(() => {
    if (!store.statsAvailable) return;
    if (tab === "recent") store.ensureRecentPlays();
    else if (tab === "most") store.ensureMostPlayed();
    else if (tab === "stats") {
      store.ensureStatsTotals();
      store.ensureTopArtists();
      store.ensureTopAlbums();
      store.ensureByGenre();
      store.ensureByDayOfWeek();
      store.ensureByHour();
    }
  });

  function timeAgo(unix: number): string {
    const diff = Date.now() / 1000 - unix;
    if (diff < 60) return "just now";
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    if (diff < 604800) return `${Math.floor(diff / 86400)}d ago`;
    return new Date(unix * 1000).toLocaleDateString();
  }

  function fmtTotalTime(ms: number): string {
    const sec = Math.floor(ms / 1000);
    const days = Math.floor(sec / 86400);
    const hours = Math.floor((sec % 86400) / 3600);
    const mins = Math.floor((sec % 3600) / 60);
    if (days > 0) return `${days}d ${hours}h`;
    if (hours > 0) return `${hours}h ${mins}m`;
    return `${mins}m`;
  }

  function fmtHM(ms: number): string {
    const total = Math.floor(ms / 60000);
    const h = Math.floor(total / 60);
    const m = total % 60;
    if (h > 0) return `${h}h ${m}m`;
    return `${m}m`;
  }

  function playTrack(uri: string) {
    store.playUri(uri);
  }

  // Donut geometry — single circle, segments via stroke-dasharray.
  const DONUT_RADIUS = 56;
  const DONUT_CIRC = 2 * Math.PI * DONUT_RADIUS;

  // Stable colors for donut segments. A small palette is enough when we cap to
  // ~8 visible genres + "Other".
  const PALETTE = [
    "#e87a8a", "#c4a35a", "#82a878", "#7aa6c8",
    "#a888c4", "#c89a78", "#88c4b0", "#c878a0",
  ];

  let genreSegments = $derived.by(() => {
    const g = store.byGenre ?? [];
    if (g.length === 0) return null;
    const top = g.slice(0, 8);
    const restMs = g.slice(8).reduce((a, x) => a + x.total_played_ms, 0);
    const segments = top.map((x, i) => ({
      genre: x.genre,
      total_played_ms: x.total_played_ms,
      color: PALETTE[i % PALETTE.length],
    }));
    if (restMs > 0) {
      segments.push({ genre: "Other", total_played_ms: restMs, color: "#666" });
    }
    const sum = segments.reduce((a, x) => a + x.total_played_ms, 0) || 1;
    let cumulative = 0;
    return segments.map((s) => {
      const frac = s.total_played_ms / sum;
      const dash = frac * DONUT_CIRC;
      const offset = -cumulative * DONUT_CIRC;
      cumulative += frac;
      return { ...s, dash, offset };
    });
  });

  const DOW_LABELS = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

  let dowSorted = $derived.by(() => {
    if (!store.byDayOfWeek) return null;
    // Reorder Sun-first → Mon-first (more European)
    const arr = [...store.byDayOfWeek];
    const sun = arr.shift();
    if (sun) arr.push(sun);
    return arr;
  });
  let dowMax = $derived.by(() => {
    return Math.max(1, ...(store.byDayOfWeek?.map((d) => d.total_played_ms) ?? [0]));
  });
  let dowAvg = $derived.by(() => {
    if (!store.byDayOfWeek?.length) return 0;
    const sum = store.byDayOfWeek.reduce((a, d) => a + d.total_played_ms, 0);
    return sum / store.byDayOfWeek.length;
  });

  let hourMax = $derived.by(() => {
    return Math.max(1, ...(store.byHour?.map((h) => h.total_played_ms) ?? [0]));
  });
  let hourPeak = $derived.by(() => {
    if (!store.byHour) return null;
    return store.byHour.reduce((best, h) => h.total_played_ms > best.total_played_ms ? h : best);
  });
</script>

<div class="page">
  <header class="page-header">
    <h2>History</h2>
    {#if store.statsAvailable}
      <div class="tabs">
        <button class:active={tab === "stats"} onclick={() => (tab = "stats")}>Stats</button>
        <button class:active={tab === "recent"} onclick={() => (tab = "recent")}>Recent</button>
        <button class:active={tab === "most"} onclick={() => (tab = "most")}>Most played</button>
      </div>
    {/if}
  </header>

  <div class="body">
    {#if !store.goodiesProbed}
      <div class="empty">probing…</div>
    {:else if !store.statsAvailable}
      <div class="install">
        <div class="install-icon">
          <Icon name="clock" size={32} stroke={1.4} />
        </div>
        <div class="install-title">Install <code>mopidy-tidal-goodies</code> on your Mopidy server to enable history.</div>
        <pre class="install-cmd">pip install --user mopidy-tidal-goodies
systemctl --user restart mopidy</pre>
        <a class="install-link" href="https://github.com/prietus/mopidy-tidal-goodies" target="_blank" rel="noreferrer">github.com/prietus/mopidy-tidal-goodies</a>
      </div>

    {:else if tab === "stats"}
      <!-- Hero -->
      {#if store.statsTotals}
        <section class="hero">
          <div class="hero-left">
            <div class="hero-icon"><Icon name="wave" size={28} stroke={1.6} /></div>
            <div class="hero-time">{fmtTotalTime(store.statsTotals.total_played_ms)}</div>
            <div class="hero-time-label">total listening time</div>
          </div>
          <div class="hero-right">
            <div class="hero-stat">
              <span class="hero-stat-num">{store.statsTotals.total_plays.toLocaleString()}</span>
              <span class="hero-stat-label">plays</span>
            </div>
            <div class="hero-stat">
              <span class="hero-stat-num">{store.statsTotals.unique_tracks.toLocaleString()}</span>
              <span class="hero-stat-label">tracks</span>
            </div>
            <div class="hero-stat">
              <span class="hero-stat-num">{store.statsTotals.unique_artists.toLocaleString()}</span>
              <span class="hero-stat-label">artists</span>
            </div>
            <div class="hero-stat">
              <span class="hero-stat-num">{store.statsTotals.unique_albums.toLocaleString()}</span>
              <span class="hero-stat-label">albums</span>
            </div>
          </div>
        </section>
      {/if}

      <div class="grid">
        <!-- Genres -->
        <section class="card">
          <h3>Genres</h3>
          {#if genreSegments && genreSegments.length}
            <div class="donut-row">
              <svg class="donut" viewBox="-70 -70 140 140">
                {#each genreSegments as s, i (i)}
                  <circle
                    r={DONUT_RADIUS}
                    fill="none"
                    stroke={s.color}
                    stroke-width="14"
                    stroke-dasharray={`${s.dash} ${DONUT_CIRC}`}
                    stroke-dashoffset={s.offset}
                    transform="rotate(-90)"
                  />
                {/each}
                <text class="donut-num" text-anchor="middle" dy="-2">{store.byGenre?.length ?? 0}</text>
                <text class="donut-label" text-anchor="middle" dy="14">genres</text>
              </svg>
              <ul class="legend">
                {#each genreSegments as s, i (i)}
                  <li>
                    <span class="dot" style="background: {s.color}"></span>
                    <span class="lbl truncate">{s.genre}</span>
                    <span class="val">{fmtHM(s.total_played_ms)}</span>
                  </li>
                {/each}
              </ul>
            </div>
          {:else if store.byGenreLoading}
            <div class="card-empty">loading…</div>
          {:else}
            <div class="card-empty">
              no genre data yet
              <div class="card-empty-sub">capture starts with v0.3 of the goodies plugin</div>
            </div>
          {/if}
        </section>

        <!-- Top Artists -->
        <section class="card">
          <h3>Top Artists</h3>
          {#if store.topArtists?.length}
            <ul class="rank-list">
              {#each store.topArtists as a, i (a.artist + i)}
                <li>
                  <span class="rank-num">{i + 1}</span>
                  {#if a.sample_album_uri}
                    <Cover uri={a.sample_album_uri} size={32} radius="50%" />
                  {:else}
                    <span class="rank-avatar"><Icon name="user" size={14} stroke={1.6} /></span>
                  {/if}
                  <span class="rank-name truncate">{a.artist}</span>
                  <span class="rank-val">{fmtHM(a.total_played_ms)}</span>
                </li>
              {/each}
            </ul>
          {:else if store.topArtistsLoading}
            <div class="card-empty">loading…</div>
          {:else}
            <div class="card-empty">no plays yet</div>
          {/if}
        </section>

        <!-- Top Albums -->
        <section class="card">
          <h3>Top Albums</h3>
          {#if store.topAlbums?.length}
            <ul class="rank-list">
              {#each store.topAlbums as a, i (a.album_uri ?? (a.artist + a.album))}
                <li>
                  <button
                    class="rank-clickable"
                    onclick={() => a.album_uri && store.navTo({ kind: "album-detail", uri: a.album_uri, label: a.album })}
                    disabled={!a.album_uri}
                  >
                    {#if a.album_uri}
                      <Cover uri={a.album_uri} size={36} radius="var(--radius-sm)" />
                    {:else}
                      <span class="rank-avatar sq"><Icon name="disc" size={14} stroke={1.6} /></span>
                    {/if}
                    <div class="rank-meta truncate">
                      <div class="rank-name truncate">{a.album}</div>
                      <div class="rank-sub truncate">{a.artist}</div>
                    </div>
                    <span class="rank-val">{fmtHM(a.total_played_ms)}</span>
                  </button>
                </li>
              {/each}
            </ul>
          {:else if store.topAlbumsLoading}
            <div class="card-empty">loading…</div>
          {:else}
            <div class="card-empty">no plays yet</div>
          {/if}
        </section>

        <!-- Day of Week -->
        <section class="card">
          <h3>Day of Week</h3>
          {#if dowSorted}
            <div class="bars dow">
              {#each dowSorted as d, i (i)}
                {@const idx = (d.dow + 6) % 7}
                <div class="bar-wrap" title="{DOW_LABELS[d.dow]}: {fmtHM(d.total_played_ms)}">
                  <div class="bar" style="height: {(d.total_played_ms / dowMax) * 100}%"></div>
                  <span class="bar-label">{DOW_LABELS[d.dow].slice(0, 3)}</span>
                  <span class="bar-val">{fmtHM(d.total_played_ms)}</span>
                </div>
              {/each}
            </div>
            <div class="bars-foot">avg {fmtHM(dowAvg)}/day</div>
          {:else if store.byDayOfWeekLoading}
            <div class="card-empty">loading…</div>
          {/if}
        </section>

        <!-- Listening Hours -->
        <section class="card">
          <h3>Listening Hours</h3>
          {#if store.byHour}
            <div class="bars hour">
              {#each store.byHour as h, i (i)}
                <div class="bar-wrap thin" title="{h.hour}:00 · {fmtHM(h.total_played_ms)}">
                  <div class="bar" style="height: {(h.total_played_ms / hourMax) * 100}%"></div>
                </div>
              {/each}
            </div>
            <div class="hour-axis">
              <span>0h</span><span>6h</span><span>12h</span><span>18h</span>
            </div>
            {#if hourPeak && hourPeak.total_played_ms > 0}
              <div class="bars-foot">peak listening: {String(hourPeak.hour).padStart(2,"0")}:00</div>
            {/if}
          {:else if store.byHourLoading}
            <div class="card-empty">loading…</div>
          {/if}
        </section>
      </div>

    {:else if tab === "recent"}
      {#if !store.recentPlays && store.recentPlaysLoading}
        <div class="empty">loading…</div>
      {:else if !store.recentPlays?.length}
        <div class="empty">
          <div class="empty-title">no plays yet</div>
          <div class="empty-sub">start listening — finished tracks land here</div>
        </div>
      {:else}
        <div class="rows">
          {#each store.recentPlays as p, i (p.played_at + ":" + i)}
            <button class="row" onclick={() => playTrack(p.track_uri)}>
              <div class="meta truncate">
                <div class="title truncate">{p.name || p.track_uri}</div>
                <div class="sub truncate">
                  {p.artist || "—"}{#if p.album} · {p.album}{/if}
                </div>
              </div>
              <BackendBadge backend={backendOf(p.track_uri)} />
              <span class="when">{timeAgo(p.played_at)}</span>
              <span class="dur" class:partial={!p.completed}>{fmtMs(p.played_ms)}</span>
            </button>
          {/each}
        </div>
      {/if}

    {:else if tab === "most"}
      {#if !store.mostPlayed && store.mostPlayedLoading}
        <div class="empty">loading…</div>
      {:else if !store.mostPlayed?.length}
        <div class="empty">no data yet</div>
      {:else}
        <div class="rows">
          {#each store.mostPlayed as t, i (t.track_uri)}
            <button class="row most" onclick={() => playTrack(t.track_uri)}>
              <span class="rank">{i + 1}</span>
              <div class="meta truncate">
                <div class="title truncate">{t.name || t.track_uri}</div>
                <div class="sub truncate">
                  {t.artist || "—"}{#if t.album} · {t.album}{/if}
                </div>
              </div>
              <BackendBadge backend={backendOf(t.track_uri)} />
              <span class="plays">{t.plays} {t.plays === 1 ? "play" : "plays"}</span>
            </button>
          {/each}
        </div>
      {/if}
    {/if}
  </div>
</div>

<style>
  .page { height: 100%; display: flex; flex-direction: column; }
  .page-header {
    padding: 24px 28px 14px;
    display: flex; align-items: center; justify-content: space-between;
    gap: 16px;
    border-bottom: 1px solid var(--border-soft);
    background: var(--bg-0);
    position: sticky; top: 0; z-index: 1;
    flex-shrink: 0;
  }
  h2 { margin: 0; font-size: 17px; font-weight: 700; letter-spacing: -0.01em; }
  .tabs {
    display: inline-flex;
    background: var(--bg-2);
    border-radius: var(--radius-md);
    padding: 3px; gap: 2px;
  }
  .tabs button {
    padding: 5px 12px;
    background: transparent;
    border-radius: calc(var(--radius-md) - 3px);
    color: var(--text-muted); font-size: 12px; font-weight: 500;
    cursor: pointer;
    transition: background 100ms ease, color 100ms ease;
  }
  .tabs button:hover { color: var(--text); }
  .tabs button.active {
    background: var(--bg-1); color: var(--text);
    box-shadow: var(--shadow-soft);
  }
  .body { flex: 1; overflow: auto; padding: 18px 24px 32px; }

  .empty { padding: 60px 32px; text-align: center; color: var(--text-muted); }
  .empty-title { font-size: 13.5px; font-weight: 600; color: var(--text-mid); margin-bottom: 4px; }
  .empty-sub { font-size: 11.5px; color: var(--text-faint); }

  .install { max-width: 480px; margin: 60px auto; text-align: center; color: var(--text-muted); }
  .install-icon { color: var(--text-faint); margin-bottom: 14px; }
  .install-title { font-size: 13.5px; color: var(--text-mid); margin-bottom: 16px; line-height: 1.5; }
  .install-title code {
    background: var(--bg-2); padding: 1px 6px; border-radius: 4px;
    font-size: 12px; color: var(--accent);
  }
  .install-cmd {
    background: var(--bg-2); border: 1px solid var(--border-soft);
    border-radius: var(--radius-md);
    padding: 12px 16px; font-size: 11.5px; color: var(--text);
    text-align: left; margin: 0 0 16px 0; overflow-x: auto;
  }
  .install-link { font-size: 11.5px; color: var(--accent); text-decoration: none; }
  .install-link:hover { text-decoration: underline; }

  /* ── Hero ────────────────────────────────────────────────── */
  .hero {
    display: flex; align-items: center; gap: 28px;
    padding: 28px 28px;
    background: var(--bg-1);
    border: 1px solid var(--border-soft);
    border-radius: var(--radius-lg);
    margin-bottom: 18px;
  }
  .hero-left { flex: 1; display: flex; flex-direction: column; align-items: center; gap: 4px; }
  .hero-icon { color: var(--accent); margin-bottom: 4px; }
  .hero-time { font-size: 36px; font-weight: 800; letter-spacing: -0.03em; color: var(--text); }
  .hero-time-label { font-size: 10.5px; color: var(--text-faint); text-transform: uppercase; letter-spacing: 0.06em; }
  .hero-right { display: grid; grid-template-columns: repeat(2, 1fr); gap: 10px 32px; min-width: 280px; }
  .hero-stat { display: flex; align-items: baseline; gap: 8px; }
  .hero-stat-num { font-size: 18px; font-weight: 700; color: var(--text); font-variant-numeric: tabular-nums; }
  .hero-stat-label { font-size: 11px; color: var(--text-muted); text-transform: lowercase; }

  /* ── Cards grid ──────────────────────────────────────────── */
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 14px;
  }
  .card {
    background: var(--bg-1);
    border: 1px solid var(--border-soft);
    border-radius: var(--radius-md);
    padding: 16px 18px;
    display: flex; flex-direction: column;
    gap: 10px;
    min-height: 220px;
  }
  .card h3 {
    margin: 0;
    font-size: 11px; font-weight: 700;
    text-transform: uppercase; letter-spacing: 0.06em;
    color: var(--text-mid);
  }
  .card-empty {
    color: var(--text-faint); font-size: 12px;
    padding: 28px 0; text-align: center; flex: 1;
    display: flex; flex-direction: column; justify-content: center; gap: 4px;
  }
  .card-empty-sub { font-size: 10.5px; color: var(--text-faint); opacity: 0.7; }

  /* ── Donut ───────────────────────────────────────────────── */
  .donut-row { display: flex; gap: 14px; align-items: center; }
  .donut { width: 130px; height: 130px; flex-shrink: 0; }
  .donut-num { fill: var(--text); font-size: 20px; font-weight: 700; }
  .donut-label { fill: var(--text-faint); font-size: 9px; text-transform: uppercase; letter-spacing: 0.08em; }
  .legend {
    list-style: none; padding: 0; margin: 0;
    display: flex; flex-direction: column; gap: 5px;
    flex: 1; min-width: 0;
    font-size: 11px;
  }
  .legend li { display: grid; grid-template-columns: 8px 1fr auto; gap: 8px; align-items: center; }
  .legend .dot { width: 8px; height: 8px; border-radius: 50%; }
  .legend .lbl { color: var(--text-mid); }
  .legend .val { color: var(--text-faint); font-variant-numeric: tabular-nums; }

  /* ── Rank list (top artists / albums) ────────────────────── */
  .rank-list {
    list-style: none; padding: 0; margin: 0;
    display: flex; flex-direction: column; gap: 4px;
    flex: 1;
  }
  .rank-list > li { display: contents; }
  .rank-list > li > * {
    display: grid; grid-template-columns: 18px auto 1fr auto;
    gap: 10px; align-items: center;
    padding: 4px 0;
    background: transparent; border: none;
    font-size: 12px; color: var(--text);
    text-align: left;
  }
  .rank-clickable { cursor: pointer; padding: 4px 6px; border-radius: var(--radius-sm); transition: background 80ms ease; }
  .rank-clickable:hover:not(:disabled) { background: var(--bg-hover); }
  .rank-clickable:disabled { cursor: default; }
  .rank-num { font-size: 10.5px; color: var(--text-faint); font-variant-numeric: tabular-nums; text-align: right; }
  .rank-avatar {
    width: 32px; height: 32px; border-radius: 50%;
    background: var(--bg-2);
    display: inline-flex; align-items: center; justify-content: center;
    color: var(--text-faint);
  }
  .rank-avatar.sq { width: 36px; height: 36px; border-radius: var(--radius-sm); }
  .rank-name { font-size: 12px; font-weight: 500; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .rank-meta { display: flex; flex-direction: column; gap: 1px; min-width: 0; }
  .rank-sub { font-size: 10.5px; color: var(--text-muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .rank-val { font-size: 11px; color: var(--text-faint); font-variant-numeric: tabular-nums; white-space: nowrap; }

  /* ── Bar charts ──────────────────────────────────────────── */
  .bars {
    display: flex; align-items: flex-end;
    height: 130px;
    gap: 8px;
    padding: 8px 0;
    flex: 1;
  }
  .bars.hour { gap: 2px; padding: 4px 0; height: 110px; }
  .bar-wrap {
    flex: 1; display: flex; flex-direction: column; align-items: center;
    gap: 4px;
    height: 100%;
    justify-content: flex-end;
    min-width: 0;
  }
  .bar-wrap.thin { gap: 0; }
  .bar {
    width: 100%; max-width: 28px;
    min-height: 1px;
    background: var(--accent);
    border-radius: 2px 2px 0 0;
    transition: height 200ms ease-out;
  }
  .bar-wrap.thin .bar { max-width: none; opacity: 0.85; }
  .bar-label { font-size: 10px; color: var(--text-muted); }
  .bar-val { font-size: 9.5px; color: var(--text-faint); font-variant-numeric: tabular-nums; }
  .bars-foot { font-size: 10.5px; color: var(--text-faint); margin-top: 2px; }
  .hour-axis {
    display: flex; justify-content: space-between;
    font-size: 10px; color: var(--text-faint);
    padding: 0 4px;
  }

  /* ── Recent / most-played list (unchanged) ──────────────── */
  .rows { display: flex; flex-direction: column; gap: 1px; }
  .row {
    display: grid;
    grid-template-columns: 1fr auto auto auto;
    gap: 12px; align-items: center; width: 100%;
    padding: 8px 12px;
    border-radius: var(--radius-md);
    background: transparent; text-align: left; color: var(--text);
    transition: background 120ms ease;
    cursor: pointer;
  }
  .row.most { grid-template-columns: 28px 1fr auto auto; }
  .row:hover { background: var(--bg-hover); }
  .rank { font-size: 11px; color: var(--text-faint); font-variant-numeric: tabular-nums; text-align: right; }
  .meta { display: flex; flex-direction: column; gap: 2px; min-width: 0; }
  .title { font-size: 13px; font-weight: 500; }
  .sub { font-size: 11px; color: var(--text-muted); }
  .when { font-size: 11px; color: var(--text-faint); font-variant-numeric: tabular-nums; min-width: 70px; text-align: right; }
  .dur { font-size: 11px; color: var(--text-faint); font-variant-numeric: tabular-nums; width: 48px; text-align: right; }
  .dur.partial { color: color-mix(in oklab, var(--text-faint), red 25%); }
  .plays { font-size: 11px; color: var(--text-mid); font-variant-numeric: tabular-nums; min-width: 72px; text-align: right; }
</style>
