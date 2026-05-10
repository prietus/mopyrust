<script lang="ts">
  import { store } from "../lib/store.svelte";
  import { api } from "../lib/api";
  import Icon from "./Icon.svelte";

  type Props = { onClose: () => void };
  let { onClose }: Props = $props();

  let host = $state(store.config?.host ?? "");
  let mpdPort = $state(store.config?.mpd_port ?? 6600);
  let httpPort = $state(store.config?.http_port ?? 6680);
  let theme = $state(store.config?.theme ?? "midnight");
  let lastfm = $state(store.config?.lastfm_api_key ?? "");
  let fanart = $state(store.config?.fanart_api_key ?? "");
  let discogs = $state(store.config?.discogs_token ?? "");
  let saving = $state(false);
  let error = $state<string | null>(null);
  let saved = $state(false);
  let showKeys = $state(false);

  const themes = ["midnight", "soft-dark", "daylight", "solar"];

  async function save(restart: boolean) {
    error = null;
    saving = true;
    try {
      await api.saveConfig({
        host: host.trim(),
        mpd_port: mpdPort,
        http_port: httpPort,
        theme,
        lastfm_api_key: lastfm.trim() || null,
        fanart_api_key: fanart.trim() || null,
        discogs_token: discogs.trim() || null,
      });
      store.config = await api.getConfig();
      saved = true;
      if (restart) {
        await api.restartApp();
        return;
      }
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

  function backdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) onClose();
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
  }
</script>

<svelte:window onkeydown={onKey} />

<div
  class="backdrop"
  role="dialog"
  aria-modal="true"
  aria-labelledby="settings-title"
  onclick={backdropClick}
  onkeydown={onKey}
  tabindex="-1"
