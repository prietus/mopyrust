<script lang="ts">
  import { store } from "../lib/store.svelte";
  import { api } from "../lib/api";
  import Cover from "./Cover.svelte";
  import BackendBadge from "./BackendBadge.svelte";
  import ContextMenu, { type MenuItem } from "./ContextMenu.svelte";

  type Props = {
    uri: string;
    name: string;
    backend: string;
    artist?: string | null;
    year?: string | null;
    sub?: string | null;
  };
  let { uri, name, backend, artist = null, year = null, sub = null }: Props = $props();

  let menuPos = $state<{ x: number; y: number } | null>(null);

  function open() {
    store.navTo({ kind: "album-detail", uri, label: name });
  }

  function openMenu(e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    store.ensurePlaylists();
    menuPos = { x: e.clientX, y: e.clientY };
  }

  /** Mopidy core can't enqueue an album URI directly — has to be the track URIs. */
  async function trackUris(): Promise<string[]> {
    const cached = store.albumTracks[uri];
    if (cached?.length) return cached.map((t) => t.uri);
    const m = await api.lookup([uri]);
    return (m[uri] ?? []).map((t) => t.uri);
  }

  let subline = $derived.by(() => {
    if (sub) return sub;
    const parts: string[] = [];
    if (artist) parts.push(artist);
    if (year) parts.push(year);
    return parts.join(" · ");
  });

  let menuItems = $derived.by<MenuItem[]>(() => [
    {
      kind: "item",
      label: "Play",
      icon: "play",
      action: async () => store.playUris(await trackUris()),
    },
    {
      kind: "item",
      label: "Play next",
      icon: "next",
      action: async () => store.playNextUris(await trackUris()),
    },
    {
      kind: "item",
      label: "Add to queue",
      icon: "plus",
      action: async () => store.enqueueUris(await trackUris()),
    },
    {
      kind: "submenu",
      label: "Add to playlist…",
      icon: "list",
      items: store.playlists.length
        ? store.playlists.map((p) => ({
            kind: "item" as const,
            label: p.name,
            action: async () => {
              const uris = await trackUris();
              if (uris.length) await store.addUrisToPlaylist(p.uri, uris);
            },
          }))
        : [{ kind: "item" as const, label: "(no playlists)", action: () => {}, disabled: true }],
    },
    { kind: "separator" },
    {
      kind: "item",
      label: "Go to artist",
      icon: "user",
      disabled: !artist,
      action: () => {
        if (artist) store.navTo({ kind: "artist-detail", name: artist });
      },
    },
    { kind: "separator" },
    {
      kind: "item",
      label: "Copy URI",
      action: () => {
        navigator.clipboard.writeText(uri).catch((e) => console.error("clipboard", e));
      },
    },
  ]);
</script>

<button class="card" onclick={open} oncontextmenu={openMenu}>
  <div class="cover-wrap">
    <Cover {uri} size="100%" radius="var(--radius-md)" elevation="soft" />
    <div class="badge">
      <BackendBadge {backend} />
    </div>
  </div>
  <div class="title truncate">{name}</div>
  {#if subline}
    <div class="sub truncate">{subline}</div>
  {/if}
</button>

{#if menuPos}
  <ContextMenu
    x={menuPos.x}
    y={menuPos.y}
    items={menuItems}
    onClose={() => (menuPos = null)}
  />
{/if}

<style>
  .card {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 8px;
    border-radius: var(--radius-md);
    background: transparent;
    text-align: left;
    transition: background 150ms ease, transform 150ms ease;
  }
  .card:hover {
    background: var(--bg-hover);
  }
  .card:hover .cover-wrap :global(.cover) {
    box-shadow: var(--shadow-strong);
    transform: translateY(-2px);
  }
  .cover-wrap {
    position: relative;
    width: 100%;
    aspect-ratio: 1 / 1;
  }
  .cover-wrap :global(.cover) {
    width: 100% !important;
    height: 100% !important;
    transition: box-shadow 200ms ease, transform 200ms ease;
  }
  .badge {
    position: absolute;
    top: 8px;
    right: 8px;
  }
  .title {
    font-size: 12.5px;
    font-weight: 600;
    color: var(--text);
    letter-spacing: -0.005em;
  }
  .sub {
    font-size: 10.5px;
    color: var(--text-muted);
  }
</style>
