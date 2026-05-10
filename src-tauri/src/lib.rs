mod config;
mod lyrics;
mod metadata;
mod models;
mod mopidy_client;
mod mpd_worker;

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use base64::Engine;
use serde::Serialize;
use tauri::{Manager, State};

use crate::config::AppConfig;
use crate::models::{AlbumCard, LibRef, PlaybackSnapshot, Playlist, SearchResult, TlTrack, Track};
use crate::mopidy_client::Client as Mopidy;

pub struct AppState {
    pub cfg: Mutex<AppConfig>,
    pub mopidy: Mopidy,
    pub lyrics_cache: lyrics::LyricsCache,
}

#[derive(Serialize)]
pub struct ConfigInfo {
    pub host: String,
    pub mpd_port: u16,
    pub http_port: u16,
    pub theme: Option<String>,
    pub lastfm_api_key: Option<String>,
    pub fanart_api_key: Option<String>,
    pub discogs_token: Option<String>,
    pub config_path: Option<String>,
}

// ── library / search ─────────────────────────────────────────────────────

const MAX_TRACK_HITS: usize = 100;

#[tauri::command]
async fn search(state: State<'_, AppState>, query: String) -> Result<Vec<SearchResult>, String> {
    let mut results = state.mopidy.search(&query, None).await.map_err(|e| e.0)?;
    // Tidal returns thousands of tracks for popular queries; cap so the UI
    // stays snappy. Albums/artists are already capped server-side.
    let mut total: usize = 0;
    for r in &mut results {
        if total >= MAX_TRACK_HITS {
            r.tracks.clear();
            continue;
        }
        let remaining = MAX_TRACK_HITS - total;
        if r.tracks.len() > remaining {
            r.tracks.truncate(remaining);
        }
        total += r.tracks.len();
    }
    Ok(results)
}

#[tauri::command]
async fn browse(state: State<'_, AppState>, uri: Option<String>) -> Result<Vec<LibRef>, String> {
    state.mopidy.browse(uri).await.map_err(|e| e.0)
}

#[tauri::command]
async fn lookup(
    state: State<'_, AppState>,
    uris: Vec<String>,
) -> Result<HashMap<String, Vec<Track>>, String> {
    state.mopidy.lookup(uris).await.map_err(|e| e.0)
}

#[tauri::command]
async fn get_artist_albums(
    state: State<'_, AppState>,
    artist: String,
) -> Result<Vec<AlbumCard>, String> {
    let mopidy = state.mopidy.clone();
    let mut q: HashMap<String, Vec<String>> = HashMap::new();
    q.insert("albumartist".into(), vec![artist.clone()]);
    let results = mopidy.search_query(q, None, true).await.map_err(|e| e.0)?;

    let mut seen: HashSet<String> = HashSet::new();
    let mut cards: Vec<AlbumCard> = Vec::new();

    for r in results {
        for t in r.tracks {
            let Some(album) = &t.album else { continue };
            if album.name.is_empty() { continue }
            let album_uri = album.uri.clone().unwrap_or_default();
            let date = album.date.clone().or_else(|| t.date.clone()).unwrap_or_default();
            let key = if !album_uri.is_empty() {
                album_uri.clone()
            } else {
                format!("{}|{}", album.name.to_lowercase(), date)
            };
            if !seen.insert(key) { continue; }

            let backend = backend_label(if !album_uri.is_empty() { &album_uri } else { &t.uri }).to_string();
            let year = if !date.is_empty() {
                Some(date.split('-').next().unwrap_or(&date).to_string())
            } else { None };
            cards.push(AlbumCard {
                uri: if album_uri.is_empty() { t.uri.clone() } else { album_uri },
                name: album.name.clone(),
                backend,
                artist: artist.clone(),
                year,
            });
        }
    }

    // Sort chronological with nulls last; tiebreak by name.
    cards.sort_by(|a, b| {
        let ay = a.year.as_deref();
        let by = b.year.as_deref();
        match (ay, by) {
            (Some(x), Some(y)) if x != y => x.cmp(y),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            _ => sort_key(&a.name).cmp(&sort_key(&b.name)),
        }
    });
    Ok(cards)
}

