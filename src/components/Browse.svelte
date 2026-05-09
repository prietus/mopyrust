<script lang="ts">
  import { store } from "../lib/store.svelte";
  import { backendOf } from "../lib/format";
  import BackendBadge from "./BackendBadge.svelte";
  import Icon from "./Icon.svelte";
  import type { LibRef } from "../lib/types";

  function iconFor(kind: string): string {
    if (kind === "album") return "disc";
    if (kind === "artist") return "user";
    if (kind === "playlist") return "list";
    if (kind === "track") return "music";
    return "folder";
  }

  function activate(r: LibRef) {
    const k = (r.kind || "").toLowerCase();
    if (k === "track") {
      store.playUri(r.uri);
    } else if (k === "album") {
      store.navTo({ kind: "album-detail", uri: r.uri, label: r.name });
    } else {
      store.browseInto(r.name, r.uri);
    }
  }

  function enqueue(r: LibRef, e: MouseEvent) {
    e.stopPropagation();
    if ((r.kind || "").toLowerCase() === "track") {
      store.enqueueUri(r.uri);
    }
  }
</script>

<div class="page">
  <header class="page-header">
    <h2>Browse</h2>
    <div class="crumbs">
      {#if store.browseStack.length > 1}
        <button class="back" onclick={() => store.browseUp()}>
          <Icon name="chevron-left" size={14} stroke={1.8} />
          subir
        </button>
      {/if}
      <div class="path">
        {#each store.browseStack as c, i (i)}
          <span class:current={i === store.browseStack.length - 1}>{c.label}</span>
          {#if i + 1 < store.browseStack.length}
            <span class="sep">›</span>
          {/if}
        {/each}
      </div>
    </div>
  </header>

  <div class="body">
    {#if store.browseLoading}
      <div class="empty">cargando…</div>
    {:else if store.browseItems.length === 0}
      <div class="empty">vacío</div>
    {:else}
      {#each store.browseItems as r (r.uri)}
        {@const k = (r.kind || "").toLowerCase()}
        <div
          class="row"
          role="button"
          tabindex="0"
          onclick={() => activate(r)}
          onkeydown={(e) => {
            if (e.key === "Enter" || e.key === " ") { e.preventDefault(); activate(r); }
          }}
        >
          <span class="ic"><Icon name={iconFor(k)} size={14} stroke={1.7} /></span>
          <div class="meta truncate">
            <div class="title truncate">{r.name}</div>
            <div class="sub">{k}</div>
          </div>
          <BackendBadge backend={backendOf(r.uri)} />
          {#if k === "track"}
            <button class="plus" onclick={(e) => enqueue(r, e)} aria-label="añadir a cola">
              <Icon name="plus" size={13} stroke={2} />
            </button>
          {/if}
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
    align-items: center;
    gap: 20px;
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
  .crumbs {
    display: flex;
    align-items: center;
    gap: 12px;
    flex: 1;
    min-width: 0;
  }
  .back {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 4px 10px;
    border-radius: var(--radius-md);
    color: var(--text-mid);
    font-size: 11.5px;
    font-weight: 500;
    transition: background 120ms ease;
  }
  .back:hover {
    background: var(--bg-hover);
    color: var(--text);
  }
  .path {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: var(--text-mid);
    min-width: 0;
    overflow: hidden;
  }
  .path .current {
    color: var(--text);
    font-weight: 600;
  }
  .path .sep {
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
    grid-template-columns: 22px 1fr auto auto;
    gap: 12px;
    align-items: center;
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
  .ic {
    color: var(--text-muted);
    display: flex;
    align-items: center;
    justify-content: center;
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
    font-size: 10.5px;
    color: var(--text-faint);
  }
  .plus {
    width: 26px;
    height: 26px;
    border-radius: var(--radius-pill);
    color: var(--text-muted);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    transition: background 120ms ease, color 120ms ease;
  }
  .plus:hover {
    background: var(--bg-elev);
    color: var(--text);
  }
  .empty {
    padding: 60px 32px;
    text-align: center;
    color: var(--text-faint);
  }
</style>
