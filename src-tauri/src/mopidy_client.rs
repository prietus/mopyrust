use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::models::{Image, LibRef, PlaybackSnapshot, Ref, SearchResult, TlTrack, Track};

#[derive(Clone)]
pub struct Client {
    inner: reqwest::Client,
    base: Arc<String>,
}

#[derive(Debug, Clone)]
pub struct ClientError(pub String);

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(&self.0) }
}
impl std::error::Error for ClientError {}

impl From<reqwest::Error> for ClientError {
    fn from(e: reqwest::Error) -> Self { Self(format!("http: {e}")) }
}
impl From<serde_json::Error> for ClientError {
    fn from(e: serde_json::Error) -> Self { Self(format!("json: {e}")) }
}

#[derive(Serialize)]
struct RpcReq<'a> {
    jsonrpc: &'a str,
    id: u32,
    method: &'a str,
    params: Value,
}

#[derive(Deserialize)]
struct RpcResp {
    #[serde(default)]
    result: Option<Value>,
    #[serde(default)]
    error: Option<Value>,
}

impl Client {
    pub fn new(host: &str, port: u16) -> Self {
        let inner = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("reqwest client build");
        Self { inner, base: Arc::new(format!("http://{host}:{port}")) }
    }

    pub fn rpc_url(&self) -> String { format!("{}/mopidy/rpc", self.base) }

    pub fn image_url(&self, uri: &str) -> String {
        if uri.starts_with("http://") || uri.starts_with("https://") {
            uri.to_string()
        } else if uri.starts_with('/') {
            format!("{}{uri}", self.base)
        } else {
            format!("{}/{uri}", self.base)
        }
    }

    async fn call(&self, method: &str, params: Value) -> Result<Value, ClientError> {
        let body = RpcReq { jsonrpc: "2.0", id: 1, method, params };
        let resp = self
            .inner
            .post(self.rpc_url())
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json::<RpcResp>()
            .await?;
        if let Some(err) = resp.error {
            return Err(ClientError(format!("rpc error from {method}: {err}")));
        }
        Ok(resp.result.unwrap_or(Value::Null))
    }

    pub async fn search(
        &self,
        query: &str,
        uris: Option<Vec<String>>,
    ) -> Result<Vec<SearchResult>, ClientError> {
        let params = json!({ "query": { "any": [query] }, "uris": uris });
        let v = self.call("core.library.search", params).await?;
        Ok(serde_json::from_value(v)?)
    }