#[tauri::command]
async fn get_playlists(state: State<'_, AppState>) -> Result<Vec<LibRef>, String> {
    state.mopidy.playlists_as_list().await.map_err(|e| e.0)
}

#[tauri::command]
async fn lookup_playlist(state: State<'_, AppState>, uri: String) -> Result<Option<Playlist>, String> {
    state.mopidy.playlist_lookup(&uri).await.map_err(|e| e.0)
}

#[tauri::command]
async fn get_artists(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let mut artists = state
        .mopidy
        .get_distinct("albumartist")
        .await
        .map_err(|e| e.0)?;
    artists.retain(|a| !a.is_empty());
    artists.sort_by(|a, b| sort_key(a).cmp(&sort_key(b)));
    Ok(artists)
}

#[tauri::command]
async fn discover_albums(state: State<'_, AppState>) -> Result<Vec<AlbumCard>, String> {
    let mopidy = state.mopidy.clone();
    // Canonical URIs — much more reliable than fuzzy "find subnode named album".
    let local = mopidy
        .browse(Some("local:directory?type=album".into()))
        .await
        .unwrap_or_default();
    let tidal = mopidy
        .browse(Some("tidal:my_albums".into()))
        .await
        .unwrap_or_default();

    let combined: Vec<LibRef> = local
        .into_iter()
        .chain(tidal.into_iter())
        .filter(|r| r.kind.eq_ignore_ascii_case("album"))
        .collect();

    if combined.is_empty() {
        return Ok(Vec::new());
    }

    // Single batched lookup to enrich with artist/year.
    let uris: Vec<String> = combined.iter().map(|r| r.uri.clone()).collect();
    let lookup = mopidy.lookup(uris).await.unwrap_or_default();

    let mut cards: Vec<AlbumCard> = Vec::with_capacity(combined.len());
    for r in combined {
        let tracks = lookup.get(&r.uri).cloned().unwrap_or_default();
        let first = tracks.first();
        let artist = first
            .map(|t| {
                t.album
                    .as_ref()
                    .and_then(|a| a.artists.first().map(|x| x.name.clone()))
                    .filter(|s| !s.is_empty())
                    .or_else(|| t.artists.first().map(|x| x.name.clone()))
                    .unwrap_or_default()
            })
            .unwrap_or_default();
        let year = first.and_then(|t| {
            t.album
                .as_ref()
                .and_then(|a| a.date.clone())
                .or_else(|| t.date.clone())
                .map(|d| d.split('-').next().unwrap_or(&d).to_string())
        });
        cards.push(AlbumCard {
            uri: r.uri.clone(),
            name: r.name,
            backend: backend_label(&r.uri).to_string(),
            artist,
            year,
        });
    }

    cards.sort_by(|a, b| sort_key(&a.name).cmp(&sort_key(&b.name)));
    Ok(cards)
}

fn sort_key(title: &str) -> String {
    let lower = title.to_lowercase();
    for prefix in ["the ", "a ", "an "] {
        if let Some(rest) = lower.strip_prefix(prefix) {
            return rest.trim().to_string();
        }
    }
    lower.trim().to_string()
}

fn backend_label(uri: &str) -> &'static str {
    if uri.starts_with("tidal:") { "tidal" }
    else if uri.starts_with("local:") { "local" }
    else if uri.starts_with("file:") { "file" }
    else if uri.starts_with("spotify:") { "spotify" }
    else if uri.starts_with("podcast:") { "podcast" }
    else if uri.starts_with("m3u:") { "m3u" }
    else if uri.starts_with("youtube:") { "youtube" }
    else { "other" }
}

// ── tracklist ────────────────────────────────────────────────────────────

#[tauri::command]
async fn get_queue(state: State<'_, AppState>) -> Result<Vec<TlTrack>, String> {
    state.mopidy.get_tl_tracks().await.map_err(|e| e.0)
}

