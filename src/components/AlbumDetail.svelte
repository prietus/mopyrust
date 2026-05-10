<script lang="ts">
  import { store } from "../lib/store.svelte";
  import { fmtMs, backendOf, artistsOf } from "../lib/format";
  import Cover from "./Cover.svelte";
  import BackendBadge from "./BackendBadge.svelte";
  import TrackRow from "./TrackRow.svelte";
  import Icon from "./Icon.svelte";

  type Props = { uri: string; label: string };
  let { uri, label }: Props = $props();

  let tracks = $derived(store.albumTracks[uri] ?? []);
  let loading = $derived(!!store.albumLoading[uri]);

  let title = $derived(tracks[0]?.album?.name || label);
  let artist = $derived(tracks[0] ? artistsOf(tracks[0]) : "");
  let year = $derived(tracks[0]?.album?.date ?? tracks[0]?.date);
  let runtime = $derived(tracks.reduce((a, t) => a + (t.length ?? 0), 0));

  // Trigger metadata fetch once we know artist+title.
  let metaKey = $derived(
    artist && title ? `${artist.toLowerCase()}::${title.toLowerCase()}` : ""
  );
  $effect(() => {
    if (artist && title) store.ensureAlbumMeta(artist, title);
  });
  let meta = $derived(metaKey ? store.albumMeta[metaKey] : undefined);
  let release = $derived(meta?.release ?? null);
  let wiki = $derived(meta?.wiki ?? null);

  let producers = $derived(creditNames(release?.credits ?? [], ["producer", "executive producer"]));
  let engineers = $derived(creditNames(release?.credits ?? [], ["engineer", "recording", "mix", "balance"]));
  let mastering = $derived(creditNames(release?.credits ?? [], ["mastering"]));

  function creditNames(
    credits: { name: string; role: string }[],
    keywords: string[],
  ): string[] {
    const seen = new Set<string>();
    const out: string[] = [];
    for (const c of credits) {
      const role = c.role.toLowerCase();
      if (keywords.some((k) => role.includes(k)) && !seen.has(c.name)) {
        seen.add(c.name);
        out.push(c.name);
      }
    }
    return out;
  }

  let bioOpen = $state(false);

  function playAll() {
    if (tracks.length) store.playUris(tracks.map((t) => t.uri));
  }
  function queueAll() {
    if (tracks.length) store.enqueueUris(tracks.map((t) => t.uri));
  }
</script>

