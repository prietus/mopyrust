# mopyrust

A native Mopidy client for desktop, written in Rust + Tauri + Svelte 5.

Built around a remote Mopidy setup running `mopidy-mpd`, `mopidy-tidal` and
`mopidy-filesystem` feeding a USB DAC bit-perfect — but works with any Mopidy
server.

## Features

- Cross-source search (local library + Tidal) and library browsing
- Albums, Artists, Playlists, generic Browse, Queue with drag-to-reorder
- Now Playing with split view: queue + synced lyrics (lrclib.net)
- Bit-perfect audio chip showing the actual ALSA output rate / bits / channels
- Album & artist metadata enrichment via MusicBrainz + Wikipedia
  (genres, label/catalog, country, production credits, biography) — disk-cached
  for 30 days
- Optional API keys for Last.fm, Fanart.tv and Discogs (UI present)
- Themes: midnight, soft-dark, daylight, solar

## Architecture

- **Backend (`src-tauri/`)** — Rust + Tauri 2. Handles all Mopidy JSON-RPC,
  cover fetching (base64 data URLs to avoid WebView CORS), MPD idle
  subscription (only used for connection state and the bit-perfect audio chip),
  lyrics, and metadata aggregation.
- **Frontend (`src/`)** — Svelte 5 with runes, plain CSS variables for theming.

All transport, queue mutations and state queries go through Mopidy HTTP RPC.
MPD is a pure subscriber: subsystems for live updates, `status:audio` for the
output format. This avoids the desync between `mopidy-mpd`'s tracklist view
and core when the queue is mutated via JSON-RPC.

## Requirements

- A reachable Mopidy server with `mopidy-http` and `mopidy-mpd` enabled
- Rust stable toolchain
- Node 18+
- macOS, Linux or Windows (developed on macOS, primary target Linux desktop)

## Run

```sh
npm install
npm run tauri dev
```

Production build:

```sh
npm run tauri build
```

## Configuration

On first run a template is written to:

- macOS: `~/Library/Application Support/mopyrust/config.toml`
- Linux: `~/.config/mopyrust/config.toml`
- Windows: `%APPDATA%\mopyrust\config.toml`

Or use the in-app Settings dialog (gear icon in the sidebar) to set host,
ports, theme and optional API keys.

## License

MIT — see [LICENSE](LICENSE).
