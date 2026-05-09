<script lang="ts">
  import { store, type Section } from "../lib/store.svelte";
  import Icon from "./Icon.svelte";

  type Props = { onOpenSettings: () => void };
  let { onOpenSettings }: Props = $props();

  const items: { kind: Section["kind"]; label: string; icon: string }[] = [
    { kind: "now-playing", label: "Now Playing", icon: "play" },
    { kind: "search", label: "Search", icon: "search" },
    { kind: "albums", label: "Albums", icon: "grid" },
    { kind: "artists", label: "Artists", icon: "user" },
    { kind: "playlists", label: "Playlists", icon: "list" },
    { kind: "browse", label: "Browse", icon: "folder" },
    { kind: "queue", label: "Queue", icon: "list" },
  ];

  function go(kind: Section["kind"]) {
    store.navTo({ kind } as Section);
  }
</script>

<aside>
  <header>
    <h1>mopyrust</h1>
    <p>mopidy · tidal</p>
  </header>

  <nav>
    {#each items as item (item.kind)}
      {@const active = store.rootKind === item.kind}
      <button
        class="nav-item"
        class:active
        onclick={() => go(item.kind)}
        title={item.label}
      >
        <span class="bar"></span>
        <span class="icon"><Icon name={item.icon} size={15} stroke={1.8} /></span>
        <span class="label">{item.label}</span>
      </button>
    {/each}
  </nav>

  <div class="spacer"></div>

  <footer>
    <div class="conn-row">
      <div class="conn" data-state={store.conn}>
        <span class="dot"></span>
        <span class="state">
          {#if store.conn === "connected"}conectado{:else if store.conn === "connecting"}conectando…{:else if store.conn === "disconnected"}desconectado{:else}error{/if}
        </span>
      </div>
      <button class="settings-btn" aria-label="ajustes" title="ajustes" onclick={onOpenSettings}>
        <Icon name="settings" size={14} stroke={1.7} />
      </button>
    </div>
    {#if store.config}
      <button class="host" onclick={onOpenSettings} title="cambiar host">
        {store.config.host}:{store.config.mpd_port}
      </button>
    {/if}
  </footer>
</aside>

<style>
  aside {
    width: 220px;
    flex-shrink: 0;
    background: var(--bg-1);
    display: flex;
    flex-direction: column;
    border-right: 1px solid var(--border-soft);
    height: 100%;
  }

  header {
    padding: 20px 18px 16px;
  }
  header h1 {
    margin: 0;
    font-size: 16px;
    font-weight: 700;
    letter-spacing: -0.01em;
    color: var(--text);
  }
  header p {
    margin: 2px 0 0;
    font-size: 10.5px;
    color: var(--text-faint);
    letter-spacing: 0.04em;
  }

  nav {
    padding: 0 8px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .nav-item {
    display: grid;
    grid-template-columns: 3px 18px 1fr;
    align-items: center;
    gap: 10px;
    padding: 8px 10px 8px 6px;
    border-radius: var(--radius-md);
    color: var(--text-mid);
    transition: background 120ms ease, color 120ms ease;
    text-align: left;
  }
  .nav-item:hover {
    background: var(--bg-hover);
    color: var(--text);
  }
  .bar {
    display: block;
    width: 3px;
    height: 16px;
    border-radius: 2px;
    background: transparent;
    transition: background 150ms ease;
  }
  .icon {
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
  }
  .label {
    font-size: 13px;
    font-weight: 500;
    letter-spacing: -0.005em;
  }
  .nav-item.active {
    color: var(--text);
    background: var(--accent-soft);
  }
  .nav-item.active .bar {
    background: var(--accent);
  }
  .nav-item.active .icon {
    color: var(--accent);
  }

  .spacer {
    flex: 1;
  }

  footer {
    padding: 14px 18px 18px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .conn {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 11px;
    font-weight: 500;
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--text-faint);
  }
  .conn[data-state="connected"] .dot {
    background: var(--accent);
    box-shadow: 0 0 6px var(--accent-ring);
  }
  .conn[data-state="connected"] .state {
    color: var(--accent);
  }
  .conn[data-state="connecting"] .dot {
    background: var(--text-muted);
  }
  .conn[data-state="connecting"] .state {
    color: var(--text-muted);
  }
  .conn[data-state="error"] .dot {
    background: #d96666;
  }
  .conn[data-state="error"] .state {
    color: #d96666;
  }
  .conn[data-state="disconnected"] .state {
    color: var(--text-faint);
  }
  .conn-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }
  .settings-btn {
    width: 24px;
    height: 24px;
    border-radius: var(--radius-pill);
    color: var(--text-faint);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    transition: background 120ms ease, color 120ms ease;
  }
  .settings-btn:hover {
    background: var(--bg-hover);
    color: var(--text);
  }
  .host {
    font-size: 10.5px;
    color: var(--text-faint);
    letter-spacing: 0.02em;
    background: transparent;
    text-align: left;
    padding: 0;
    transition: color 120ms ease;
  }
  .host:hover {
    color: var(--accent);
  }
</style>
