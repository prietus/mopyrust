<script lang="ts">
  import { store } from "../lib/store.svelte";
  import Cover from "./Cover.svelte";
  import BackendBadge from "./BackendBadge.svelte";

  type Props = {
    uri: string;
    name: string;
    backend: string;
    artist?: string | null;
    year?: string | null;
    sub?: string | null;
  };
  let { uri, name, backend, artist = null, year = null, sub = null }: Props = $props();

  function open() {
    store.navTo({ kind: "album-detail", uri, label: name });
  }

  let subline = $derived.by(() => {
    if (sub) return sub;
    const parts: string[] = [];
    if (artist) parts.push(artist);
    if (year) parts.push(year);
    return parts.join(" · ");
  });
</script>

<button class="card" onclick={open}>
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
