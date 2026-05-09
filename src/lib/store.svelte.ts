import { listen } from "@tauri-apps/api/event";
import { api } from "./api";
import type {
  AlbumCard,
  AlbumMeta,
  ArtistMeta,
  AudioFormat,
  AudioPayload,
  ConfigInfo,
  ConnEvent,
  ConnState,
  LibRef,
  LyricsResult,
  PlayState,
  Playlist,
  PlaybackSnapshot,
  SearchResult,
  TlTrack,
  Track,
} from "./types";

export type Section =
  | { kind: "now-playing" }
  | { kind: "search" }
  | { kind: "albums" }
  | { kind: "artists" }
  | { kind: "playlists" }
  | { kind: "browse" }
  | { kind: "queue" }
  | { kind: "album-detail"; uri: string; label: string }
  | { kind: "artist-detail"; name: string }
  | { kind: "playlist-detail"; uri: string; label: string };

const HEARTBEAT_MS = 15_000;
const TICK_MS = 1_000;

class Store {
  // routing
  navStack: Section[] = $state([{ kind: "now-playing" }]);

  // playback (Drift's model: separate fields, source of truth = Mopidy JSON-RPC)
  playback: PlayState | string = $state("stopped");
  current: Track | null = $state(null);
  currentTlid: number | null = $state(null);
  elapsed: number = $state(0); // seconds
  /** -1 when Mopidy has `mixer = none` (typical for bit-perfect remote setups). */
  volume: number = $state(-1);
  audioFormat: AudioFormat | null = $state(null);
  bitrate: number | null = $state(null);

  // connection (MPD idle subscription)
  conn: ConnState = $state("connecting");
  connError: string | null = $state(null);

  // queue
  queue: TlTrack[] = $state([]);

  // albums
  albums: AlbumCard[] = $state([]);
  albumsLoaded = $state(false);
  albumsLoading = $state(false);
  albumsFilter = $state("");

  // artists
  artists: string[] = $state([]);
  artistsLoaded = $state(false);
  artistsLoading = $state(false);

  // album detail
  albumTracks: Record<string, Track[]> = $state({});
  albumLoading: Record<string, boolean> = $state({});

  // artist detail (per-name)
  artistAlbums: Record<string, AlbumCard[]> = $state({});
  artistLoading: Record<string, boolean> = $state({});

  // playlists
  playlists: LibRef[] = $state([]);
  playlistsLoaded = $state(false);
  playlistsLoading = $state(false);
  playlistDetail: Record<string, Playlist> = $state({});
  playlistLoading: Record<string, boolean> = $state({});

  // browse
  browseStack: { label: string; uri: string | null }[] = $state([
    { label: "library", uri: null },
  ]);
  browseItems: LibRef[] = $state([]);
  browseLoading = $state(false);

  // search
  searchInput = $state("");
  searchQuery = $state("");
  searchResults: SearchResult[] = $state([]);
  searchLoading = $state(false);

  // covers
  covers: Record<string, string> = $state({});
  coverPending: Record<string, boolean> = $state({});

  // ui prefs
  nowPlayingPane: "none" | "queue" | "lyrics" = $state("none");

  // lyrics cache (per track URI). null = looked up, no result.
  lyrics: Record<string, LyricsResult | null> = $state({});
  lyricsPending: Record<string, boolean> = $state({});

  // metadata cache (MusicBrainz + Wikipedia). null = looked up, no result.
  albumMeta: Record<string, AlbumMeta | null> = $state({});
  albumMetaPending: Record<string, boolean> = $state({});
  artistMeta: Record<string, ArtistMeta | null> = $state({});
  artistMetaPending: Record<string, boolean> = $state({});

  // config
  config: ConfigInfo | null = $state(null);

  // ── derived ─────────────────────────────────────────────────────────────

  get section(): Section {
    return this.navStack[this.navStack.length - 1];
  }

  get rootKind(): Section["kind"] {
    const k = this.section.kind;
    if (k === "album-detail") return "albums";
    if (k === "artist-detail") return "artists";
    if (k === "playlist-detail") return "playlists";
    return k;
  }

