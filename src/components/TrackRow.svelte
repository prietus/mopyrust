<script lang="ts">
  import type { Track } from "../lib/types";
  import { store } from "../lib/store.svelte";
  import { fmtMs, artistsOf, albumOf, backendOf } from "../lib/format";
  import Icon from "./Icon.svelte";
  import BackendBadge from "./BackendBadge.svelte";

  type Props = {
    track: Track;
    index?: number | null;
    showAlbum?: boolean;
    showBackend?: boolean;
  };
  let { track, index = null, showAlbum = true, showBackend = true }: Props = $props();

  function play() {
    store.playUri(track.uri);
  }
  function enqueue(e: MouseEvent) {
    e.stopPropagation();
    store.enqueueUri(track.uri);
  }
  function rowKey(e: KeyboardEvent) {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      play();
    }
  }
</script>

<div
  class="row"
  role="button"
  tabindex="0"
  onclick={play}
  onkeydown={rowKey}
  style={index === null ? '--num-col: 0' : '--num-col: 28px'}
>
  {#if index !== null}
    <span class="num">{index + 1}</span>
  {/if}
  <div class="meta truncate">
    <div class="title truncate">{track.name || track.uri}</div>
    <div class="sub truncate">
      {artistsOf(track) || "—"}{#if showAlbum && albumOf(track)} · {albumOf(track)}{/if}
    </div>
  </div>
  {#if showBackend}
    <BackendBadge backend={backendOf(track.uri)} />
  {/if}
  <span class="dur">{fmtMs(track.length)}</span>
  <button class="plus" onclick={enqueue} aria-label="añadir a cola" title="añadir a cola">
    <Icon name="plus" size={14} stroke={2} />
  </button>
</div>

<style>
  .row {
    display: grid;
    grid-template-columns: var(--num-col, 0px) 1fr auto auto auto;
    gap: 12px;
    align-items: center;
    width: 100%;
    padding: 8px 12px;
    border-radius: var(--radius-md);
    background: transparent;
    text-align: left;
    color: var(--text);
    transition: background 120ms ease;
  }
  .row:hover {
    background: var(--bg-hover);
  }
  .num {
    font-size: 11px;
    color: var(--text-faint);
    font-variant-numeric: tabular-nums;
    text-align: right;
    min-width: 24px;
  }
  .meta {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .title {
    font-size: 13px;
    font-weight: 500;
  }
  .sub {
    font-size: 11px;
    color: var(--text-muted);
  }
  .dur {
    font-size: 11px;
    color: var(--text-faint);
    font-variant-numeric: tabular-nums;
    width: 44px;
    text-align: right;
  }
  .plus {
    width: 26px;
    height: 26px;
    border-radius: var(--radius-pill);
    color: var(--text-muted);
    background: transparent;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    transition: background 120ms ease, color 120ms ease;
  }
  .plus:hover {
    background: var(--bg-elev);
    color: var(--text);
  }
</style>
