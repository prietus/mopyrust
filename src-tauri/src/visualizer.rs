//! WebSocket client for mopidy-goodies' `/goodies/audio/visualizer` feed.
//!
//! The server broadcasts raw PCM (S16LE @ 48000 Hz stereo by convention) as
//! binary WebSocket frames. We mix to mono, accumulate into a fixed FFT
//! window, compute magnitudes, group into log-spaced bars, smooth with a
//! decay envelope, and emit each frame to the frontend via `viz:frame`.
//!
//! Lifecycle: one task per `visualizer_start(host)` call; the previous task
//! (if any) is aborted by replacing its `JoinHandle` in state. The task
//! reconnects with exponential backoff on its own, so transient network
//! blips don't require frontend intervention.

use std::sync::Arc;
use std::time::Duration;

use futures_util::StreamExt;
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tauri::async_runtime::JoinHandle;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;

/// Number of bars sent to the frontend. 32 is a sweet spot for both the
/// ratatui-style terminal display and a desktop canvas — dense enough to
/// look musical, sparse enough that each bar moves visibly.
pub const BARS: usize = 32;

/// FFT window length in mono samples. 2048 at 48 kHz ≈ 42.7 ms, giving
/// ~23.4 Hz bin resolution — good enough to separate low-end bars on a
/// log axis. Higher would smear transients; lower would lose bass detail.
const FFT_N: usize = 2048;

/// Source PCM convention. Has to match the FIFO branch in mopidy.conf.
const SAMPLE_RATE: f32 = 48000.0;

/// Per-frame attack (towards new peak) and decay (towards zero). Frame
/// rate is ≈ rate/HOP ≈ 48000/512 ≈ 94 fps, so a decay of 0.82 fades
/// from 1.0 to ~0.05 in ~15 frames (~160 ms) — snappy enough to feel
/// in sync with transients. Attack=1.0: snap to any louder peak.
const ATTACK: f32 = 1.0;
const DECAY: f32 = 0.82;

/// Handle stored in app state so `visualizer_stop` can abort the task.
#[derive(Default)]
pub struct VisualizerHandle(pub Mutex<Option<JoinHandle<()>>>);

#[derive(Serialize, Clone)]
struct FramePayload {
    /// Bar magnitudes in [0, 1], length `BARS`.
    bars: Vec<f32>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "lowercase")]
enum VizState { Connecting, Connected, Disconnected, Error }

#[derive(Serialize, Clone)]
struct StateEvent {
    state: VizState,
    error: Option<String>,
}

pub async fn start(app: AppHandle, handle: Arc<VisualizerHandle>, host: String) {
    // Replace any previous task — last call wins.
    let mut slot = handle.0.lock().await;
    if let Some(prev) = slot.take() {
        prev.abort();
    }
    let app2 = app.clone();
    let task = tauri::async_runtime::spawn(async move {
        run(app2, host).await;
    });
    *slot = Some(task);
}

pub async fn stop(handle: Arc<VisualizerHandle>) {
    let mut slot = handle.0.lock().await;
    if let Some(t) = slot.take() {
        t.abort();
    }
}

async fn run(app: AppHandle, host: String) {
    let url = format!("ws://{host}:6680/goodies/audio/visualizer");
    let mut backoff = Duration::from_secs(1);
    loop {
        let _ = app.emit("viz:state", StateEvent {
            state: VizState::Connecting,
            error: None,
        });
        match run_session(&app, &url).await {
            Ok(()) => {
                let _ = app.emit("viz:state", StateEvent {
                    state: VizState::Disconnected,
                    error: None,
                });
                backoff = Duration::from_secs(1);
            }
            Err(e) => {
                let _ = app.emit("viz:state", StateEvent {
                    state: VizState::Error,
                    error: Some(e),
                });
            }
        }
        sleep(backoff).await;
        backoff = (backoff * 2).min(Duration::from_secs(30));
    }
}

async fn run_session(app: &AppHandle, url: &str) -> Result<(), String> {
    let (ws, _) = connect_async(url)
        .await
        .map_err(|e| format!("ws connect: {e}"))?;
    let _ = app.emit("viz:state", StateEvent {
        state: VizState::Connected,
        error: None,
    });
    let (_w, mut r) = ws.split();

    let mut fft = FftCtx::new();

    while let Some(msg) = r.next().await {
        let msg = msg.map_err(|e| format!("ws read: {e}"))?;
        match msg {
            Message::Binary(bytes) => {
                fft.feed_pcm(&bytes);
                while let Some(bars) = fft.try_frame() {
                    let _ = app.emit("viz:frame", FramePayload { bars });
                }
            }
            Message::Close(_) => return Ok(()),
            // Server doesn't send pings or text but be forgiving anyway.
            _ => {}
        }
    }
    Ok(())
}

