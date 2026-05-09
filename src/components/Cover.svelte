<script lang="ts">
  import { store } from "../lib/store.svelte";
  import Icon from "./Icon.svelte";

  type Props = {
    uri: string | null | undefined;
    size?: number | string;
    radius?: string;
    elevation?: "none" | "soft" | "strong";
  };

  let { uri, size = 64, radius = "var(--radius-md)", elevation = "none" }: Props = $props();

  $effect(() => {
    if (uri) store.ensureCover(uri);
  });

  let src = $derived(uri ? store.covers[uri] : undefined);

  let style = $derived(
    `--cover-size: ${typeof size === "number" ? size + "px" : size}; ` +
      `--cover-radius: ${radius};`,
  );
</script>

<div class="cover" {style} data-elevation={elevation}>
  {#if src}
    <img {src} alt="" />
  {:else}
    <div class="placeholder">
      <Icon name="music" size={Math.max(20, Math.min(64, typeof size === "number" ? size * 0.32 : 28))} stroke={1.6} />
    </div>
  {/if}
</div>

<style>
  .cover {
    width: var(--cover-size);
    height: var(--cover-size);
    border-radius: var(--cover-radius);
    overflow: hidden;
    background: var(--bg-2);
    flex-shrink: 0;
    position: relative;
    border: 1px solid var(--border-soft);
    transition: box-shadow 200ms ease;
  }
  .cover[data-elevation="soft"] {
    box-shadow: var(--shadow-soft);
  }
  .cover[data-elevation="strong"] {
    box-shadow: var(--shadow-strong);
  }
  .cover img {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .placeholder {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-faint);
    background: linear-gradient(135deg, var(--bg-2), var(--bg-1));
  }
</style>
