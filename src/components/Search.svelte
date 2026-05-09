<script lang="ts">
  import { store } from "../lib/store.svelte";
  import { backendOf } from "../lib/format";
  import AlbumCard from "./AlbumCard.svelte";
  import TrackRow from "./TrackRow.svelte";
  import Icon from "./Icon.svelte";

  function submit(e: Event) {
    e.preventDefault();
    store.runSearch(store.searchInput);
  }

  type AlbumEntry = { uri: string; name: string; backend: string; artist: string; year: string | null };
  let allAlbums = $derived.by(() => {
    const out: AlbumEntry[] = [];
    for (const r of store.searchResults) {
      for (const a of r.albums) {
        if (!a.uri) continue;
        // Prefer the item's own URI for backend detection (SearchResult.uri is
        // the search query like "tidal:search?...", not always the item's host).
        const bk = backendOf(a.uri || r.uri || "");
        out.push({
          uri: a.uri,
          name: a.name,
          backend: bk,
          artist: a.artists.map((x) => x.name).join(", "),
          year: a.date ? a.date.split("-")[0] : null,
        });
      }
    }
    return out;
  });

  let allTracks = $derived.by(() => {
    return store.searchResults.flatMap((r) =>
      r.tracks.map((t) => ({ track: t, backend: backendOf(t.uri || r.uri || "") })),
    );
  });
</script>

<div class="page">
  <header class="page-header">
    <h2>Search</h2>
    <form onsubmit={submit} class="search-form">
      <span class="ic"><Icon name="search" size={14} stroke={1.8} /></span>
      <input
        type="search"
        placeholder="buscar en tidal + librería local…"
        bind:value={store.searchInput}
      />
    </form>
  </header>

  <div class="body">
    {#if !store.searchQuery}
      <div class="empty">
        <div class="empty-title">encuentra música en tidal y en tu librería local</div>
        <div class="empty-sub">escribe arriba y pulsa enter</div>
      </div>
    {:else if store.searchLoading}
      <div class="empty">buscando «{store.searchQuery}»…</div>
    {:else}
      {#if allAlbums.length}
        <section>
          <h3>álbumes</h3>
          <div class="grid">
            {#each allAlbums as a (a.uri)}
              <AlbumCard
                uri={a.uri}
                name={a.name}
                backend={a.backend}
                artist={a.artist || null}
                year={a.year}
              />
            {/each}
          </div>
        </section>
      {/if}
      {#if allTracks.length}
        <section>
          <h3>pistas</h3>
          <div class="rows">
            {#each allTracks as r, i (i + r.track.uri)}
              <TrackRow track={r.track} />
            {/each}
          </div>
        </section>
      {/if}
      {#if !allAlbums.length && !allTracks.length}
        <div class="empty">sin resultados para «{store.searchQuery}»</div>
      {/if}
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
  h2 {
    margin: 0;
    font-size: 17px;
    font-weight: 700;
    letter-spacing: -0.01em;
  }
  .search-form {
    position: relative;
    flex: 1;
    max-width: 380px;
  }
  .search-form .ic {
    position: absolute;
    left: 10px;
    top: 50%;
    transform: translateY(-50%);
    color: var(--text-muted);
    pointer-events: none;
  }
  .search-form input {
    padding-left: 32px;
  }

  .body {
    flex: 1;
    overflow: auto;
    padding: 20px 24px 32px;
    display: flex;
    flex-direction: column;
    gap: 28px;
  }

  section {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  h3 {
    margin: 0;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text-faint);
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 12px;
  }
  .rows {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .empty {
    padding: 60px 32px;
    text-align: center;
    color: var(--text-muted);
  }
  .empty-title {
    font-size: 13.5px;
    font-weight: 600;
    color: var(--text-mid);
    margin-bottom: 4px;
  }
  .empty-sub {
    font-size: 11.5px;
    color: var(--text-faint);
  }
</style>
