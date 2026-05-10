<script lang="ts">
  import { store } from "../lib/store.svelte";
  import AlbumCard from "./AlbumCard.svelte";

  let filter = $derived(store.albumsFilter.toLowerCase());
  let visible = $derived(
    filter
      ? store.albums.filter((a) => a.name.toLowerCase().includes(filter))
      : store.albums,
  );
</script>

<div class="page">
  <header class="page-header">
    <div class="title-row">
      <h2>Albums</h2>
      <span class="count">{store.albums.length} albums</span>
    </div>
    <input
      type="search"
      placeholder="filter…"
      bind:value={store.albumsFilter}
    />
  </header>

  <div class="body">
    {#if store.albumsLoading && store.albums.length === 0}
      <div class="empty">discovering albums…</div>
    {:else if store.albums.length === 0}
      <div class="empty">
        <div class="empty-title">no albums</div>
        <div class="empty-sub">check that mopidy-local and/or mopidy-tidal are configured</div>
      </div>
    {:else}
      <div class="grid">
        {#each visible as a (a.uri)}
          <AlbumCard
            uri={a.uri}
            name={a.name}
            backend={a.backend}
            artist={a.artist}
            year={a.year}
          />
        {/each}
      </div>
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
    padding: 20px 24px 32px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 12px;
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
