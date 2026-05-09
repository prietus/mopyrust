<script lang="ts">
  import { store } from "../lib/store.svelte";
  import { backendOf } from "../lib/format";
  import BackendBadge from "./BackendBadge.svelte";
  import Icon from "./Icon.svelte";

  let filter = $state("");
  let visible = $derived(
    filter
      ? store.playlists.filter((p) => p.name.toLowerCase().includes(filter.toLowerCase()))
      : store.playlists,
  );

  function open(uri: string, label: string) {
    store.navTo({ kind: "playlist-detail", uri, label });
  }
</script>

<div class="page">
  <header class="page-header">
    <div class="title-row">
      <h2>Playlists</h2>
      <span class="count">{store.playlists.length}</span>
    </div>
    <input type="search" placeholder="filtrar…" bind:value={filter} />
  </header>

  <div class="body">
    {#if store.playlistsLoading && store.playlists.length === 0}
      <div class="empty">cargando playlists…</div>
    {:else if store.playlists.length === 0}
      <div class="empty">
        <div class="empty-title">sin playlists</div>
        <div class="empty-sub">crea playlists en mopidy o sincroniza desde tidal</div>
      </div>
    {:else}
      <ul class="list">
        {#each visible as p (p.uri)}
          <li>
            <button
              class="row"
              onclick={() => open(p.uri, p.name)}
              onkeydown={(e) => {
                if (e.key === "Enter" || e.key === " ") { e.preventDefault(); open(p.uri, p.name); }
              }}
            >
              <span class="ic"><Icon name="list" size={14} stroke={1.7} /></span>
              <span class="name truncate">{p.name}</span>
              <BackendBadge backend={backendOf(p.uri)} />
              <span class="chev"><Icon name="chevron-right" size={14} stroke={1.6} /></span>
            </button>
          </li>
        {/each}
      </ul>
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
    justify-content: space-between;
    gap: 16px;
    border-bottom: 1px solid var(--border-soft);
    background: var(--bg-0);
    position: sticky;
    top: 0;
    z-index: 1;
  }
  .title-row {
    display: flex;
    align-items: baseline;
    gap: 12px;
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
  input[type="search"] {
    max-width: 280px;
  }

  .body {
    flex: 1;
    overflow: auto;
    padding: 12px 24px 32px;
  }
  ul.list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  ul.list li { display: contents; }

  .row {
    display: grid;
    grid-template-columns: 22px 1fr auto 16px;
    align-items: center;
    gap: 12px;
    padding: 8px 12px;
    border-radius: var(--radius-md);
    color: var(--text);
    background: transparent;
    text-align: left;
    width: 100%;
    transition: background 120ms ease;
  }
  .row:hover {
    background: var(--bg-hover);
  }
  .row:hover .chev { color: var(--accent); }
  .ic {
    color: var(--text-muted);
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .chev {
    color: var(--text-faint);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: color 120ms ease;
  }
  .name {
    font-size: 13px;
    font-weight: 500;
  }

  .empty {
    padding: 60px 32px;
    text-align: center;
    color: var(--text-muted);
  }
  .empty-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-mid);
    margin-bottom: 4px;
  }
  .empty-sub {
    font-size: 11.5px;
    color: var(--text-faint);
  }
</style>