/// Replace the queue with `uris` and start playback at the first track.
/// All queue mutations go through JSON-RPC; mopidy-mpd's `add` rejects many
/// Tidal URIs and its tracklist view desyncs from core after JSON-RPC mutation,
/// which makes MPD `play`/`playid` no-op.
#[tauri::command]
async fn play_uris(state: State<'_, AppState>, uris: Vec<String>) -> Result<(), String> {
    state.mopidy.tracklist_clear().await.map_err(|e| e.0)?;
    if uris.is_empty() {
        return Ok(());
    }
    let added = state.mopidy.tracklist_add(uris, None).await.map_err(|e| e.0)?;
    let first_tlid = added.first().map(|t| t.tlid);
    state.mopidy.playback_play(first_tlid).await.map_err(|e| e.0)?;
    Ok(())
}

#[tauri::command]
async fn enqueue_uris(state: State<'_, AppState>, uris: Vec<String>) -> Result<(), String> {
    if uris.is_empty() {
        return Ok(());
    }
    state.mopidy.tracklist_add(uris, None).await.map_err(|e| e.0)?;
    Ok(())
}

/// Insert `uris` immediately after the currently-playing track. Falls back to
/// appending when nothing is playing.
#[tauri::command]
async fn play_next_uris(
    state: State<'_, AppState>,
    uris: Vec<String>,
    current_tlid: Option<u32>,
) -> Result<(), String> {
    if uris.is_empty() {
        return Ok(());
    }
    let at_position = match current_tlid {
        Some(tlid) => state
            .mopidy
            .tracklist_index(tlid)
            .await
            .map_err(|e| e.0)?
            .map(|i| i + 1),
        None => None,
    };
    state
        .mopidy
        .tracklist_add(uris, at_position)
        .await
        .map_err(|e| e.0)?;
    Ok(())
}

// ── tidal-goodies (favorites) ────────────────────────────────────────────

fn parse_tidal_album_id(uri: &str) -> Result<&str, String> {
    uri.strip_prefix("tidal:album:")
        .ok_or_else(|| format!("not a tidal album URI: {uri}"))
}

#[tauri::command]
async fn get_tidal_favorite_album_ids(
    state: State<'_, AppState>,
) -> Result<Option<Vec<String>>, String> {
    state
        .mopidy
        .goodies_favorite_album_ids()
        .await
        .map_err(|e| e.0)
}

#[tauri::command]
async fn set_tidal_album_favorite(
    state: State<'_, AppState>,
    uri: String,
    favorited: bool,
) -> Result<bool, String> {
    let id = parse_tidal_album_id(&uri)?;
    state
        .mopidy
        .goodies_set_album_favorite(id, favorited)
        .await
        .map_err(|e| e.0)
}

#[tauri::command]
async fn refresh_library(state: State<'_, AppState>, uri: Option<String>) -> Result<(), String> {
    state
        .mopidy
        .library_refresh(uri.as_deref())
        .await
        .map_err(|e| e.0)
}

#[tauri::command]
async fn goodies_health(
    state: State<'_, AppState>,
) -> Result<Option<serde_json::Value>, String> {
    state.mopidy.goodies_health().await.map_err(|e| e.0)
}

#[tauri::command]
async fn goodies_stats_recent(
    state: State<'_, AppState>,
    limit: u32,
) -> Result<serde_json::Value, String> {
    state.mopidy.goodies_stats_recent(limit).await.map_err(|e| e.0)
}

#[tauri::command]
async fn goodies_stats_most_played(
    state: State<'_, AppState>,
    limit: u32,
    since: Option<i64>,
) -> Result<serde_json::Value, String> {
    state
        .mopidy
        .goodies_stats_most_played(limit, since)
        .await
        .map_err(|e| e.0)
}

#[tauri::command]
async fn goodies_stats_totals(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    state.mopidy.goodies_stats_totals().await.map_err(|e| e.0)
}

