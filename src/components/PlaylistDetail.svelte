<script lang="ts">
  import { store } from "../lib/store.svelte";
  import { fmtMs, backendOf } from "../lib/format";
  import TrackRow from "./TrackRow.svelte";
  import BackendBadge from "./BackendBadge.svelte";
  import Icon from "./Icon.svelte";

  type Props = { uri: string; label: string };
  let { uri, label }: Props = $props();

  let pl = $derived(store.playlistDetail[uri]);
  let loading = $derived(!!store.playlistLoading[uri]);
  let tracks = $derived(pl?.tracks ?? []);
  let runtime = $derived(tracks.reduce((a, t) => a + (t.length ?? 0), 0));

  function playAll() {
    if (tracks.length) store.playUris(tracks.map((t) => t.uri));
  }
  function queueAll() {
    if (tracks.length) store.enqueueUris(tracks.map((t) => t.uri));
  }
</script>

<div class="page">
  <div class="back-row">
    <button class="back" onclick={() => store.navBack()}>
      <Icon name="chevron-left" size={14} stroke={1.8} />
      <span>playlists</span>
    </button>
  </div>

  <header class="hero">
    <div class="cover">
      <Icon name="list" size={56} stroke={1.4} />
    </div>
    <div class="info">
      <div class="title">{pl?.name || label}</div>
      <div class="chips">
        <BackendBadge backend={backendOf(uri)} size="md" />
        {#if tracks.length}
          <span class="pill">{tracks.length} {tracks.length === 1 ? "track" : "tracks"}</span>
        {/if}
        {#if runtime > 0}
          <span class="pill">{fmtMs(runtime)}</span>
        {/if}
      </div>
      <div class="actions">
        <button class="btn primary" onclick={playAll} disabled={!tracks.length}>
          <Icon name="play" size={13} />
          play all
        </button>
        <button class="btn" onclick={queueAll} disabled={!tracks.length}>
          <Icon name="plus" size={13} stroke={2} />
          add to queue
        </button>
      </div>
    </div>
  </header>

  <div class="tracks">
    {#if loading && tracks.length === 0}
      <div class="empty">loading tracks…</div>
    {:else if tracks.length === 0}
      <div class="empty">empty playlist</div>
    {:else}
      {#each tracks as t, i (i + "-" + t.uri)}
        <TrackRow track={t} index={i} />
      {/each}
    {/if}
  </div>
</div>

<style>
  .page {
    height: 100%;
    overflow: auto;
  }
  .back-row { padding: 18px 24px 0; }
  .back {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 6px 10px;
    border-radius: var(--radius-md);
    color: var(--text-mid);
    font-size: 12px;
    font-weight: 500;
    background: transparent;
    transition: background 120ms ease, color 120ms ease;
  }
  .back:hover { background: var(--bg-hover); color: var(--text); }

  .hero {
    display: flex;
    gap: 20px;
    align-items: flex-end;
    padding: 16px 28px 24px;
    border-bottom: 1px solid var(--border-soft);
  }
  .cover {
    width: 180px;
    height: 180px;
    border-radius: var(--radius-lg);
    background: linear-gradient(135deg, var(--bg-2), var(--bg-1));
    border: 1px solid var(--border-soft);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-faint);
    box-shadow: var(--shadow-soft);
    flex-shrink: 0;
  }
  .info {
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-width: 0;
    flex: 1;
  }
  .title {
    font-size: 22px;
    font-weight: 700;
    color: var(--text);
    letter-spacing: -0.01em;
    line-height: 1.2;
  }
  .chips { display: flex; flex-wrap: wrap; gap: 6px; }
  .actions { display: flex; gap: 8px; margin-top: 4px; }

  .tracks {
    padding: 12px 24px 32px;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .empty {
    padding: 40px;
    text-align: center;
    color: var(--text-faint);
    font-size: 12px;
  }
</style>