  get duration(): number {
    return this.current?.length ? this.current.length / 1000 : 0;
  }

  get isPlaying(): boolean {
    return this.playback === "playing";
  }

  get hasVolume(): boolean {
    return this.volume >= 0;
  }

  // ── nav ─────────────────────────────────────────────────────────────────

  navTo(s: Section) {
    if (s.kind === "album-detail") {
      this.navStack.push(s);
      this.ensureAlbumDetail(s.uri);
    } else if (s.kind === "artist-detail") {
      this.navStack.push(s);
      this.ensureArtistAlbums(s.name);
    } else if (s.kind === "playlist-detail") {
      this.navStack.push(s);
      this.ensurePlaylistDetail(s.uri);
    } else {
      this.navStack = [s];
      if (s.kind === "albums") this.ensureAlbums();
      else if (s.kind === "artists") this.ensureArtists();
      else if (s.kind === "playlists") this.ensurePlaylists();
      else if (s.kind === "browse") this.ensureBrowse();
      else if (s.kind === "queue") this.refreshQueue();
    }
  }

  navBack() {
    if (this.navStack.length > 1) this.navStack.pop();
  }

  // ── covers ──────────────────────────────────────────────────────────────

  async ensureLyrics(track: Track | null, force = false) {
    if (!track) return;
    const key = track.uri;
    // Re-fetch when forced, or when we have no positive cache yet. Negative
    // results are NOT cached on the frontend — Mopidy may enrich metadata
    // (album name) shortly after track-changed, and we want to retry once
    // it does. The backend has its own positive+negative cache so retries
    // are cheap.
    if (!force && this.lyrics[key]) return;
    if (this.lyricsPending[key]) return;
    const artist = track.artists.map((a) => a.name).join(", ");
    const title = track.name;
    const album = track.album?.name ?? "";
    const durationMs = track.length ?? 0;
    if (!artist || !title) return;
    this.lyricsPending[key] = true;
    try {
      const res = await api.getLyrics({ artist, title, album, durationMs });
      if (res) {
        this.lyrics[key] = res;
      } else {
        // Use null sentinel so the UI can distinguish "tried, nothing" from
        // "not yet attempted". Doesn't block retries (we use truthy checks
        // and the `force` flag).
        this.lyrics[key] = null;
      }
    } catch (e) {
      console.error("lyrics", e);
    } finally {
      delete this.lyricsPending[key];
    }
  }

  async retryLyrics(track: Track | null) {
    if (!track) return;
    delete this.lyrics[track.uri];
    await this.ensureLyrics(track, true);
  }

  async ensureCover(uri: string | null | undefined) {
    if (!uri) return;
    if (this.covers[uri] || this.coverPending[uri]) return;
    this.coverPending[uri] = true;
    try {
      const data = await api.coverFor(uri);
      if (data) this.covers[uri] = data;
    } finally {
      delete this.coverPending[uri];
    }
  }

  // ── albums / artists ────────────────────────────────────────────────────

  async ensureAlbums() {
    if (this.albumsLoaded || this.albumsLoading) return;
    this.albumsLoading = true;
    try {
      const list = await api.discoverAlbums();
      this.albums = list;
      this.albumsLoaded = true;
      for (const a of list) this.ensureCover(a.uri);
    } catch (e) {
      console.error("albums", e);
    } finally {
      this.albumsLoading = false;
    }
  }

  async ensureArtists() {
    if (this.artistsLoaded || this.artistsLoading) return;
    this.artistsLoading = true;
    try {
      this.artists = await api.getArtists();
      this.artistsLoaded = true;
    } catch (e) {
      console.error("artists", e);
    } finally {
      this.artistsLoading = false;
    }
  }

  async ensurePlaylists() {
    if (this.playlistsLoaded || this.playlistsLoading) return;
    this.playlistsLoading = true;
    try {
      this.playlists = await api.getPlaylists();
      this.playlistsLoaded = true;
    } catch (e) {
      console.error("playlists", e);
    } finally {
      this.playlistsLoading = false;
    }
  }

