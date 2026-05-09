import { invoke } from "@tauri-apps/api/core";
import type {
  AlbumCard,
  AlbumMeta,
  ArtistMeta,
  ConfigInfo,
  LibRef,
  LyricsResult,
  PlaybackSnapshot,
  Playlist,
  SearchResult,
  TlTrack,
  Track,
} from "./types";

export const api = {
  search(query: string) {
    return invoke<SearchResult[]>("search", { query });
  },
  browse(uri: string | null) {
    return invoke<LibRef[]>("browse", { uri });
  },
  lookup(uris: string[]) {
    return invoke<Record<string, Track[]>>("lookup", { uris });
  },
  getArtists() {
    return invoke<string[]>("get_artists");
  },
  getArtistAlbums(artist: string) {
    return invoke<AlbumCard[]>("get_artist_albums", { artist });
  },
  getPlaylists() {
    return invoke<LibRef[]>("get_playlists");
  },
  lookupPlaylist(uri: string) {
    return invoke<Playlist | null>("lookup_playlist", { uri });
  },
  discoverAlbums() {
    return invoke<AlbumCard[]>("discover_albums");
  },
  getQueue() {
    return invoke<TlTrack[]>("get_queue");
  },
  playUris(uris: string[]) {
    return invoke<void>("play_uris", { uris });
  },
  enqueueUris(uris: string[]) {
    return invoke<void>("enqueue_uris", { uris });
  },
  playTlid(tlid: number) {
    return invoke<void>("play_tlid", { tlid });
  },
  removeTlid(tlid: number) {
    return invoke<void>("remove_tlid", { tlid });
  },
  moveTrack(from: number, to: number) {
    return invoke<void>("move_track", { from, to });
  },
  // Transport — all routed via Mopidy JSON-RPC.
  play() { return invoke<void>("play"); },
  pause() { return invoke<void>("pause"); },
  resume() { return invoke<void>("resume"); },
  stop() { return invoke<void>("stop"); },
  next() { return invoke<void>("next"); },
  previous() { return invoke<void>("previous"); },
  seek(seconds: number) { return invoke<void>("seek", { seconds }); },
  setVolume(volume: number) { return invoke<void>("set_volume", { volume }); },
  getPlayback() { return invoke<PlaybackSnapshot>("get_playback"); },
  coverFor(uri: string) {
    return invoke<string | null>("cover_for", { uri });
  },
  getLyrics(args: { artist: string; title: string; album: string; durationMs: number }) {
    return invoke<LyricsResult | null>("get_lyrics", {
      artist: args.artist,
      title: args.title,
      album: args.album,
      durationMs: args.durationMs,
    });
  },
  getConfig() {
    return invoke<ConfigInfo>("get_config");
  },
  saveConfig(cfg: {
    host: string;
    mpd_port: number;
    http_port: number;
    theme: string | null;
    lastfm_api_key: string | null;
    fanart_api_key: string | null;
    discogs_token: string | null;
  }) {
    return invoke<void>("save_config", { args: cfg });
  },
  restartApp() {
    return invoke<void>("restart_app");
  },
  getAlbumMetadata(artist: string, album: string) {
    return invoke<AlbumMeta>("get_album_metadata", { artist, album });
  },
  getArtistMetadata(name: string) {
    return invoke<ArtistMeta>("get_artist_metadata", { name });
  },
};
