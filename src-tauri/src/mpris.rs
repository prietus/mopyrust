//! System media controls bridge.
//!
//! Wraps the `souvlaki` crate, which abstracts MPRIS (Linux), SMTC (Windows)
//! and `MPRemoteCommandCenter` (macOS) behind one trait. This lets the OS
//! media keys, lock-screen widgets, GNOME shell, KDE Plasma, polybar/waybar,
//! etc. control mopyrust.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use souvlaki::{
    MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition,
    PlatformConfig, SeekDirection,
};
use tokio::sync::Notify;

use crate::mopidy_client::Client as Mopidy;

/// Default offset when MPRIS clients send a directionless ``Seek`` event.
const SEEK_DEFAULT_MS: i64 = 10_000;

pub struct MprisHandle {
    controls: Mutex<MediaControls>,
    cover_cache: tokio::sync::Mutex<CoverCache>,
}

#[derive(Default)]
struct CoverCache {
    last_uri: Option<String>,
    last_path: Option<PathBuf>,
}

impl MprisHandle {
    pub fn try_init(mopidy: Mopidy, refresh: Arc<Notify>) -> Option<Arc<Self>> {
        let config = PlatformConfig {
            dbus_name: "mopyrust",
            display_name: "mopyrust",
            hwnd: None,
        };

        let mut controls = match MediaControls::new(config) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("mpris: init failed, system controls disabled: {e:?}");
                return None;
            }
        };

        let mopidy_for_cb = mopidy.clone();
        if let Err(e) = controls.attach(move |event| {
            handle_event(event, mopidy_for_cb.clone());
        }) {
            eprintln!("mpris: attach failed, system controls disabled: {e:?}");
            return None;
        }

        let handle = Arc::new(Self {
            controls: Mutex::new(controls),
            cover_cache: tokio::sync::Mutex::new(CoverCache::default()),
        });

        // Outbound: react to MPD idle signals by pushing snapshot to the OS.
        let pusher = handle.clone();
        let mopidy_for_task = mopidy.clone();
        tauri::async_runtime::spawn(async move {
            pusher.refresh(&mopidy_for_task).await;
            loop {
                refresh.notified().await;
                pusher.refresh(&mopidy_for_task).await;
            }
        });

        Some(handle)
    }

    async fn refresh(&self, mopidy: &Mopidy) {
        let snap = match mopidy.fetch_playback().await {
            Ok(s) => s,
            Err(_) => return,
        };

        // Cover: fetch only when the track URI changes — repeated state
        // updates for the same track reuse the on-disk cache.
        let cover_url_owned: Option<String> = if let Some(track) = &snap.current {
            let mut cache = self.cover_cache.lock().await;
            if cache.last_uri.as_deref() != Some(track.uri.as_str()) {
                cache.last_uri = Some(track.uri.clone());
                cache.last_path = fetch_cover(mopidy, &track.uri).await;
            }
            cache.last_path.as_deref().map(path_to_file_url)
        } else {
            None
        };

        // Strings need to outlive the set_metadata call, build them in locals
        // and feed &str references at the end.
        let title;
        let artist;
        let album;
        let duration;
        let metadata = if let Some(track) = &snap.current {
            title = track.name.clone();
            artist = track
                .artists
                .iter()
                .map(|a| a.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            album = track
                .album
                .as_ref()
                .map(|a| a.name.clone())
                .unwrap_or_default();
            duration = track.length.map(|ms| Duration::from_millis(ms.max(0) as u64));
            MediaMetadata {
                title: Some(&title),
                artist: if artist.is_empty() { None } else { Some(&artist) },
                album: if album.is_empty() { None } else { Some(&album) },
                cover_url: cover_url_owned.as_deref(),
                duration,
            }
        } else {
            MediaMetadata::default()
        };

        // Position: report current elapsed so the OS widget shows a real
        // progress bar instead of a static one. The OS advances it locally
        // while playing — we don't need to push every second.
        let progress = Some(MediaPosition(Duration::from_millis(
            snap.elapsed_ms.max(0) as u64,
        )));
        let playback = match snap.state.as_str() {
            "playing" => MediaPlayback::Playing { progress },
            "paused" => MediaPlayback::Paused { progress },
            _ => MediaPlayback::Stopped,
        };

        let mut controls = self.controls.lock().unwrap();
        let _ = controls.set_metadata(metadata);
        let _ = controls.set_playback(playback);
    }
}