<div class="page">
  <button class="back" onclick={() => store.navBack()}>
    <Icon name="chevron-left" size={14} stroke={1.8} />
    <span>back</span>
  </button>

  <div class="hero">
    <Cover {uri} size={260} radius="var(--radius-lg)" elevation="strong" />
    <div class="info">
      <div class="title">{title}</div>
      {#if artist}
        <div class="artist">{artist}</div>
      {/if}
      <div class="chips">
        <BackendBadge backend={backendOf(uri)} size="md" />
        {#if year}
          <span class="pill">{String(year).split("-")[0]}</span>
        {/if}
        {#if tracks.length}
          <span class="pill">{tracks.length} {tracks.length === 1 ? "track" : "tracks"}</span>
        {/if}
        {#if runtime > 0}
          <span class="pill">{fmtMs(runtime)}</span>
        {/if}
        {#if release?.country}
          <span class="pill">{release.country}</span>
        {/if}
      </div>
      {#if release && (release.label || release.catalog_number)}
        <div class="release-line">
          {#if release.label}<span>{release.label}</span>{/if}
          {#if release.catalog_number}<span class="dot">·</span><span class="mono">{release.catalog_number}</span>{/if}
        </div>
      {/if}
      {#if release?.genres?.length}
        <div class="genres">
          {#each release.genres.slice(0, 6) as g (g)}
            <span class="genre">{g}</span>
          {/each}
        </div>
      {/if}
      <div class="actions">
        <button class="btn primary" onclick={playAll} disabled={!tracks.length}>
          <Icon name="play" size={13} />
          play all
        </button>
        <button class="btn" onclick={queueAll} disabled={!tracks.length}>
          <Icon name="plus" size={13} stroke={2} />
          add to queue
        </button>
      </div>
    </div>
  </div>

  {#if wiki?.extract}
    <section class="wiki">
      <button class="wiki-toggle" onclick={() => (bioOpen = !bioOpen)} aria-expanded={bioOpen}>
        <span class="chev" class:open={bioOpen}>
          <Icon name="chevron-right" size={13} stroke={1.7} />
        </span>
        <span class="wiki-title">review · {wiki.title}</span>
        <span class="wiki-lang">{wiki.language}</span>
      </button>
      {#if bioOpen}
        <p class="wiki-extract">{wiki.extract}</p>
        {#if wiki.page_url}
          <a class="wiki-link" href={wiki.page_url} target="_blank" rel="noreferrer">
            view on Wikipedia
          </a>
        {/if}
      {/if}
    </section>
  {/if}

  {#if producers.length || engineers.length || mastering.length}
    <section class="credits">
      <h3>Credits</h3>
      {#if producers.length}
        <div class="credit-row">
          <span class="credit-label">production</span>
          <span class="credit-names">{producers.join(", ")}</span>
        </div>
      {/if}
      {#if engineers.length}
        <div class="credit-row">
          <span class="credit-label">engineering</span>
          <span class="credit-names">{engineers.join(", ")}</span>
        </div>
      {/if}
      {#if mastering.length}
        <div class="credit-row">
          <span class="credit-label">mastering</span>
          <span class="credit-names">{mastering.join(", ")}</span>
        </div>
      {/if}
    </section>
  {/if}

  <div class="tracklist">
    {#if loading && tracks.length === 0}
      <div class="empty">loading tracks…</div>
    {:else if tracks.length === 0}
      <div class="empty">empty album</div>
    {:else}
      {#each tracks as t, i (t.uri)}
        <TrackRow track={t} index={i} showAlbum={false} showBackend={false} />
      {/each}
    {/if}
  </div>
</div>

<style>
  .page {
    height: 100%;
    overflow: auto;
    padding: 20px 28px 40px;
  }
  .back {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 6px 10px;
    border-radius: var(--radius-md);
    color: var(--text-mid);
    font-size: 12px;
    font-weight: 500;
    background: transparent;
    margin-bottom: 16px;
    transition: background 120ms ease, color 120ms ease;
  }
  .back:hover {
    background: var(--bg-hover);
    color: var(--text);
  }

  .hero {
    display: flex;
    gap: 24px;
    align-items: flex-end;
    padding-bottom: 24px;
    border-bottom: 1px solid var(--border-soft);
    margin-bottom: 16px;
  }
  .info {
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-width: 0;
    flex: 1;
  }
  .title {
    font-size: 22px;
    font-weight: 700;
    color: var(--text);
    letter-spacing: -0.01em;
    line-height: 1.2;
  }
  .artist {
    font-size: 13px;
    color: var(--text-mid);
    font-weight: 500;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 4px;
  }
  .actions {
    display: flex;
    gap: 8px;
    margin-top: 8px;
  }

  .release-line {
    font-size: 11.5px;
    color: var(--text-muted);
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 2px;
  }
  .release-line .dot { color: var(--text-faint); }
  .release-line .mono {
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 11px;
    color: var(--text-mid);
  }
  .genres {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-top: 4px;
  }
  .genre {
    background: var(--accent-soft);
    color: var(--accent);
    border: 1px solid var(--accent-ring);
    border-radius: var(--radius-pill);
    padding: 2px 9px;
    font-size: 10.5px;
    font-weight: 500;
    letter-spacing: 0.01em;
  }

  .wiki {
    margin-bottom: 14px;
    padding: 10px 14px;
    background: var(--bg-2);
    border: 1px solid var(--border-soft);
    border-radius: var(--radius-md);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .wiki-toggle {
    display: flex;
    align-items: center;
    gap: 8px;
    background: transparent;
    padding: 0;
    text-align: left;
    cursor: pointer;
  }
  .wiki-toggle .chev {
    color: var(--text-muted);
    display: inline-flex;
    transition: transform 150ms ease;
  }
  .wiki-toggle .chev.open { transform: rotate(90deg); }
  .wiki-title {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-mid);
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }
  .wiki-lang {
    margin-left: auto;
    font-size: 9.5px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-faint);
  }
  .wiki-extract {
    margin: 0;
    font-size: 12.5px;
    color: var(--text-mid);
    line-height: 1.6;
  }
  .wiki-link {
    font-size: 11px;
    color: var(--accent);
    text-decoration: none;
    align-self: flex-start;
  }
  .wiki-link:hover { text-decoration: underline; }

  .credits {
    margin-bottom: 14px;
    padding: 10px 14px;
    background: var(--bg-2);
    border: 1px solid var(--border-soft);
    border-radius: var(--radius-md);
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .credits h3 {
    margin: 0 0 4px 0;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-mid);
  }
  .credit-row {
    display: flex;
    gap: 12px;
    font-size: 12px;
    line-height: 1.5;
  }
  .credit-label {
    flex-shrink: 0;
    width: 90px;
    color: var(--text-faint);
    font-size: 10.5px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    padding-top: 2px;
  }
  .credit-names {
    color: var(--text);
    flex: 1;
  }

  .tracklist {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .empty {
    padding: 40px;
    text-align: center;
    color: var(--text-faint);
    font-size: 12px;
  }
</style>