    /// Field-scoped search (e.g. {"albumartist": ["X"]}). `exact = true` is
    /// the equivalent of `find` rather than `search`.
    pub async fn search_query(
        &self,
        query: HashMap<String, Vec<String>>,
        uris: Option<Vec<String>>,
        exact: bool,
    ) -> Result<Vec<SearchResult>, ClientError> {
        let params = json!({ "query": query, "uris": uris, "exact": exact });
        let v = self.call("core.library.search", params).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn browse(&self, uri: Option<String>) -> Result<Vec<LibRef>, ClientError> {
        let params = json!({ "uri": uri });
        let v = self.call("core.library.browse", params).await?;
        let raw: Vec<Ref> = serde_json::from_value(v)?;
        Ok(raw.into_iter().map(Ref::into_lib).collect())
    }

    pub async fn lookup(&self, uris: Vec<String>) -> Result<HashMap<String, Vec<Track>>, ClientError> {
        let params = json!({ "uris": uris });
        let v = self.call("core.library.lookup", params).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn get_images(
        &self,
        uris: Vec<String>,
    ) -> Result<HashMap<String, Vec<Image>>, ClientError> {
        let params = json!({ "uris": uris });
        let v = self.call("core.library.get_images", params).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn get_tl_tracks(&self) -> Result<Vec<TlTrack>, ClientError> {
        let v = self.call("core.tracklist.get_tl_tracks", json!({})).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn tracklist_add(
        &self,
        uris: Vec<String>,
        at_position: Option<u32>,
    ) -> Result<Vec<TlTrack>, ClientError> {
        let params = json!({ "uris": uris, "at_position": at_position });
        let v = self.call("core.tracklist.add", params).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn tracklist_clear(&self) -> Result<(), ClientError> {
        self.call("core.tracklist.clear", json!({})).await?;
        Ok(())
    }

    pub async fn tracklist_remove(&self, tlids: Vec<u32>) -> Result<Vec<TlTrack>, ClientError> {
        let params = json!({ "criteria": { "tlid": tlids } });
        let v = self.call("core.tracklist.remove", params).await?;
        Ok(serde_json::from_value(v)?)
    }

    /// Move a contiguous range [start, end) to `to_position` (0-based).
    pub async fn tracklist_move(
        &self,
        start: u32,
        end: u32,
        to_position: u32,
    ) -> Result<(), ClientError> {
        let params = json!({ "start": start, "end": end, "to_position": to_position });
        self.call("core.tracklist.move", params).await?;
        Ok(())
    }

    pub async fn playlists_as_list(&self) -> Result<Vec<LibRef>, ClientError> {
        let v = self.call("core.playlists.as_list", json!({})).await?;
        // Same Ref shape as browse output but `type` is always "playlist".
        let raw: Vec<crate::models::Ref> = serde_json::from_value(v)?;
        Ok(raw.into_iter().map(crate::models::Ref::into_lib).collect())
    }

    pub async fn playlist_lookup(&self, uri: &str) -> Result<Option<crate::models::Playlist>, ClientError> {
        let params = json!({ "uri": uri });
        let v = self.call("core.playlists.lookup", params).await?;
        if v.is_null() { return Ok(None); }
        Ok(serde_json::from_value(v)?)
    }

    pub async fn get_distinct(&self, field: &str) -> Result<Vec<String>, ClientError> {
        let params = json!({ "field": field, "query": {} });
        let v = self.call("core.library.get_distinct", params).await?;
        Ok(serde_json::from_value(v).unwrap_or_default())
    }

    // ── playback ────────────────────────────────────────────────────────

    pub async fn playback_play(&self, tlid: Option<u32>) -> Result<(), ClientError> {
        let params = if let Some(id) = tlid {
            json!({ "tlid": id })
        } else {
            json!({})
        };
        self.call("core.playback.play", params).await?;
        Ok(())
    }

    pub async fn playback_pause(&self) -> Result<(), ClientError> {
        self.call("core.playback.pause", json!({})).await?;
        Ok(())
    }

    pub async fn playback_resume(&self) -> Result<(), ClientError> {
        self.call("core.playback.resume", json!({})).await?;
        Ok(())
    }

    pub async fn playback_stop(&self) -> Result<(), ClientError> {
        self.call("core.playback.stop", json!({})).await?;
        Ok(())
    }

    pub async fn playback_next(&self) -> Result<(), ClientError> {
        self.call("core.playback.next", json!({})).await?;
        Ok(())
    }

    pub async fn playback_previous(&self) -> Result<(), ClientError> {
        self.call("core.playback.previous", json!({})).await?;
        Ok(())
    }

    pub async fn playback_seek(&self, time_position_ms: i64) -> Result<(), ClientError> {
        let params = json!({ "time_position": time_position_ms });
        self.call("core.playback.seek", params).await?;
        Ok(())
    }

    pub async fn set_volume(&self, volume: u32) -> Result<(), ClientError> {
        let v = volume.min(100);
        self.call("core.mixer.set_volume", json!({ "volume": v })).await?;
        Ok(())
    }

    /// Aggregated snapshot. Four parallel JSON-RPC calls.
    pub async fn fetch_playback(&self) -> Result<PlaybackSnapshot, ClientError> {
        let (state, tl_track, pos, vol) = tokio::join!(
            self.call("core.playback.get_state", json!({})),
            self.call("core.playback.get_current_tl_track", json!({})),
            self.call("core.playback.get_time_position", json!({})),
            self.call("core.mixer.get_volume", json!({})),
        );
        let state_str: String = state
            .ok()
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_else(|| "stopped".to_string());
        let (current, current_tlid) = match tl_track {
            Ok(v) if !v.is_null() => {
                let tlid = v.get("tlid").and_then(|x| x.as_u64()).map(|x| x as u32);
                let track: Option<Track> = v.get("track").cloned().and_then(|t| serde_json::from_value(t).ok());
                (track, tlid)
            }
            _ => (None, None),
        };
        let elapsed_ms = pos
            .ok()
            .and_then(|v| serde_json::from_value::<i64>(v).ok())
            .unwrap_or(0);
        let volume = vol
            .ok()
            .and_then(|v| serde_json::from_value::<Option<i32>>(v).ok())
            .flatten()
            .unwrap_or(-1);
        Ok(PlaybackSnapshot {
            state: state_str,
            current,
            current_tlid,
            elapsed_ms,
            volume,
        })
    }

    pub async fn fetch_bytes(&self, url: &str) -> Result<Vec<u8>, ClientError> {
        let bytes = self
            .inner
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;
        Ok(bytes.to_vec())
    }
}
