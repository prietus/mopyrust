use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Artist {
    #[serde(default)]
    pub uri: Option<String>,
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Album {
    #[serde(default)]
    pub uri: Option<String>,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub artists: Vec<Artist>,
    #[serde(default)]
    pub num_tracks: Option<u32>,
    #[serde(default)]
    pub date: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Track {
    pub uri: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub artists: Vec<Artist>,
    #[serde(default)]
    pub album: Option<Album>,
    #[serde(default)]
    pub length: Option<u64>,
    #[serde(default)]
    pub track_no: Option<u32>,
    #[serde(default)]
    pub disc_no: Option<u32>,
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default)]
    pub bitrate: Option<u32>,
    #[serde(default)]
    pub genre: Option<String>,
    #[serde(default)]
    pub composers: Vec<Artist>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "__model__")]
pub enum Ref {
    #[serde(rename = "Ref")]
    Ref {
        #[serde(rename = "type")]
        kind: String,
        uri: String,
        #[serde(default)]
        name: String,
    },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LibRef {
    pub kind: String,
    pub uri: String,
    pub name: String,
}

impl Ref {
    pub fn into_lib(self) -> LibRef {
        match self {
            Ref::Ref { kind, uri, name } => LibRef { kind, uri, name },
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Image {
    pub uri: String,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchResult {
    #[serde(default)]
    pub uri: Option<String>,
    #[serde(default)]
    pub tracks: Vec<Track>,
    #[serde(default)]
    pub albums: Vec<Album>,
    #[serde(default)]
    pub artists: Vec<Artist>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TlTrack {
    pub tlid: u32,
    pub track: Track,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Playlist {
    pub uri: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub tracks: Vec<Track>,
    #[serde(default)]
    pub last_modified: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AudioFormat {
    pub rate: u32,
    pub bits: u32,
    pub channels: u32,
}

impl AudioFormat {
    pub fn parse(s: &str) -> Option<Self> {
        let mut it = s.split(':');
        let rate = it.next()?.parse().ok()?;
        let bits_s = it.next()?;
        let bits = if bits_s == "f" { 32 } else { bits_s.parse().ok()? };
        let channels = it.next()?.parse().ok()?;
        Some(Self { rate, bits, channels })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumCard {
    pub uri: String,
    pub name: String,
    pub backend: String,
    #[serde(default)]
    pub artist: String,
    #[serde(default)]
    pub year: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlaybackSnapshot {
    pub state: String,                  // "playing" | "paused" | "stopped"
    pub current: Option<Track>,
    pub current_tlid: Option<u32>,
    pub elapsed_ms: i64,
    pub volume: i32,
}
