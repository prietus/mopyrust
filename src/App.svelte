<script lang="ts">
  import { onMount } from "svelte";
  import { store } from "./lib/store.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import MiniPlayer from "./components/MiniPlayer.svelte";
  import NowPlaying from "./components/NowPlaying.svelte";
  import Search from "./components/Search.svelte";
  import Albums from "./components/Albums.svelte";
  import AlbumDetail from "./components/AlbumDetail.svelte";
  import Artists from "./components/Artists.svelte";
  import ArtistDetail from "./components/ArtistDetail.svelte";
  import Playlists from "./components/Playlists.svelte";
  import PlaylistDetail from "./components/PlaylistDetail.svelte";
  import Browse from "./components/Browse.svelte";
  import Queue from "./components/Queue.svelte";
  import History from "./components/History.svelte";
  import Settings from "./components/Settings.svelte";

  let settingsOpen = $state(false);

  onMount(() => {
    store.init().then(() => {
      // First run: open Settings automatically if host is still default.
      if (store.config?.host === "127.0.0.1" || !store.config?.host) {
        settingsOpen = true;
      }
    });
  });

  $effect(() => {
    const t = store.config?.theme ?? "midnight";
    document.documentElement.setAttribute("data-theme", t);
  });

  let onNowPlaying = $derived(store.section.kind === "now-playing");
</script>

<main>
  <div class="app">
    <Sidebar onOpenSettings={() => (settingsOpen = true)} />
    <section class="content">
      {#if store.section.kind === "now-playing"}
        <NowPlaying />
      {:else if store.section.kind === "search"}
        <Search />
      {:else if store.section.kind === "albums"}
        <Albums />
      {:else if store.section.kind === "artists"}
        <Artists />
      {:else if store.section.kind === "playlists"}
        <Playlists />
      {:else if store.section.kind === "browse"}
        <Browse />
      {:else if store.section.kind === "queue"}
        <Queue />
      {:else if store.section.kind === "history"}
        <History />
      {:else if store.section.kind === "album-detail"}
        <AlbumDetail uri={store.section.uri} label={store.section.label} />
      {:else if store.section.kind === "artist-detail"}
        <ArtistDetail name={store.section.name} />
      {:else if store.section.kind === "playlist-detail"}
        <PlaylistDetail uri={store.section.uri} label={store.section.label} />
      {/if}
    </section>
  </div>
  {#if !onNowPlaying}
    <MiniPlayer />
  {/if}
  {#if settingsOpen}
    <Settings onClose={() => (settingsOpen = false)} />
  {/if}
</main>

<style>
  main {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
  }
  .app {
    flex: 1;
    display: flex;
    min-height: 0;
  }
  .content {
    flex: 1;
    min-width: 0;
    background: var(--bg-0);
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
</style>