>
  <div class="modal" role="document">
    <header>
      <h2 id="settings-title">Settings</h2>
      <button class="close" aria-label="close" onclick={onClose}>
        <Icon name="x" size={14} stroke={2} />
      </button>
    </header>

    <div class="form">
      <section>
        <h3>Server</h3>
        <p class="hint">
          Mopidy server host (e.g. <code>192.168.1.44</code> or <code>wasp.local</code>).
        </p>

        <label>
          <span>Host</span>
          <input type="text" bind:value={host} placeholder="192.168.1.10" autocomplete="off" />
        </label>

        <div class="ports">
          <label>
            <span>MPD port</span>
            <input type="number" min="1" max="65535" bind:value={mpdPort} />
          </label>
          <label>
            <span>HTTP port</span>
            <input type="number" min="1" max="65535" bind:value={httpPort} />
          </label>
        </div>

        <label>
          <span>Theme</span>
          <select bind:value={theme}>
            {#each themes as t (t)}
              <option value={t}>{t}</option>
            {/each}
          </select>
        </label>
      </section>

      <section>
        <button
          class="section-toggle"
          onclick={() => (showKeys = !showKeys)}
          aria-expanded={showKeys}
        >
          <span class="chev" class:open={showKeys}>
            <Icon name="chevron-right" size={14} stroke={1.7} />
          </span>
          <h3>Metadata services</h3>
          <span class="optional">optional</span>
        </button>

        {#if showKeys}
          <p class="hint">
            Enables biographies, genres, similar artists and high-resolution photos.
            Without these keys you still get MusicBrainz and Wikipedia.
          </p>

          <label>
            <span>Last.fm API key</span>
            <input type="password" bind:value={lastfm} placeholder="32 hex characters" autocomplete="off" />
            <span class="field-hint">
              <a href="https://www.last.fm/api/account/create" target="_blank" rel="noreferrer">create key</a>
              · artist bio, tags, similar
            </span>
          </label>

          <label>
            <span>Fanart.tv API key</span>
            <input type="password" bind:value={fanart} placeholder="" autocomplete="off" />
            <span class="field-hint">
              <a href="https://fanart.tv/get-an-api-key/" target="_blank" rel="noreferrer">get key</a>
              · HD artist images
            </span>
          </label>

          <label>
            <span>Discogs personal token</span>
            <input type="password" bind:value={discogs} placeholder="" autocomplete="off" />
            <span class="field-hint">
              <a href="https://www.discogs.com/settings/developers" target="_blank" rel="noreferrer">generate token</a>
              · exact pressing credits
            </span>
          </label>
        {/if}
      </section>

      {#if store.config?.config_path}
        <div class="path">
          <span class="path-label">config file</span>
          <code>{store.config.config_path}</code>
        </div>
      {/if}

      {#if error}
        <div class="error">{error}</div>
      {/if}
      {#if saved && !error}
        <div class="ok">saved · restart to apply host changes</div>
      {/if}
    </div>

    <footer>
      <button class="btn" onclick={onClose} disabled={saving}>cancel</button>
      <button class="btn" onclick={() => save(false)} disabled={saving}>save</button>
      <button class="btn primary" onclick={() => save(true)} disabled={saving}>
        save and restart
      </button>
    </footer>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.55);
    backdrop-filter: blur(6px);
    -webkit-backdrop-filter: blur(6px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    animation: fade 150ms ease-out;
  }
  @keyframes fade {
    from { opacity: 0; }
    to { opacity: 1; }
  }
  .modal {
    width: 480px;
    max-width: calc(100vw - 32px);
    max-height: calc(100vh - 64px);
    background: var(--bg-1);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-strong);
    overflow: hidden;
    display: flex;
    flex-direction: column;
    animation: pop 180ms cubic-bezier(0.2, 0.9, 0.3, 1.1);
  }
  @keyframes pop {
    from { opacity: 0; transform: translateY(8px) scale(0.97); }
    to { opacity: 1; transform: none; }
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 18px 20px 12px;
    border-bottom: 1px solid var(--border-soft);
    flex-shrink: 0;
  }
  h2 {
    margin: 0;
    font-size: 15px;
    font-weight: 700;
    letter-spacing: -0.01em;
  }
  .close {
    width: 28px;
    height: 28px;
    border-radius: var(--radius-pill);
    color: var(--text-muted);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    transition: background 120ms ease, color 120ms ease;
  }
  .close:hover {
    background: var(--bg-hover);
    color: var(--text);
  }

  .form {
    padding: 16px 20px 4px;
    display: flex;
    flex-direction: column;
    gap: 18px;
    overflow: auto;
    flex: 1;
  }

  section {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  section h3 {
    margin: 0;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-mid);
  }
  .section-toggle {
    display: flex;
    align-items: center;
    gap: 8px;
    background: transparent;
    padding: 4px 0;
    cursor: pointer;
    text-align: left;
  }
  .section-toggle .chev {
    color: var(--text-muted);
    display: inline-flex;
    transition: transform 150ms ease;
  }
  .section-toggle .chev.open {
    transform: rotate(90deg);
  }
  .section-toggle .optional {
    margin-left: auto;
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-faint);
  }
  .hint {
    font-size: 11.5px;
    color: var(--text-muted);
    line-height: 1.5;
    margin: 0;
  }
  .hint code {
    background: var(--bg-2);
    padding: 1px 5px;
    border-radius: 4px;
    font-size: 11px;
    color: var(--text-mid);
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  label > span {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-mid);
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }
  .field-hint {
    font-size: 10.5px !important;
    text-transform: none !important;
    letter-spacing: normal !important;
    color: var(--text-faint) !important;
    font-weight: 400 !important;
  }
  .field-hint a {
    color: var(--accent);
    text-decoration: none;
  }
  .field-hint a:hover {
    text-decoration: underline;
  }
  .ports {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }

  input[type="text"],
  input[type="number"],
  input[type="password"],
  select {
    background: var(--bg-2);
    border: 1px solid var(--border-soft);
    border-radius: var(--radius-md);
    padding: 8px 12px;
    color: var(--text);
    outline: none;
    font: inherit;
    transition: border-color 120ms ease, box-shadow 120ms ease;
    width: 100%;
  }
  input:focus,
  select:focus {
    border-color: var(--accent-ring);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }
  select {
    appearance: none;
    cursor: pointer;
    background-image: linear-gradient(45deg, transparent 50%, var(--text-muted) 50%),
      linear-gradient(135deg, var(--text-muted) 50%, transparent 50%);
    background-position: calc(100% - 14px) 50%, calc(100% - 9px) 50%;
    background-size: 5px 5px, 5px 5px;
    background-repeat: no-repeat;
    padding-right: 28px;
  }

  .path {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 8px 10px;
    background: var(--bg-2);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-soft);
  }
  .path-label {
    font-size: 9.5px;
    font-weight: 600;
    color: var(--text-faint);
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  .path code {
    font-size: 11px;
    color: var(--text-mid);
    word-break: break-all;
  }

  .error {
    background: rgba(217, 102, 102, 0.12);
    border: 1px solid rgba(217, 102, 102, 0.4);
    color: #e58a8a;
    padding: 8px 12px;
    border-radius: var(--radius-md);
    font-size: 11.5px;
  }
  .ok {
    background: var(--accent-soft);
    border: 1px solid var(--accent-ring);
    color: var(--accent);
    padding: 8px 12px;
    border-radius: var(--radius-md);
    font-size: 11.5px;
  }

  footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 14px 20px 18px;
    border-top: 1px solid var(--border-soft);
    flex-shrink: 0;
  }
</style>