#[tauri::command]
async fn goodies_stats_top_artists(
    state: State<'_, AppState>,
    limit: u32,
    since: Option<i64>,
) -> Result<serde_json::Value, String> {
    state
        .mopidy
        .goodies_stats_top_artists(limit, since)
        .await
        .map_err(|e| e.0)
}

#[tauri::command]
async fn goodies_stats_top_albums(
    state: State<'_, AppState>,
    limit: u32,
    since: Option<i64>,
) -> Result<serde_json::Value, String> {
    state
        .mopidy
        .goodies_stats_top_albums(limit, since)
        .await
        .map_err(|e| e.0)
}

#[tauri::command]
async fn goodies_stats_by_genre(
    state: State<'_, AppState>,
    limit: u32,
    since: Option<i64>,
) -> Result<serde_json::Value, String> {
    state
        .mopidy
        .goodies_stats_by_genre(limit, since)
        .await
        .map_err(|e| e.0)
}

#[tauri::command]
async fn goodies_stats_by_day_of_week(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    state.mopidy.goodies_stats_by_day_of_week().await.map_err(|e| e.0)
}

#[tauri::command]
async fn goodies_stats_by_hour(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    state.mopidy.goodies_stats_by_hour().await.map_err(|e| e.0)
}

/// Append `uris` (resolved to Tracks via library lookup) to the existing
/// playlist at `playlist_uri` and persist it. Tidal-backed playlists may not
/// support save and will surface the backend error.
#[tauri::command]
async fn add_uris_to_playlist(
    state: State<'_, AppState>,
    playlist_uri: String,
    uris: Vec<String>,
) -> Result<(), String> {
    if uris.is_empty() {
        return Ok(());
    }
    let mut pl = state
        .mopidy
        .playlist_lookup(&playlist_uri)
        .await
        .map_err(|e| e.0)?
        .ok_or_else(|| "playlist not found".to_string())?;
    let resolved = state.mopidy.lookup(uris).await.map_err(|e| e.0)?;
    for tracks in resolved.into_values() {
        pl.tracks.extend(tracks);
    }
    state.mopidy.playlist_save(&pl).await.map_err(|e| e.0)?;
    Ok(())
}

#[tauri::command]
async fn play_tlid(state: State<'_, AppState>, tlid: u32) -> Result<(), String> {
    state.mopidy.playback_play(Some(tlid)).await.map_err(|e| e.0)
}

#[tauri::command]
async fn remove_tlid(state: State<'_, AppState>, tlid: u32) -> Result<(), String> {
    state.mopidy.tracklist_remove(vec![tlid]).await.map_err(|e| e.0)?;
    Ok(())
}

/// Move the queue entry at `from` to `to` (0-based positions, like SwiftUI's
/// `.onMove`). When moving down, the source row is removed before insertion,
/// so we adjust the target.
#[tauri::command]
async fn move_track(state: State<'_, AppState>, from: u32, to: u32) -> Result<(), String> {
    if from == to {
        return Ok(());
    }
    let target = if to > from { to - 1 } else { to };
    state
        .mopidy
        .tracklist_move(from, from + 1, target)
        .await
        .map_err(|e| e.0)?;
    Ok(())
}

// ── transport ────────────────────────────────────────────────────────────

#[tauri::command]
async fn play(state: State<'_, AppState>) -> Result<(), String> {
    state.mopidy.playback_play(None).await.map_err(|e| e.0)
}

#[tauri::command]
async fn pause(state: State<'_, AppState>) -> Result<(), String> {
    state.mopidy.playback_pause().await.map_err(|e| e.0)
}

#[tauri::command]
async fn resume(state: State<'_, AppState>) -> Result<(), String> {
    state.mopidy.playback_resume().await.map_err(|e| e.0)
}

#[tauri::command]
async fn stop(state: State<'_, AppState>) -> Result<(), String> {
    state.mopidy.playback_stop().await.map_err(|e| e.0)
}