/// FFT pipeline: PCM bytes in, log-spaced smoothed bars out.
struct FftCtx {
    planner: FftPlanner<f32>,
    /// Rolling mono buffer; new samples appended, oldest dropped once we
    /// reach FFT_N. Re-computes on every `hop` new samples.
    mono: Vec<f32>,
    /// Sample count since the last FFT; we emit a frame every `hop`.
    since_last: usize,
    /// Hann window precomputed.
    window: Vec<f32>,
    /// Last emitted bar magnitudes — applied as attack/decay envelope on
    /// the next frame so bars don't snap-flash.
    smoothed: Vec<f32>,
    /// Frequency bins that mark the start of each log-spaced bar.
    bar_bins: Vec<usize>,
}

const HOP: usize = 512; // ~10.7 ms at 48 kHz → ~94 fps (75% overlap with FFT_N=2048)

impl FftCtx {
    fn new() -> Self {
        let window: Vec<f32> = (0..FFT_N)
            .map(|i| {
                // Hann window — flat at center, smooth taper at edges. Cuts
                // spectral leakage that would otherwise smear notes.
                let x = (i as f32) / (FFT_N as f32 - 1.0);
                0.5 - 0.5 * (2.0 * std::f32::consts::PI * x).cos()
            })
            .collect();
        let bar_bins = log_spaced_bins();
        Self {
            planner: FftPlanner::new(),
            mono: Vec::with_capacity(FFT_N + HOP),
            since_last: 0,
            window,
            smoothed: vec![0.0; BARS],
            bar_bins,
        }
    }

    fn feed_pcm(&mut self, bytes: &[u8]) {
        // Interpret as S16LE stereo, mix L+R to mono. `bytes` may not be
        // a whole number of stereo frames if the WS chunk happens to split
        // mid-frame; drop any trailing odd bytes.
        let frames = bytes.len() / 4; // 2 bytes × 2 channels
        for i in 0..frames {
            let o = i * 4;
            let l = i16::from_le_bytes([bytes[o], bytes[o + 1]]) as f32;
            let r = i16::from_le_bytes([bytes[o + 2], bytes[o + 3]]) as f32;
            let m = (l + r) * 0.5 / 32768.0;
            self.mono.push(m);
            self.since_last += 1;
        }
        // Cap the buffer so it can't grow unbounded if we ever skip frames.
        if self.mono.len() > FFT_N + HOP * 2 {
            let drop = self.mono.len() - FFT_N;
            self.mono.drain(..drop);
        }
    }

    fn try_frame(&mut self) -> Option<Vec<f32>> {
        if self.mono.len() < FFT_N || self.since_last < HOP {
            return None;
        }
        self.since_last = 0;
        let start = self.mono.len() - FFT_N;
        let slice = &self.mono[start..];
        let mut buf: Vec<Complex<f32>> = slice
            .iter()
            .zip(&self.window)
            .map(|(&s, &w)| Complex { re: s * w, im: 0.0 })
            .collect();
        let fft = self.planner.plan_fft_forward(FFT_N);
        fft.process(&mut buf);

        // Group into bars by log-spaced bin ranges, taking max magnitude
        // in each range (max plays better than mean for percussive content).
        let mut out = Vec::with_capacity(BARS);
        for i in 0..BARS {
            let lo = self.bar_bins[i];
            let hi = self.bar_bins[i + 1];
            let mut peak = 0.0f32;
            for k in lo..hi {
                let c = buf[k];
                let mag = (c.re * c.re + c.im * c.im).sqrt();
                if mag > peak {
                    peak = mag;
                }
            }
            // Compress magnitude — raw FFT magnitudes have a huge dynamic
            // range and look "all bass" linearly. `log10(1 + x)` gives a
            // perceptually reasonable curve; the divisor normalises it.
            let v = (1.0 + peak).log10() / 3.0;
            let v = v.clamp(0.0, 1.0);
            // Envelope: attack instantly upwards, decay slowly downwards.
            let prev = self.smoothed[i];
            let next = if v > prev {
                prev + (v - prev) * ATTACK
            } else {
                prev * DECAY
            };
            self.smoothed[i] = next;
            out.push(next);
        }
        Some(out)
    }
}

/// Log-spaced bin boundaries: low frequencies get few bins per bar, high
/// frequencies get many. Returns `BARS + 1` indices into `[0, FFT_N/2]`.
fn log_spaced_bins() -> Vec<usize> {
    // Audible range we care about: ~30 Hz to ~Nyquist/2. The very lowest
    // bins (DC, bin 1) carry rumble and don't help musically.
    let f_min: f32 = 30.0;
    let f_max: f32 = SAMPLE_RATE / 2.0 * 0.95;
    let log_min = f_min.ln();
    let log_max = f_max.ln();
    let mut out = Vec::with_capacity(BARS + 1);
    for i in 0..=BARS {
        let t = i as f32 / BARS as f32;
        let f = (log_min + (log_max - log_min) * t).exp();
        let bin = (f * FFT_N as f32 / SAMPLE_RATE) as usize;
        out.push(bin.min(FFT_N / 2 - 1));
    }
    // Ensure strict monotonicity (low bars can collapse to the same bin).
    for i in 1..out.len() {
        if out[i] <= out[i - 1] {
            out[i] = out[i - 1] + 1;
        }
    }
    out
}
