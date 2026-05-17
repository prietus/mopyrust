<script lang="ts">
  import { store } from "../lib/store.svelte";
  import { api } from "../lib/api";
  import { fmtTime, fmtMs, fmtAudio, sourceFormat, artistsOf, albumOf, backendOf } from "../lib/format";
  import { parseSynced, activeLineIndex } from "../lib/lrc";
  import Icon from "./Icon.svelte";
  import BackendBadge from "./BackendBadge.svelte";
  import Cover from "./Cover.svelte";
  import Visualizer from "./Visualizer.svelte";

  let seekDrag = $state<number | null>(null);
  let elapsed = $derived(seekDrag ?? store.elapsed);
  let duration = $derived(store.duration);
  let coverUrl = $derived(store.current ? store.covers[store.current.uri] : undefined);
  let year = $derived(store.current?.album?.date ?? store.current?.date);
  let pane = $derived(store.nowPlayingPane);
  let showQueue = $derived(pane === "queue");
  let showLyrics = $derived(pane === "lyrics");
  let showViz = $derived(pane === "viz");

  function onSeekInput(e: Event) {
    seekDrag = +(e.target as HTMLInputElement).value;
  }
  function onSeekChange() {
    if (seekDrag != null) {
      store.setSeek(seekDrag);
      seekDrag = null;
    }
  }
  function onVolume(e: Event) {
    store.setVolume(+(e.target as HTMLInputElement).value);
  }
  function setPane(next: "none" | "queue" | "lyrics" | "viz") {
    store.nowPlayingPane = store.nowPlayingPane === next ? "none" : next;
  }

  // ── queue auto-scroll ─────────────────────────────────────────────────
  let queueListEl = $state<HTMLDivElement | null>(null);
  $effect(() => {
    if (!showQueue) return;
    const tlid = store.currentTlid;
    if (tlid == null || !queueListEl) return;
    const el = queueListEl.querySelector<HTMLElement>(`[data-tlid="${tlid}"]`);
    if (el) el.scrollIntoView({ block: "center", behavior: "smooth" });
  });

  // ── lyrics ────────────────────────────────────────────────────────────
  $effect(() => {
    if (showLyrics && store.current) store.ensureLyrics(store.current);
  });

  let lyrics = $derived(store.current ? store.lyrics[store.current.uri] : undefined);
  let pending = $derived(store.current ? !!store.lyricsPending[store.current.uri] : false);
  let synced = $derived(lyrics?.synced ? parseSynced(lyrics.synced) : []);
  let activeIdx = $derived(synced.length ? activeLineIndex(synced, elapsed) : -1);

  let lyricsListEl = $state<HTMLDivElement | null>(null);
  $effect(() => {
    if (!showLyrics || !lyricsListEl) return;
    if (activeIdx < 0) return;
    const el = lyricsListEl.querySelector<HTMLElement>(`[data-i="${activeIdx}"]`);
    if (el) el.scrollIntoView({ block: "center", behavior: "smooth" });
  });

  function seekToLine(t: number) {
    store.setSeek(t);
  }
</script>

<div
  class="now"
  class:split={pane !== "none"}
  style={coverUrl ? `--bg-image: url(${coverUrl})` : ""}
  class:has-cover={!!coverUrl}