#[tauri::command]
async fn next(state: State<'_, AppState>) -> Result<(), String> {
    state.mopidy.playback_next().await.map_err(|e| e.0)
}

#[tauri::command]
async fn previous(state: State<'_, AppState>) -> Result<(), String> {
    state.mopidy.playback_previous().await.map_err(|e| e.0)
}

#[tauri::command]
async fn seek(state: State<'_, AppState>, seconds: f64) -> Result<(), String> {
    let ms = (seconds * 1000.0).max(0.0) as i64;
    state.mopidy.playback_seek(ms).await.map_err(|e| e.0)
}

#[tauri::command]
async fn set_volume(state: State<'_, AppState>, volume: u32) -> Result<(), String> {
    state.mopidy.set_volume(volume).await.map_err(|e| e.0)
}

#[tauri::command]
async fn get_playback(state: State<'_, AppState>) -> Result<PlaybackSnapshot, String> {
    state.mopidy.fetch_playback().await.map_err(|e| e.0)
}

// ── covers ───────────────────────────────────────────────────────────────

#[tauri::command]
async fn cover_for(state: State<'_, AppState>, uri: String) -> Result<Option<String>, String> {
    let imgs = state
        .mopidy
        .get_images(vec![uri.clone()])
        .await
        .map_err(|e| e.0)?;
    let Some(list) = imgs.get(&uri) else { return Ok(None) };
    // Prefer largest variant; fall back to first.
    let chosen = list
        .iter()
        .max_by_key(|i| i.width.unwrap_or(0) as u64 * i.height.unwrap_or(0) as u64)
        .or_else(|| list.first());
    let Some(img) = chosen else { return Ok(None) };
    let url = state.mopidy.image_url(&img.uri);
    let bytes = state.mopidy.fetch_bytes(&url).await.map_err(|e| e.0)?;
    let mime = sniff_mime(&bytes);
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(Some(format!("data:{mime};base64,{b64}")))
}

fn sniff_mime(bytes: &[u8]) -> &'static str {
    if bytes.starts_with(&[0xFF, 0xD8]) { "image/jpeg" }
    else if bytes.starts_with(b"\x89PNG") { "image/png" }
    else if bytes.starts_with(b"RIFF") && bytes.len() > 12 && &bytes[8..12] == b"WEBP" { "image/webp" }
    else if bytes.starts_with(b"GIF8") { "image/gif" }
    else { "application/octet-stream" }
}

// ── config ───────────────────────────────────────────────────────────────

#[tauri::command]
async fn get_lyrics(
    state: State<'_, AppState>,
    artist: String,
    title: String,
    album: String,
    duration_ms: i64,
) -> Result<Option<lyrics::LyricsResult>, String> {
    lyrics::fetch_lyrics(&state.lyrics_cache, &artist, &title, &album, duration_ms).await
}

#[tauri::command]
async fn get_config(state: State<'_, AppState>) -> Result<ConfigInfo, String> {
    let cfg = state.cfg.lock().map_err(|e| e.to_string())?.clone();
    Ok(ConfigInfo {
        host: cfg.host,
        mpd_port: cfg.mpd_port,
        http_port: cfg.http_port,
        theme: cfg.theme,
        lastfm_api_key: cfg.lastfm_api_key,
        fanart_api_key: cfg.fanart_api_key,
        discogs_token: cfg.discogs_token,
        config_path: config::config_path().map(|p| p.display().to_string()),
    })
}

#[derive(serde::Deserialize)]
pub struct SaveConfigArgs {
    host: String,
    mpd_port: u16,
    http_port: u16,
    theme: Option<String>,
    #[serde(default)]
    lastfm_api_key: Option<String>,
    #[serde(default)]
    fanart_api_key: Option<String>,
    #[serde(default)]
    discogs_token: Option<String>,
}