  async ensurePlaylistDetail(uri: string) {
    if (this.playlistDetail[uri] || this.playlistLoading[uri]) return;
    this.playlistLoading[uri] = true;
    try {
      const pl = await api.lookupPlaylist(uri);
      if (pl) this.playlistDetail[uri] = pl;
    } catch (e) {
      console.error("playlist detail", e);
    } finally {
      delete this.playlistLoading[uri];
    }
  }

  async ensureArtistAlbums(name: string) {
    if (this.artistAlbums[name] || this.artistLoading[name]) return;
    this.artistLoading[name] = true;
    try {
      const list = await api.getArtistAlbums(name);
      this.artistAlbums[name] = list;
      for (const a of list) this.ensureCover(a.uri);
    } catch (e) {
      console.error("artist albums", e);
    } finally {
      delete this.artistLoading[name];
    }
  }

  async ensureAlbumMeta(artist: string, album: string) {
    const a = artist.trim();
    const b = album.trim();
    if (!a || !b) return;
    const key = `${a.toLowerCase()}::${b.toLowerCase()}`;
    if (key in this.albumMeta || this.albumMetaPending[key]) return;
    this.albumMetaPending[key] = true;
    try {
      const m = await api.getAlbumMetadata(a, b);
      this.albumMeta[key] = m && (m.release || m.wiki) ? m : null;
    } catch (e) {
      console.error("album meta", e);
      this.albumMeta[key] = null;
    } finally {
      delete this.albumMetaPending[key];
    }
  }

  async ensureArtistMeta(name: string) {
    const n = name.trim();
    if (!n) return;
    const key = n.toLowerCase();
    if (key in this.artistMeta || this.artistMetaPending[key]) return;
    this.artistMetaPending[key] = true;
    try {
      const m = await api.getArtistMetadata(n);
      this.artistMeta[key] = m && (m.info || m.wiki) ? m : null;
    } catch (e) {
      console.error("artist meta", e);
      this.artistMeta[key] = null;
    } finally {
      delete this.artistMetaPending[key];
    }
  }

  async ensureAlbumDetail(uri: string) {
    if (this.albumTracks[uri] || this.albumLoading[uri]) return;
    this.albumLoading[uri] = true;
    this.ensureCover(uri);
    try {
      const m = await api.lookup([uri]);
      this.albumTracks[uri] = (m[uri] ?? []).slice().sort((a, b) => {
        const da = a.disc_no ?? 1;
        const db = b.disc_no ?? 1;
        if (da !== db) return da - db;
        const ta = a.track_no ?? 0;
        const tb = b.track_no ?? 0;
        return ta - tb;
      });
    } catch (e) {
      console.error("album detail", e);
    } finally {
      delete this.albumLoading[uri];
    }
  }

  // ── browse ──────────────────────────────────────────────────────────────

  async ensureBrowse() {
    if (this.browseItems.length > 0 || this.browseLoading) return;
    await this.browseRefresh();
  }

  async browseRefresh() {
    const top = this.browseStack[this.browseStack.length - 1];
    this.browseLoading = true;
    try {
      this.browseItems = await api.browse(top.uri);
    } catch (e) {
      console.error("browse", e);
    } finally {
      this.browseLoading = false;
    }
  }

  async browseInto(label: string, uri: string) {
    this.browseStack.push({ label, uri });
    await this.browseRefresh();
  }

  async browseUp() {
    if (this.browseStack.length > 1) {
      this.browseStack.pop();
      await this.browseRefresh();
    }
  }

  // ── search ──────────────────────────────────────────────────────────────

  async runSearch(query: string) {
    const q = query.trim();
    if (!q) return;
    this.searchQuery = q;
    this.searchLoading = true;
    this.searchResults = [];
    try {
      this.searchResults = await api.search(q);
      for (const r of this.searchResults) {
        for (const a of r.albums) if (a.uri) this.ensureCover(a.uri);
      }
    } catch (e) {
      console.error("search", e);
    } finally {
      this.searchLoading = false;
    }
  }