>
  <div class="ambient"></div>
  <div class="scrim"></div>

  <div class="pane-toggles">
    <button
      class="pane-toggle"
      class:active={showLyrics}
      aria-label="letra"
      title="letra"
      onclick={() => setPane("lyrics")}
    >
      <Icon name="wave" size={15} stroke={1.7} />
    </button>
    <button
      class="pane-toggle"
      class:active={showQueue}
      aria-label="queue"
      title="queue"
      onclick={() => setPane("queue")}
    >
      <Icon name="list" size={15} stroke={1.7} />
    </button>
    <button
      class="pane-toggle"
      class:active={showViz}
      aria-label="visualizer"
      title="visualizer"
      onclick={() => setPane("viz")}
    >
      <Icon name="equalizer" size={15} stroke={1.7} />
    </button>
  </div>

  <div class="layout">
    <div class="player">
      <div class="cover-wrap">
        {#if coverUrl}
          <img class="cover" src={coverUrl} alt="" />
        {:else}
          <div class="cover placeholder">
            <Icon name="music" size={64} stroke={1.4} />
          </div>
        {/if}
      </div>

      <div class="info">
        <div class="title">
          {store.current ? (store.current.name || store.current.uri) : "nada sonando"}
        </div>
        <div class="artist">{store.current ? artistsOf(store.current) || "—" : ""}</div>
        <div class="album">{store.current ? albumOf(store.current) : ""}</div>

        <div class="chips">
          {#if store.current}
            <BackendBadge backend={backendOf(store.current.uri)} size="md" />
          {/if}
          {#if year}
            <span class="pill">{String(year).split("-")[0]}</span>
          {/if}
          {#if store.current && sourceFormat(store.current.uri)}
            <span class="pill">{sourceFormat(store.current.uri)}</span>
          {/if}
          {#if store.audioFormat}
            <span class="pill accent">{fmtAudio(store.audioFormat)}</span>
          {/if}
          {#if store.bitrate && store.bitrate > 0}
            <span class="pill">{store.bitrate} kbps</span>
          {/if}
        </div>
      </div>

      <div class="transport">
        <button class="icon-btn" aria-label="prev" onclick={() => api.previous()}>
          <Icon name="prev" size={20} stroke={1.4} />
        </button>
        <button class="icon-btn primary big" aria-label="play/pause" onclick={() => store.togglePlay()}>
          <Icon name={store.isPlaying ? "pause" : "play"} size={24} stroke={1.4} />
        </button>
        <button class="icon-btn" aria-label="next" onclick={() => api.next()}>
          <Icon name="next" size={20} stroke={1.4} />
        </button>
      </div>

      <div class="progress">
        <span class="time">{fmtTime(elapsed)}</span>
        <input
          type="range"
          min="0"
          max={Math.max(duration, elapsed, 1)}
          step="1"
          value={elapsed}
          oninput={onSeekInput}
          onchange={onSeekChange}
        />
        <span class="time">{fmtTime(duration)}</span>
      </div>

      {#if store.hasVolume}
        <div class="volume-row">
          <Icon name={store.volume === 0 ? "volume-mute" : "volume"} size={14} stroke={1.6} />
          <input type="range" min="0" max="100" step="1" value={store.volume} oninput={onVolume} />
          <span class="vol-num">{store.volume}</span>
        </div>
      {/if}
    </div>

    {#if showLyrics}
      <aside class="lyrics-pane">
        <div class="qhead">
          <span class="qtitle">Lyrics</span>
          {#if synced.length}<span class="qcount">synced · {synced.length} lines</span>
          {:else if lyrics?.plain}<span class="qcount">plain text</span>
          {/if}
        </div>
        <div class="lyrics-body" bind:this={lyricsListEl}>
          {#if pending && !lyrics}
            <div class="qempty">searching lyrics…</div>
          {:else if lyrics?.instrumental}
            <div class="qempty">instrumental</div>
          {:else if synced.length > 0}
            {#each synced as line, i (i)}
              <div
                class="lline"
                class:active={i === activeIdx}
                class:upcoming={i > activeIdx}
                data-i={i}
                role="button"
                tabindex="0"
                onclick={() => seekToLine(line.time)}
                onkeydown={(e) => {
                  if (e.key === "Enter" || e.key === " ") { e.preventDefault(); seekToLine(line.time); }
                }}
              >
                {line.text || "♪"}
              </div>
            {/each}
          {:else if lyrics?.plain}
            <pre class="plain">{lyrics.plain}</pre>
          {:else}
            <div class="qempty">
              <div>no lyrics found</div>
              <button
                class="retry"
                onclick={() => store.retryLyrics(store.current)}
                disabled={pending}
              >
                {pending ? "searching…" : "retry"}
              </button>
            </div>
          {/if}
        </div>
      </aside>
    {:else if showQueue}
      <aside class="queue-pane">
        <div class="qhead">
          <span class="qtitle">Up next</span>
          <span class="qcount">{store.queue.length}</span>
        </div>
        <div class="qlist" bind:this={queueListEl}>
          {#if store.queue.length === 0}
            <div class="qempty">queue is empty</div>
          {:else}
            {#each store.queue as tl (tl.tlid)}
              {@const isCurrent = store.currentTlid === tl.tlid}
              <div
                class="qrow"
                class:current={isCurrent}
                data-tlid={tl.tlid}
                role="button"
                tabindex="0"
                onclick={() => api.playTlid(tl.tlid)}
                onkeydown={(e) => {
                  if (e.key === "Enter" || e.key === " ") {
                    e.preventDefault();
                    api.playTlid(tl.tlid);
                  }
                }}
              >
                <Cover uri={tl.track.uri} size={36} radius="var(--radius-sm)" />
                <div class="qmeta truncate">
                  <div class="qtitle-row truncate">
                    {#if isCurrent}<span class="qdot">●</span>{/if}
                    {tl.track.name || tl.track.uri}
                  </div>
                  <div class="qsub truncate">{artistsOf(tl.track) || "—"}</div>
                </div>
                <span class="qdur">{fmtMs(tl.track.length)}</span>
              </div>
            {/each}
          {/if}
        </div>
      </aside>
    {:else if showViz}
      <Visualizer />
    {/if}
  </div>
</div>

<style>
  .now {
    position: relative;
    width: 100%;
    height: 100%;
    overflow: hidden;
    isolation: isolate;
  }
  .ambient {
    position: absolute;
    inset: -10%;
    background-image: var(--bg-image);
    background-size: cover;
    background-position: center;
    filter: blur(80px) saturate(1.2);
    opacity: 0.35;
    z-index: -2;
    transition: opacity 600ms ease;
  }
  .now:not(.has-cover) .ambient { opacity: 0; }
  .scrim {
    position: absolute;
    inset: 0;
    background: linear-gradient(180deg, rgba(0, 0, 0, 0.15), var(--bg-0) 70%);
    z-index: -1;
  }

  .pane-toggles {
    position: absolute;
    top: 16px;
    right: 16px;
    z-index: 5;
    display: flex;
    gap: 6px;
  }
  .pane-toggle {
    width: 32px;
    height: 32px;
    border-radius: var(--radius-pill);
    color: var(--text-mid);
    background: var(--bg-2);
    border: 1px solid var(--border-soft);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    transition: background 120ms ease, color 120ms ease, border-color 120ms ease;
  }
  .pane-toggle:hover {
    background: var(--bg-hover);
    color: var(--text);
  }
  .pane-toggle.active {
    background: var(--accent-soft);
    color: var(--accent);
    border-color: var(--accent-ring);
  }

  .layout {
    height: 100%;
    display: grid;
    grid-template-columns: 1fr;
  }
  .now.split .layout {
    grid-template-columns: 1fr minmax(280px, 360px);
  }

  .player {
    overflow: auto;
    padding: 56px 32px 72px;
    display: flex;
    flex-direction: column;
    align-items: center;
  }
  .now.split .player { padding: 40px 24px 60px; }

  .cover-wrap {
    width: 280px;
    height: 280px;
    margin-bottom: 28px;
  }
  .now.split .cover-wrap {
    width: 220px;
    height: 220px;
    margin-bottom: 22px;
  }
  .cover {
    width: 100%;
    height: 100%;
    border-radius: var(--radius-lg);
    object-fit: cover;
    box-shadow: var(--shadow-strong);
    background: var(--bg-2);
  }
  .cover.placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-faint);
    background: linear-gradient(135deg, var(--bg-2), var(--bg-1));
    border: 1px solid var(--border-soft);
  }

  .info {
    text-align: center;
    margin-bottom: 24px;
    max-width: 480px;
  }
  .title {
    font-size: 20px;
    font-weight: 700;
    color: var(--text);
    letter-spacing: -0.01em;
    margin-bottom: 6px;
    line-height: 1.25;
  }
  .now.split .title { font-size: 17px; }
  .artist {
    font-size: 13px;
    color: var(--text-mid);
    font-weight: 500;
    margin-bottom: 2px;
  }
  .album {
    font-size: 11.5px;
    color: var(--text-muted);
    margin-bottom: 14px;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 6px;
  }

  .transport {
    display: flex;
    align-items: center;
    gap: 18px;
    margin-bottom: 18px;
  }
  .icon-btn.big {
    width: 56px;
    height: 56px;
  }

  .progress {
    display: grid;
    grid-template-columns: 44px 1fr 44px;
    align-items: center;
    gap: 12px;
    width: 100%;
    max-width: 380px;
    margin-bottom: 12px;
  }
  .time {
    font-size: 11px;
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
    text-align: center;
  }

  .volume-row {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    max-width: 280px;
    color: var(--text-muted);
  }
  .volume-row input[type="range"] { flex: 1; }
  .vol-num {
    width: 28px;
    text-align: right;
    font-size: 10.5px;
    font-variant-numeric: tabular-nums;
  }

  /* ── queue pane ───────────────────────────────────────────────── */

  .queue-pane {
    border-left: 1px solid var(--border-soft);
    background: rgba(0, 0, 0, 0.18);
    backdrop-filter: blur(10px);
    -webkit-backdrop-filter: blur(10px);
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }
  .qhead {
    padding: 16px 18px 10px;
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 10px;
    border-bottom: 1px solid var(--border-soft);
  }
  .qtitle {
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--text-mid);
  }
  .qcount {
    font-size: 11px;
    color: var(--text-faint);
  }
  .qlist {
    flex: 1;
    overflow: auto;
    padding: 6px 8px 16px;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .qempty {
    padding: 32px;
    text-align: center;
    color: var(--text-faint);
    font-size: 12px;
  }
  .qrow {
    display: grid;
    grid-template-columns: 36px 1fr auto;
    gap: 10px;
    align-items: center;
    padding: 6px 10px;
    border-radius: var(--radius-md);
    text-align: left;
    transition: background 120ms ease;
    color: var(--text);
    cursor: pointer;
  }
  .qrow:hover { background: var(--bg-hover); }
  .qrow.current { background: var(--accent-soft); }
  .qrow.current .qtitle-row { color: var(--accent); }
  .qmeta {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
  }
  .qtitle-row {
    font-size: 12.5px;
    font-weight: 500;
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .qdot {
    color: var(--accent);
    font-size: 9px;
  }
  .qsub {
    font-size: 10.5px;
    color: var(--text-muted);
  }
  .qdur {
    font-size: 10.5px;
    color: var(--text-faint);
    font-variant-numeric: tabular-nums;
  }

  /* ── lyrics pane ─────────────────────────────────────────────── */

  .lyrics-pane {
    border-left: 1px solid var(--border-soft);
    background: rgba(0, 0, 0, 0.18);
    backdrop-filter: blur(10px);
    -webkit-backdrop-filter: blur(10px);
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }
  .lyrics-body {
    flex: 1;
    overflow: auto;
    padding: 24px 22px 32vh; /* tail padding so the active line can center */
  }
  .lline {
    display: block;
    background: transparent;
    border: 0;
    width: 100%;
    text-align: left;
    padding: 6px 6px;
    border-radius: var(--radius-sm);
    font-size: 14.5px;
    line-height: 1.4;
    color: var(--text-faint);
    cursor: pointer;
    transition: color 250ms ease, transform 250ms ease, background 120ms ease;
  }
  .lline:hover {
    background: var(--bg-hover);
  }
  .lline.upcoming {
    color: var(--text-faint);
  }
  .lline.active {
    color: var(--text);
    font-weight: 600;
    transform: translateX(2px);
  }
  .plain {
    font-family: inherit;
    font-size: 13px;
    line-height: 1.55;
    color: var(--text-mid);
    white-space: pre-wrap;
    word-break: break-word;
    margin: 0;
  }
  .qempty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    padding: 32px 24px;
    text-align: center;
    color: var(--text-faint);
    font-size: 12.5px;
  }
  .retry {
    padding: 6px 14px;
    border-radius: var(--radius-md);
    background: var(--bg-2);
    color: var(--text-mid);
    border: 1px solid var(--border-soft);
    font-size: 11.5px;
    font-weight: 500;
    transition: background 120ms ease, color 120ms ease;
  }
  .retry:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--text);
  }
</style>