/// Callback invoked by souvlaki on the platform thread. Cannot block the
/// thread (D-Bus on Linux) so we hand off to Tokio immediately.
fn handle_event(event: MediaControlEvent, mopidy: Mopidy) {
    tauri::async_runtime::spawn(async move {
        let _ = match event {
            MediaControlEvent::Play => mopidy.playback_resume().await,
            MediaControlEvent::Pause => mopidy.playback_pause().await,
            MediaControlEvent::Toggle => match mopidy.fetch_playback().await {
                Ok(snap) if snap.state == "playing" => mopidy.playback_pause().await,
                _ => mopidy.playback_resume().await,
            },
            MediaControlEvent::Next => mopidy.playback_next().await,
            MediaControlEvent::Previous => mopidy.playback_previous().await,
            MediaControlEvent::Stop => mopidy.playback_stop().await,
            MediaControlEvent::SetPosition(MediaPosition(d)) => {
                mopidy.playback_seek(d.as_millis() as i64).await
            }
            MediaControlEvent::Seek(direction) => {
                relative_seek(&mopidy, direction, SEEK_DEFAULT_MS).await
            }
            MediaControlEvent::SeekBy(direction, d) => {
                relative_seek(&mopidy, direction, d.as_millis() as i64).await
            }
            // Raise/Quit/OpenUri intentionally ignored — Raise would require
            // bringing the Tauri window to front (doable later); Quit we
            // shouldn't honor without confirmation.
            _ => Ok(()),
        };
    });
}

async fn relative_seek(
    mopidy: &Mopidy,
    direction: SeekDirection,
    delta_ms: i64,
) -> Result<(), crate::mopidy_client::ClientError> {
    let snap = mopidy.fetch_playback().await?;
    let new_ms = match direction {
        SeekDirection::Forward => snap.elapsed_ms + delta_ms,
        SeekDirection::Backward => snap.elapsed_ms - delta_ms,
    };
    mopidy.playback_seek(new_ms.max(0)).await
}

// ── covers ─────────────────────────────────────────────────────────────────

async fn fetch_cover(mopidy: &Mopidy, uri: &str) -> Option<PathBuf> {
    let imgs = mopidy.get_images(vec![uri.to_string()]).await.ok()?;
    let list = imgs.get(uri)?;
    let chosen = list
        .iter()
        .max_by_key(|i| i.width.unwrap_or(0) as u64 * i.height.unwrap_or(0) as u64)
        .or_else(|| list.first())?;
    let url = mopidy.image_url(&chosen.uri);
    let bytes = mopidy.fetch_bytes(&url).await.ok()?;
    let dir = cover_cache_dir();
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("mpris: cover cache dir: {e}");
        return None;
    }
    let path = dir.join(format!("{}.{}", uri_hash(uri), ext_of_bytes(&bytes)));
    if let Err(e) = std::fs::write(&path, &bytes) {
        eprintln!("mpris: write cover {}: {e}", path.display());
        return None;
    }
    Some(path)
}

fn cover_cache_dir() -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push("mopyrust-mpris-covers");
    p
}

/// 16-hex-char FNV-1a hash. Deterministic across runs (unlike std's
/// `DefaultHasher` which uses a random seed) so the same track always hits
/// the same cached file.
fn uri_hash(uri: &str) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in uri.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100_0000_01b3);
    }
    format!("{h:016x}")
}

fn ext_of_bytes(bytes: &[u8]) -> &'static str {
    if bytes.starts_with(&[0xFF, 0xD8]) { "jpg" }
    else if bytes.starts_with(b"\x89PNG") { "png" }
    else if bytes.starts_with(b"RIFF") && bytes.len() > 12 && &bytes[8..12] == b"WEBP" { "webp" }
    else if bytes.starts_with(b"GIF8") { "gif" }
    else { "bin" }
}

fn path_to_file_url(path: &Path) -> String {
    // Cache filenames are hash-based ASCII, so no escaping needed; the
    // parent dir comes from std::env::temp_dir() which on Unix/macOS/Linux
    // doesn't contain spaces. Safe enough.
    format!("file://{}", path.display())
}
