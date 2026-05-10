<script lang="ts">
  import type { Track } from "../lib/types";
  import { store } from "../lib/store.svelte";
  import { fmtMs, artistsOf, albumOf, backendOf } from "../lib/format";
  import Icon from "./Icon.svelte";
  import BackendBadge from "./BackendBadge.svelte";
  import ContextMenu, { type MenuItem } from "./ContextMenu.svelte";

  type Props = {
    track: Track;
    index?: number | null;
    showAlbum?: boolean;
    showBackend?: boolean;
  };
  let { track, index = null, showAlbum = true, showBackend = true }: Props = $props();

  let menuPos = $state<{ x: number; y: number } | null>(null);

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

  function openMenu(e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    store.ensurePlaylists();
    menuPos = { x: e.clientX, y: e.clientY };
  }

  let menuItems = $derived.by<MenuItem[]>(() => {
    const albumUri = track.album?.uri ?? null;
    const albumName = track.album?.name ?? null;
    const firstArtist = track.artists[0]?.name ?? null;
    const trackUri = track.uri;
    return [
      { kind: "item", label: "Play", icon: "play", action: () => store.playUri(trackUri) },
      {
        kind: "item",
        label: "Play next",
        icon: "next",
        action: () => store.playNextUris([trackUri]),
      },
      {
        kind: "item",
        label: "Add to queue",
        icon: "plus",
        action: () => store.enqueueUri(trackUri),
      },
      {
        kind: "submenu",
        label: "Add to playlist…",
        icon: "list",
        items: store.playlists.length
          ? store.playlists.map((p) => ({
              kind: "item" as const,
              label: p.name,
              action: () => store.addUrisToPlaylist(p.uri, [trackUri]),
            }))
          : [{ kind: "item" as const, label: "(no playlists)", action: () => {}, disabled: true }],
      },
      { kind: "separator" },
      {
        kind: "item",
        label: "Go to album",
        icon: "disc",
        disabled: !albumUri || !albumName,
        action: () => {
          if (albumUri && albumName) {
            store.navTo({ kind: "album-detail", uri: albumUri, label: albumName });
          }
        },
      },
      {
        kind: "item",
        label: "Go to artist",
        icon: "user",
        disabled: !firstArtist,
        action: () => {
          if (firstArtist) store.navTo({ kind: "artist-detail", name: firstArtist });
        },
      },
      { kind: "separator" },
      {
        kind: "item",
        label: "Copy URI",
        action: () => {
          navigator.clipboard.writeText(trackUri).catch((e) => console.error("clipboard", e));
        },
      },
    ];
  });
</script>

<div
  class="row"
  role="button"
  tabindex="0"
  onclick={play}
  onkeydown={rowKey}
  oncontextmenu={openMenu}
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
  <button class="plus" onclick={enqueue} aria-label="add to queue" title="add to queue">
    <Icon name="plus" size={14} stroke={2} />
  </button>
</div>

{#if menuPos}
  <ContextMenu
    x={menuPos.x}
    y={menuPos.y}
    items={menuItems}
    onClose={() => (menuPos = null)}
  />
{/if}

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
