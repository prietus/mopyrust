<script lang="ts">
  import { store } from "../lib/store.svelte";
  import AlbumCard from "./AlbumCard.svelte";
  import Icon from "./Icon.svelte";

  type Props = { name: string };
  let { name }: Props = $props();

  let albums = $derived(store.artistAlbums[name] ?? []);
  let loading = $derived(!!store.artistLoading[name]);

  $effect(() => {
    if (name) store.ensureArtistMeta(name);
  });
  let meta = $derived(name ? store.artistMeta[name.toLowerCase()] : undefined);
  let info = $derived(meta?.info ?? null);
  let wiki = $derived(meta?.wiki ?? null);
  let bioOpen = $state(true);
</script>

<div class="page">
  <div class="back-row">
    <button class="back" onclick={() => store.navBack()}>
      <Icon name="chevron-left" size={14} stroke={1.8} />
      <span>artistas</span>
    </button>
  </div>

  <header class="hero">
    <div class="avatar" class:has-photo={!!wiki?.thumbnail_url}>
      {#if wiki?.thumbnail_url}
        <img src={wiki.thumbnail_url} alt={name} loading="lazy" />
      {:else}
        <Icon name="user" size={42} stroke={1.4} />
      {/if}
    </div>
    <div class="info">
      <div class="title">{name}</div>
      <div class="meta">
        {#if info?.type}<span>{info.type}</span><span class="dot">·</span>{/if}
        {#if info?.area}<span>{info.area}</span><span class="dot">·</span>{/if}
        {#if info?.begin_date}<span>{info.begin_date}{info.end_date ? `–${info.end_date}` : ""}</span><span class="dot">·</span>{/if}
        <span>{albums.length} {albums.length === 1 ? "álbum" : "álbumes"}</span>
      </div>
    </div>
  </header>

  {#if wiki?.extract}
    <section class="bio">
      <button class="bio-toggle" onclick={() => (bioOpen = !bioOpen)} aria-expanded={bioOpen}>
        <span class="chev" class:open={bioOpen}>
          <Icon name="chevron-right" size={13} stroke={1.7} />
        </span>
        <span class="bio-label">biografía · {wiki.title}</span>
        <span class="bio-lang">{wiki.language}</span>
      </button>
      {#if bioOpen}
        <p class="bio-extract">{wiki.extract}</p>
        {#if wiki.page_url}
          <a class="bio-link" href={wiki.page_url} target="_blank" rel="noreferrer">ver en Wikipedia</a>
        {/if}
      {/if}
    </section>
  {/if}

  {#if info?.members?.length}
    <section class="members">
      <h3>Miembros</h3>
      <ul>
        {#each info.members as m (m.name + m.period)}
          <li>
            <span class="m-name">{m.name}</span>
            {#if m.role}<span class="m-role">{m.role}</span>{/if}
            {#if m.period}<span class="m-period">{m.period}</span>{/if}
          </li>
        {/each}
      </ul>
    </section>
  {/if}

  <div class="body">
    {#if loading && albums.length === 0}
      <div class="empty">cargando álbumes…</div>
    {:else if albums.length === 0}
      <div class="empty">
        <div class="empty-title">sin álbumes</div>
        <div class="empty-sub">no se encontraron álbumes para este artista</div>
      </div>
    {:else}
      <div class="grid">
        {#each albums as a (a.uri || a.name)}
          <AlbumCard
            uri={a.uri}
            name={a.name}
            backend={a.backend}
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
    overflow: auto;
  }

  .back-row {
    padding: 18px 24px 0;
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
    transition: background 120ms ease, color 120ms ease;
  }
  .back:hover {
    background: var(--bg-hover);
    color: var(--text);
  }

  .hero {
    display: flex;
    align-items: center;
    gap: 20px;
    padding: 16px 28px 24px;
    border-bottom: 1px solid var(--border-soft);
  }
  .avatar {
    width: 96px;
    height: 96px;
    border-radius: 50%;
    background: linear-gradient(135deg, var(--bg-2), var(--bg-1));
    border: 1px solid var(--border-soft);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-faint);
    flex-shrink: 0;
    overflow: hidden;
  }
  .avatar.has-photo {
    background: var(--bg-2);
  }
  .avatar img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .info {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }
  .title {
    font-size: 24px;
    font-weight: 700;
    color: var(--text);
    letter-spacing: -0.01em;
    line-height: 1.15;
  }
  .meta {
    font-size: 11.5px;
    color: var(--text-muted);
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    align-items: center;
  }
  .meta .dot {
    color: var(--text-faint);
  }

  .bio {
    margin: 0 28px 12px;
    padding: 10px 14px;
    background: var(--bg-2);
    border: 1px solid var(--border-soft);
    border-radius: var(--radius-md);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .bio-toggle {
    display: flex;
    align-items: center;
    gap: 8px;
    background: transparent;
    padding: 0;
    text-align: left;
    cursor: pointer;
  }
  .bio-toggle .chev {
    color: var(--text-muted);
    display: inline-flex;
    transition: transform 150ms ease;
  }
  .bio-toggle .chev.open { transform: rotate(90deg); }
  .bio-label {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-mid);
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }
  .bio-lang {
    margin-left: auto;
    font-size: 9.5px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-faint);
  }
  .bio-extract {
    margin: 0;
    font-size: 12.5px;
    color: var(--text-mid);
    line-height: 1.6;
  }
  .bio-link {
    font-size: 11px;
    color: var(--accent);
    text-decoration: none;
    align-self: flex-start;
  }
  .bio-link:hover { text-decoration: underline; }

  .members {
    margin: 0 28px 12px;
    padding: 10px 14px;
    background: var(--bg-2);
    border: 1px solid var(--border-soft);
    border-radius: var(--radius-md);
  }
  .members h3 {
    margin: 0 0 6px 0;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-mid);
  }
  .members ul {
    margin: 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .members li {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    font-size: 12px;
    line-height: 1.5;
  }
  .m-name {
    color: var(--text);
    font-weight: 500;
  }
  .m-role {
    color: var(--text-muted);
  }
  .m-period {
    margin-left: auto;
    color: var(--text-faint);
    font-size: 11px;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  }

  .body {
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