#[tauri::command]
async fn save_config(state: State<'_, AppState>, args: SaveConfigArgs) -> Result<(), String> {
    let host = args.host.trim();
    if host.is_empty() {
        return Err("empty host".into());
    }
    if args.mpd_port == 0 || args.http_port == 0 {
        return Err("invalid ports".into());
    }
    let trim_opt = |s: Option<String>| s.map(|x| x.trim().to_string()).filter(|x| !x.is_empty());
    let cfg = AppConfig {
        host: host.to_string(),
        mpd_port: args.mpd_port,
        http_port: args.http_port,
        theme: args.theme.filter(|s| !s.is_empty()),
        lastfm_api_key: trim_opt(args.lastfm_api_key),
        fanart_api_key: trim_opt(args.fanart_api_key),
        discogs_token: trim_opt(args.discogs_token),
    };
    config::save(&cfg)?;
    *state.cfg.lock().map_err(|e| e.to_string())? = cfg;
    Ok(())
}

#[tauri::command]
async fn restart_app(app: tauri::AppHandle) {
    app.restart();
}

// ── metadata ─────────────────────────────────────────────────────────────

#[tauri::command]
async fn get_album_metadata(
    meta: State<'_, metadata::MetadataState>,
    artist: String,
    album: String,
) -> Result<metadata::AlbumMeta, String> {
    Ok(meta.album(&artist, &album).await)
}

#[tauri::command]
async fn get_artist_metadata(
    meta: State<'_, metadata::MetadataState>,
    name: String,
) -> Result<metadata::ArtistMeta, String> {
    Ok(meta.artist(&name).await)
}

// ── app entry point ──────────────────────────────────────────────────────

/// macOS' default soft limit for open file descriptors is 256 — far too low
/// for a desktop app that holds an MPD idle socket, JSON-RPC connections, a
/// cover-cache HTTP keep-alive pool and metadata clients all at once. When
/// FDs run out, even `getaddrinfo` starts failing with EAI_NONAME because it
/// can't open the UDP socket to query the resolver. Bump the soft limit to
/// the hard limit (typically 10k+) so we never see EMFILE in normal use.
#[cfg(unix)]
fn raise_fd_limit() {
    unsafe {
        let mut rl: libc::rlimit = std::mem::zeroed();
        if libc::getrlimit(libc::RLIMIT_NOFILE, &mut rl) != 0 {
            return;
        }
        let target = rl.rlim_max.min(65536);
        if rl.rlim_cur < target {
            rl.rlim_cur = target;
            let _ = libc::setrlimit(libc::RLIMIT_NOFILE, &rl);
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(unix)]
    raise_fd_limit();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let cfg = config::load_or_template();
            let mopidy = Mopidy::new(&cfg.host, cfg.http_port);
            mpd_worker::spawn(app.handle().clone(), cfg.host.clone(), cfg.mpd_port);
            app.manage(AppState {
                cfg: Mutex::new(cfg),
                mopidy,
                lyrics_cache: lyrics::LyricsCache::new(),
            });
            app.manage(metadata::MetadataState::new());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            search,
            browse,
            lookup,
            get_artists,
            get_artist_albums,
            get_playlists,
            lookup_playlist,
            discover_albums,
            get_queue,
            play_uris,
            enqueue_uris,
            play_next_uris,
            add_uris_to_playlist,
            get_tidal_favorite_album_ids,
            set_tidal_album_favorite,
            refresh_library,
            goodies_health,
            goodies_stats_recent,
            goodies_stats_most_played,
            goodies_stats_totals,
            goodies_stats_top_artists,
            goodies_stats_top_albums,
            goodies_stats_by_genre,
            goodies_stats_by_day_of_week,
            goodies_stats_by_hour,
            play_tlid,
            remove_tlid,
            move_track,
            play,
            pause,
            resume,
            stop,
            next,
            previous,
            seek,
            set_volume,
            get_playback,
            cover_for,
            get_lyrics,
            get_config,
            save_config,
            restart_app,
            get_album_metadata,
            get_artist_metadata,
        ])
        .run(tauri::generate_context!())
        .expect("error while running mopyrust");
}