  // ── queue ───────────────────────────────────────────────────────────────

  async refreshQueue() {
    try {
      this.queue = await api.getQueue();
    } catch (e) {
      console.error("queue", e);
    }
  }

  // ── transport ───────────────────────────────────────────────────────────

  async togglePlay() {
    try {
      if (this.playback === "playing") await api.pause();
      else if (this.playback === "paused") await api.resume();
      else await api.play();
    } catch (e) { console.error("toggle", e); }
    this.refreshPlayback();
  }

  async setSeek(seconds: number) {
    try { await api.seek(seconds); } catch (e) { console.error("seek", e); }
    this.refreshPlayback();
  }

  async setVolume(v: number) {
    this.volume = v;
    try { await api.setVolume(v); } catch (e) { console.error("setVolume", e); }
  }

  async playUri(uri: string) {
    try { await api.playUris([uri]); } catch (e) { console.error("playUri", e); }
  }
  async enqueueUri(uri: string) {
    try { await api.enqueueUris([uri]); } catch (e) { console.error("enqueueUri", e); }
  }
  async playUris(uris: string[]) {
    try { await api.playUris(uris); } catch (e) { console.error("playUris", e); }
  }
  async enqueueUris(uris: string[]) {
    try { await api.enqueueUris(uris); } catch (e) { console.error("enqueueUris", e); }
  }

  // ── playback refresh (snapshot) ────────────────────────────────────────

  async refreshPlayback() {
    try {
      const snap = await api.getPlayback();
      this.applyPlayback(snap);
    } catch (e) {
      console.warn("refreshPlayback failed", e);
    }
  }

  private applyPlayback(snap: PlaybackSnapshot) {
    this.playback = snap.state;
    this.elapsed = snap.elapsed_ms / 1000;
    // Keep -1 to signal "no mixer". UI hides the volume control in that case.
    this.volume = snap.volume;
    this.currentTlid = snap.current_tlid;
    const newUri = snap.current?.uri ?? null;
    const oldUri = this.current?.uri ?? null;
    this.current = snap.current;
    if (newUri && newUri !== oldUri) {
      this.ensureCover(newUri);
      // Pre-fetch lyrics if the lyrics pane is currently visible.
      if (this.nowPlayingPane === "lyrics" && snap.current) {
        this.ensureLyrics(snap.current);
      }
    }
  }

  // ── lifecycle ───────────────────────────────────────────────────────────

  async init() {
    try { this.config = await api.getConfig(); } catch (e) { console.error("config", e); }

    await Promise.all([
      listen<AudioPayload>("mpd:audio", (e) => {
        this.audioFormat = e.payload.audio;
        this.bitrate = e.payload.bitrate;
      }),
      listen<ConnEvent>("mpd:connection", (e) => {
        this.conn = e.payload.state;
        this.connError = e.payload.error;
        // On (re)connect, immediately fetch playback.
        if (e.payload.state === "connected") {
          this.refreshPlayback();
          this.refreshQueue();
        }
      }),
      listen<string[]>("mpd:changed", (e) => {
        // Any subsystem change → refresh playback. Queue/playlist also reload.
        this.refreshPlayback();
        if (e.payload.includes("playlist") || e.payload.includes("queue")) {
          this.refreshQueue();
        }
      }),
    ]);

    // 1Hz tick advances elapsed locally between idle events.
    setInterval(() => {
      if (this.playback === "playing") {
        const d = this.duration;
        this.elapsed = d > 0 ? Math.min(this.elapsed + 1, d) : this.elapsed + 1;
      }
    }, TICK_MS);

    // Heartbeat: idle TCP can stall on Wi-Fi sleep / NAT timeouts. JSON-RPC
    // is independent so this catches drift.
    setInterval(() => {
      this.refreshPlayback();
    }, HEARTBEAT_MS);

    // Refresh on window focus.
    window.addEventListener("focus", () => this.refreshPlayback());

    // Initial pull.
    await this.refreshPlayback();
    await this.refreshQueue();
  }
}

export const store = new Store();
