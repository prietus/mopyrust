use std::sync::Arc;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::Notify;
use tokio::time::sleep;

use crate::models::AudioFormat;

/// MPD subsystems we listen to via `idle`.
const IDLE_CMD: &[u8] = b"idle player mixer options output update playlist database\n";

#[derive(Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum ConnState { Connecting, Connected, Disconnected, Error }

#[derive(Serialize, Clone)]
#[serde(rename_all = "kebab-case")]
struct ConnEvent {
    state: ConnState,
    error: Option<String>,
}

/// Spawn the MPD listener. Pure subscriber: emits `mpd:connection`,
/// `mpd:audio` (audio chain format from `status:audio`), and
/// `mpd:changed` (subsystems list). Never sends commands — those go via
/// Mopidy JSON-RPC because mopidy-mpd's tracklist desyncs from core when
/// the queue is mutated through the HTTP API.
pub fn spawn(app: AppHandle, host: String, port: u16, refresh: Arc<Notify>) {
    tauri::async_runtime::spawn(async move {
        run(app, host, port, refresh).await;
    });
}

async fn run(app: AppHandle, host: String, port: u16, refresh: Arc<Notify>) {
    loop {
        let _ = app.emit(
            "mpd:connection",
            ConnEvent { state: ConnState::Connecting, error: None },
        );
        // Reconnects probably also mean state changed since last view —
        // signal MPRIS so it doesn't lag.
        refresh.notify_one();
        match run_session(&app, &host, port, &refresh).await {
            Ok(()) => {
                let _ = app.emit(
                    "mpd:connection",
                    ConnEvent { state: ConnState::Disconnected, error: None },
                );
            }
            Err(e) => {
                let _ = app.emit(
                    "mpd:connection",
                    ConnEvent { state: ConnState::Error, error: Some(e) },
                );
            }
        }
        sleep(Duration::from_secs(3)).await;
    }
}

async fn run_session(
    app: &AppHandle,
    host: &str,
    port: u16,
    refresh: &Notify,
) -> Result<(), String> {
    let stream = TcpStream::connect((host, port))
        .await
        .map_err(|e| format!("mpd connect {host}:{port}: {e}"))?;
    let _ = stream.set_nodelay(true);
    let (r, mut w) = stream.into_split();
    let mut r = BufReader::new(r);

    // Greeting: "OK MPD x.y.z\n"
    let mut greet = String::new();
    r.read_line(&mut greet).await.map_err(|e| format!("read greet: {e}"))?;
    if greet.is_empty() {
        return Err("empty greeting".into());
    }
    let _ = app.emit(
        "mpd:connection",
        ConnEvent { state: ConnState::Connected, error: None },
    );

    // Initial audio format probe so the chip lights up immediately on connect.
    refresh_audio(&mut r, &mut w, app).await?;

    loop {
        // Block in idle until something changes.
        w.write_all(IDLE_CMD).await.map_err(|e| format!("idle: {e}"))?;
        let mut changed: Vec<String> = Vec::new();
        loop {
            let mut line = String::new();
            let n = r.read_line(&mut line).await.map_err(|e| format!("read: {e}"))?;
            if n == 0 {
                return Err("eof".into());
            }
            let t = line.trim_end();
            if t == "OK" {
                break;
            }
            if let Some(rest) = t.strip_prefix("ACK") {
                return Err(format!("idle ack: {rest}"));
            }
            if let Some(rest) = t.strip_prefix("changed: ") {
                changed.push(rest.to_string());
            }
        }
        if !changed.is_empty() {
            let _ = app.emit("mpd:changed", &changed);
            // The audio chain may have changed (next track, different sample
            // rate), so re-probe `status` and emit a fresh format.
            refresh_audio(&mut r, &mut w, app).await?;
            // Wake the MPRIS pusher so the OS widget mirrors the new state.
            refresh.notify_one();
        }
    }
}

async fn refresh_audio(
    r: &mut BufReader<tokio::net::tcp::OwnedReadHalf>,
    w: &mut tokio::net::tcp::OwnedWriteHalf,
    app: &AppHandle,
) -> Result<(), String> {
    w.write_all(b"status\n").await.map_err(|e| format!("status: {e}"))?;
    let mut audio: Option<AudioFormat> = None;
    let mut bitrate: Option<u32> = None;
    loop {
        let mut line = String::new();
        let n = r.read_line(&mut line).await.map_err(|e| format!("read: {e}"))?;
        if n == 0 {
            return Err("eof".into());
        }
        let t = line.trim_end();
        if t == "OK" {
            break;
        }
        if let Some(rest) = t.strip_prefix("ACK") {
            return Err(format!("status ack: {rest}"));
        }
        if let Some((k, v)) = t.split_once(": ") {
            match k {
                "audio" => audio = AudioFormat::parse(v),
                "bitrate" => bitrate = v.parse().ok(),
                _ => {}
            }
        }
    }
    let _ = app.emit(
        "mpd:audio",
        AudioPayload { audio, bitrate },
    );
    Ok(())
}

#[derive(Serialize, Clone)]
struct AudioPayload {
    audio: Option<AudioFormat>,
    bitrate: Option<u32>,
}
