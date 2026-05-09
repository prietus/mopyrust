use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

const BASE_URL: &str = "https://lrclib.net/api";
const USER_AGENT: &str = concat!("mopyrust/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LyricsResult {
    pub plain: Option<String>,
    pub synced: Option<String>,
    pub instrumental: bool,
}

impl LyricsResult {
    #[allow(dead_code)]
    pub fn has_any_text(&self) -> bool {
        self.plain.as_deref().map(|s| !s.is_empty()).unwrap_or(false)
            || self.synced.as_deref().map(|s| !s.is_empty()).unwrap_or(false)
    }
}

#[derive(Deserialize)]
struct LrclibResp {
    #[serde(default, rename = "plainLyrics")]
    plain_lyrics: Option<String>,
    #[serde(default, rename = "syncedLyrics")]
    synced_lyrics: Option<String>,
    #[serde(default)]
    instrumental: bool,
}

impl LrclibResp {
    fn into_result(self) -> LyricsResult {
        LyricsResult {
            plain: self.plain_lyrics.filter(|s| !s.is_empty()),
            synced: self.synced_lyrics.filter(|s| !s.is_empty()),
            instrumental: self.instrumental,
        }
    }
}

#[derive(Clone)]
pub struct LyricsCache {
    inner: Arc<Mutex<HashMap<String, Option<LyricsResult>>>>,
    client: reqwest::Client,
}

impl LyricsCache {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            client: reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .timeout(Duration::from_secs(8))
                .build()
                .expect("reqwest build"),
        }
    }

    async fn get(&self, key: &str) -> Option<Option<LyricsResult>> {
        self.inner.lock().await.get(key).cloned()
    }

    async fn put(&self, key: String, value: Option<LyricsResult>) {
        self.inner.lock().await.insert(key, value);
    }
}

pub async fn fetch_lyrics(
    cache: &LyricsCache,
    artist: &str,
    title: &str,
    album: &str,
    duration_ms: i64,
) -> Result<Option<LyricsResult>, String> {
    if artist.trim().is_empty() || title.trim().is_empty() {
        return Ok(None);
    }
    let duration_s = (duration_ms / 1000).max(0);

    let exact_key = format!(
        "get|{}|{}|{}|{}",
        artist.to_lowercase(),
        title.to_lowercase(),
        album.to_lowercase(),
        duration_s,
    );
    if let Some(cached) = cache.get(&exact_key).await {
        return Ok(cached);
    }

    let client = &cache.client;
    if let Some(result) = fetch_exact(client, artist, title, album, duration_s).await? {
        cache.put(exact_key, Some(result.clone())).await;
        return Ok(Some(result));
    }

    // Negative-cache the exact-key miss so we don't re-query on every play.
    cache.put(exact_key, None).await;

    let search_key = format!(
        "search|{}|{}",
        artist.to_lowercase(),
        title.to_lowercase(),
    );
    if let Some(cached) = cache.get(&search_key).await {
        return Ok(cached);
    }

    let result = fetch_search(client, artist, title).await?;
    cache.put(search_key, result.clone()).await;
    Ok(result)
}

async fn fetch_exact(
    client: &reqwest::Client,
    artist: &str,
    title: &str,
    album: &str,
    duration_s: i64,
) -> Result<Option<LyricsResult>, String> {
    let url = format!("{BASE_URL}/get");
    let resp = client
        .get(&url)
        .query(&[
            ("artist_name", artist),
            ("track_name", title),
            ("album_name", album),
            ("duration", &duration_s.to_string()),
        ])
        .send()
        .await
        .map_err(|e| format!("lrclib get: {e}"))?;
    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    if !resp.status().is_success() {
        return Err(format!("lrclib get: HTTP {}", resp.status()));
    }
    let body: LrclibResp = resp.json().await.map_err(|e| format!("lrclib get json: {e}"))?;
    Ok(Some(body.into_result()))
}

async fn fetch_search(
    client: &reqwest::Client,
    artist: &str,
    title: &str,
) -> Result<Option<LyricsResult>, String> {
    let url = format!("{BASE_URL}/search");
    let resp = client
        .get(&url)
        .query(&[("track_name", title), ("artist_name", artist)])
        .send()
        .await
        .map_err(|e| format!("lrclib search: {e}"))?;
    if !resp.status().is_success() {
        return Ok(None);
    }
    let arr: Vec<LrclibResp> = resp
        .json()
        .await
        .map_err(|e| format!("lrclib search json: {e}"))?;
    Ok(arr.into_iter().next().map(LrclibResp::into_result))
}
