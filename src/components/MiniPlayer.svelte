<script lang="ts">
  import { store } from "../lib/store.svelte";
  import { api } from "../lib/api";
  import { fmtTime, fmtAudio, artistsOf, albumOf, backendOf } from "../lib/format";
  import Icon from "./Icon.svelte";
  import Cover from "./Cover.svelte";
  import BackendBadge from "./BackendBadge.svelte";

  let seekDrag = $state<number | null>(null);
  let elapsed = $derived(seekDrag ?? store.elapsed);
  let duration = $derived(store.duration);

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
</script>

<div class="mini">
  <div class="left">
    <Cover uri={store.current?.uri} size={52} radius="var(--radius-sm)" />
    <div class="meta truncate">
      <div class="title truncate">
        {store.current ? store.current.name || store.current.uri : "nada sonando"}
      </div>
      <div class="sub truncate">
        {store.current ? `${artistsOf(store.current) || "—"} · ${albumOf(store.current) || "—"}` : ""}
      </div>
    </div>
  </div>

  <div class="center">
    <div class="transport">
      <button class="icon-btn" aria-label="prev" onclick={() => api.previous()}>
        <Icon name="prev" size={14} stroke={1.6} />
      </button>
      <button class="icon-btn primary" aria-label="play/pause" onclick={() => store.togglePlay()}>
        <Icon name={store.isPlaying ? "pause" : "play"} size={16} stroke={1.6} />
      </button>
      <button class="icon-btn" aria-label="next" onclick={() => api.next()}>
        <Icon name="next" size={14} stroke={1.6} />
      </button>
    </div>

    <div class="progress-row">
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
  </div>

  <div class="right">
    {#if store.audioFormat}
      <span class="pill accent">{fmtAudio(store.audioFormat)}</span>
    {/if}
    {#if store.current}
      <BackendBadge backend={backendOf(store.current.uri)} />
    {/if}
    {#if store.hasVolume}
      <div class="volume">
        <Icon name={store.volume === 0 ? "volume-mute" : "volume"} size={14} stroke={1.7} />
        <input type="range" min="0" max="100" step="1" value={store.volume} oninput={onVolume} />
      </div>
    {/if}
  </div>
</div>

<style>
  .mini {
    display: grid;
    grid-template-columns: minmax(220px, 1fr) minmax(360px, 2fr) minmax(220px, 1fr);
    align-items: center;
    gap: 24px;
    padding: 12px 18px 14px;
    background: var(--bg-1);
    border-top: 1px solid var(--border-soft);
    min-height: 92px;
  }

  .left {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }
  .meta {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .title {
    font-size: 13px;
    font-weight: 600;
    color: var(--text);
  }
  .sub {
    font-size: 11px;
    color: var(--text-mid);
  }

  .center {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    min-width: 0;
  }
  .transport {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .progress-row {
    display: grid;
    grid-template-columns: 44px 1fr 44px;
    align-items: center;
    gap: 10px;
    width: 100%;
    max-width: 520px;
  }
  .time {
    font-size: 10.5px;
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
    text-align: center;
  }

  .right {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 10px;
    min-width: 0;
  }
  .volume {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--text-muted);
    width: 110px;
  }
  .volume input[type="range"] {
    flex: 1;
  }
</style>
