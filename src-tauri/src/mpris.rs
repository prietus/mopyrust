//! System media controls bridge.
//!
//! Wraps the `souvlaki` crate, which abstracts MPRIS (Linux), SMTC (Windows)
//! and `MPRemoteCommandCenter` (macOS) behind one trait. This lets the OS
//! media keys, lock-screen widgets, GNOME shell, KDE Plasma, polybar/waybar,
//! etc. control mopyrust.
//!
//! Architecture:
//!   - One [`MediaControls`] instance owned by an [`Arc<MprisHandle>`] inside
//!     [`AppState`].
//!   - `try_init` registers the inbound callback (Play/Pause/Next/Previous/...)
//!     which spawns Tokio tasks calling the mopidy client.
//!   - A long-lived task awaits a [`Notify`] signal from the MPD idle worker;
//!     on each notification it fetches a fresh `PlaybackSnapshot` and pushes
//!     metadata + state to the OS.
//!   - On platforms where souvlaki cannot initialize (e.g. headless CI, no
//!     D-Bus session), `try_init` returns `None` and mopyrust runs without
//!     system controls. No noisy errors.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use souvlaki::{
    MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, PlatformConfig,
};
use tokio::sync::Notify;

use crate::mopidy_client::Client as Mopidy;

pub struct MprisHandle {
    controls: Mutex<MediaControls>,
}

impl MprisHandle {
    pub fn try_init(mopidy: Mopidy, refresh: Arc<Notify>) -> Option<Arc<Self>> {
        let config = PlatformConfig {
            dbus_name: "mopyrust",
            display_name: "mopyrust",
            // Windows-only; not relevant on Linux/macOS where souvlaki
            // figures out the connection itself.
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
        });

        // Outbound: react to MPD idle signals by pushing snapshot to the OS.
        let pusher = handle.clone();
        let mopidy_for_task = mopidy.clone();
        tauri::async_runtime::spawn(async move {
            // Initial state — useful so the lockscreen widget shows something
            // immediately on app launch even if nothing is playing yet.
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

        // Strings need to outlive the set_metadata call (which copies them
        // internally before returning), so build them in locals first.
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
                cover_url: None, // TODO: temp-file cover
                duration,
            }
        } else {
            MediaMetadata::default()
        };

        let playback = match snap.state.as_str() {
            "playing" => MediaPlayback::Playing { progress: None },
            "paused" => MediaPlayback::Paused { progress: None },
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
            // Seek/SetPosition would be nice but require knowing current
            // position to translate relative seeks; skip for v1.
            _ => Ok(()),
        };
    });
}
