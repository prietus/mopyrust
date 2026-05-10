<script lang="ts">
  import { store } from "../lib/store.svelte";
  import { api } from "../lib/api";
  import { fmtMs, artistsOf, albumOf, backendOf } from "../lib/format";
  import BackendBadge from "./BackendBadge.svelte";
  import Icon from "./Icon.svelte";

  let dragIndex = $state<number | null>(null);
  let dropIndex = $state<number | null>(null);

  function onDragStart(e: DragEvent, idx: number) {
    dragIndex = idx;
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = "move";
      // Required by Firefox/some browsers to actually start the drag.
      e.dataTransfer.setData("text/plain", String(idx));
    }
  }

  function onDragOver(e: DragEvent, idx: number) {
    if (dragIndex === null) return;
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
    dropIndex = idx;
  }

  function onDragLeave(idx: number) {
    if (dropIndex === idx) dropIndex = null;
  }

  async function onDrop(e: DragEvent, idx: number) {
    e.preventDefault();
    const from = dragIndex;
    const to = idx;
    dragIndex = null;
    dropIndex = null;
    if (from === null || from === to) return;
    // Optimistic local reorder so the UI doesn't jump while the RPC roundtrips.
    const items = store.queue.slice();
    const [moved] = items.splice(from, 1);
    const insertAt = to > from ? to - 1 : to;
    items.splice(insertAt, 0, moved);
    store.queue = items;
    try {
      await api.moveTrack(from, to);
    } catch (e) {
      console.error("move", e);
      // Re-sync from server on failure.
      store.refreshQueue();
    }
  }

  function onDragEnd() {
    dragIndex = null;
    dropIndex = null;
  }
</script>

<div class="page">
  <header class="page-header">
    <h2>Queue</h2>
    <span class="count">{store.queue.length} {store.queue.length === 1 ? "track" : "tracks"}</span>
  </header>

  <div class="body">
    {#if store.queue.length === 0}
      <div class="empty">queue is empty</div>
    {:else}
      {#each store.queue as tl, i (tl.tlid)}
        {@const isCurrent = store.currentTlid === tl.tlid}
        {@const isDragging = dragIndex === i}
        {@const isDropTarget = dropIndex === i && dragIndex !== null && dragIndex !== i}
        <div
          class="row"
          class:current={isCurrent}
          class:dragging={isDragging}
          class:drop-target={isDropTarget}
          class:drop-above={isDropTarget && dragIndex !== null && i < dragIndex}
          class:drop-below={isDropTarget && dragIndex !== null && i > dragIndex}
          role="button"
          tabindex="0"
          draggable="true"
          ondragstart={(e) => onDragStart(e, i)}
          ondragover={(e) => onDragOver(e, i)}
          ondragleave={() => onDragLeave(i)}
          ondrop={(e) => onDrop(e, i)}
          ondragend={onDragEnd}
          onclick={() => api.playTlid(tl.tlid)}
          onkeydown={(e) => {
            if (e.key === "Enter" || e.key === " ") {
              e.preventDefault();
              api.playTlid(tl.tlid);
            }
          }}
        >
          <span class="grip" aria-label="arrastrar">
            <span class="dots"></span>
            <span class="dots"></span>
          </span>
          <span class="dot">{isCurrent ? "●" : ""}</span>
          <div class="meta truncate">
            <div class="title truncate">{tl.track.name || tl.track.uri}</div>
            <div class="sub truncate">{artistsOf(tl.track) || "—"} · {albumOf(tl.track) || "—"}</div>
          </div>
          <BackendBadge backend={backendOf(tl.track.uri)} />
          <span class="dur">{fmtMs(tl.track.length)}</span>
          <button
            class="rem"
            aria-label="quitar"
            onclick={(e) => {
              e.stopPropagation();
              api.removeTlid(tl.tlid);
            }}
          >
            <Icon name="x" size={13} stroke={2} />
          </button>
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .page {
    height: 100%;
    display: flex;
    flex-direction: column;
  }
  .page-header {
    padding: 24px 28px 16px;
    display: flex;
    align-items: baseline;
    gap: 12px;
    border-bottom: 1px solid var(--border-soft);
    background: var(--bg-0);
    position: sticky;
    top: 0;
    z-index: 1;
  }
  h2 {
    margin: 0;
    font-size: 17px;
    font-weight: 700;
    letter-spacing: -0.01em;
  }
  .count {
    font-size: 11px;
    color: var(--text-faint);
  }

  .body {
    flex: 1;
    overflow: auto;
    padding: 12px 24px 32px;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .row {
    display: grid;
    grid-template-columns: 18px 14px 1fr auto auto auto;
    gap: 12px;
    align-items: center;
    padding: 8px 12px;
    border-radius: var(--radius-md);
    background: transparent;
    text-align: left;
    color: var(--text);
    transition: background 120ms ease, color 120ms ease, transform 120ms ease,
      box-shadow 120ms ease;
    position: relative;
    cursor: grab;
  }
  .row:hover {
    background: var(--bg-hover);
  }
  .row:active { cursor: grabbing; }
  .row.current .title {
    color: var(--accent);
  }
  .row.dragging {
    opacity: 0.4;
  }
  .row.drop-target {
    background: var(--accent-soft);
  }
  .row.drop-above::before,
  .row.drop-below::after {
    content: "";
    position: absolute;
    left: 8px;
    right: 8px;
    height: 2px;
    background: var(--accent);
    border-radius: 1px;
  }
  .row.drop-above::before { top: -1px; }
  .row.drop-below::after { bottom: -1px; }

  .grip {
    display: inline-flex;
    flex-direction: column;
    gap: 3px;
    align-items: center;
    justify-content: center;
    color: var(--text-faint);
    opacity: 0;
    transition: opacity 120ms ease;
  }
  .row:hover .grip { opacity: 0.7; }
  .grip .dots {
    width: 3px;
    height: 3px;
    border-radius: 50%;
    background: currentColor;
    box-shadow: 5px 0 0 0 currentColor;
  }

  .dot {
    color: var(--accent);
    text-align: center;
    font-size: 10px;
  }
  .meta {
    display: flex;
    flex-direction: column;
    gap: 1px;
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
  .rem {
    width: 26px;
    height: 26px;
    border-radius: var(--radius-pill);
    color: var(--text-muted);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    transition: background 120ms ease, color 120ms ease;
  }
  .rem:hover {
    background: var(--bg-elev);
    color: #d96666;
  }
  .empty {
    padding: 60px 32px;
    text-align: center;
    color: var(--text-faint);
  }
</style>
