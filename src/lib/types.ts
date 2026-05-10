export interface Artist {
  uri?: string | null;
  name: string;
}

export interface Album {
  uri?: string | null;
  name: string;
  artists: Artist[];
  num_tracks?: number | null;
  date?: string | null;
}

export interface Track {
  uri: string;
  name: string;
  artists: Artist[];
  album: Album | null;
  length?: number | null;
  track_no?: number | null;
  disc_no?: number | null;
  date?: string | null;
  bitrate?: number | null;
  genre?: string | null;
  composers: Artist[];
}

export interface LibRef {
  kind: string; // album | artist | directory | playlist | track
  uri: string;
  name: string;
}

export interface SearchResult {
  uri?: string | null;
  tracks: Track[];
  albums: Album[];
  artists: Artist[];
}

export interface TlTrack {
  tlid: number;
  track: Track;
}

export interface Playlist {
  uri: string;
  name: string;
  tracks: Track[];
  last_modified: number | null;
}

export type PlayState = "playing" | "paused" | "stopped";

export interface AudioFormat {
  rate: number;
  bits: number;
  channels: number;
}

export interface AudioPayload {
  audio: AudioFormat | null;
  bitrate: number | null;
}

export interface PlaybackSnapshot {
  state: PlayState | string;
  current: Track | null;
  current_tlid: number | null;
  elapsed_ms: number;
  volume: number;
}

export interface AlbumCard {
  uri: string;
  name: string;
  backend: string;
  artist: string;
  year: string | null;
}

export type ConnState = "connecting" | "connected" | "disconnected" | "error";

export interface ConnEvent {
  state: ConnState;
  error: string | null;
}

export interface LyricsResult {
  plain: string | null;
  synced: string | null;
  instrumental: boolean;
}

export interface SyncedLine {
  time: number; // seconds
  text: string;
}

export interface ConfigInfo {
  host: string;
  mpd_port: number;
  http_port: number;
  theme: string | null;
  lastfm_api_key: string | null;
  fanart_api_key: string | null;
  discogs_token: string | null;
  config_path: string | null;
}

export interface MbCredit {
  name: string;
  role: string;
}

export interface MbRelease {
  id: string;
  title: string;
  artist: string;
  date: string;
  country: string;
  label: string;
  catalog_number: string;
  barcode: string;
  status: string;
  credits: MbCredit[];
  wikipedia_slug: string | null;
  genres: string[];
}

export interface MbMember {
  name: string;
  role: string;
  period: string;
}

export interface MbArtistInfo {
  id: string;
  name: string;
  type: string;
  begin_date: string;
  end_date: string;
  area: string;
  wikipedia_slug: string | null;
  members: MbMember[];
}

export interface WikiSummary {
  title: string;
  extract: string;
  thumbnail_url: string | null;
  original_image_url: string | null;
  page_url: string;
  language: string;
}

export interface AlbumMeta {
  release: MbRelease | null;
  wiki: WikiSummary | null;
}

export interface ArtistMeta {
  info: MbArtistInfo | null;
  wiki: WikiSummary | null;
}

// ── tidal-goodies (optional server companion) ──────────────────────────

export interface GoodiesHealth {
  version: string;
  features: {
    favorites: boolean;
    favorites_active: boolean;
    stats: boolean;
  };
}

export interface RecentPlay {
  played_at: number; // unix seconds
  track_uri: string;
  name: string;
  artist: string;
  album: string;
  duration_ms: number;
  played_ms: number;
  completed: boolean;
}

export interface MostPlayedTrack {
  track_uri: string;
  name: string;
  artist: string;
  album: string;
  plays: number;
  total_played_ms: number;
}

export interface StatsTotals {
  total_plays: number;
  total_played_ms: number;
  unique_tracks: number;
  unique_artists: number;
  unique_albums: number;
  completed_plays: number;
}

export interface TopArtist {
  artist: string;
  plays: number;
  total_played_ms: number;
  sample_album_uri: string | null;
}

export interface TopAlbum {
  artist: string;
  album: string;
  album_uri: string | null;
  plays: number;
  total_played_ms: number;
}

export interface GenreCount {
  genre: string;
  plays: number;
  total_played_ms: number;
}

export interface DayOfWeekBucket {
  dow: number; // 0=Sun..6=Sat
  plays: number;
  total_played_ms: number;
}

export interface HourBucket {
  hour: number; // 0..23
  plays: number;
  total_played_ms: number;
}
