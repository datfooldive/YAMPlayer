use rodio::Sink;
use std::time::{Duration, Instant};

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct MusicFile {
    pub path: String,
    pub name: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub title: Option<String>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct IndexedFolder {
    pub id: i64,
    pub path: String,
    pub last_indexed: String,
}

pub struct AudioState {
    pub sink: Option<Sink>,
    pub current_track: Option<String>,
    pub tracks: Vec<MusicFile>,
    pub volume: f32,
    pub playback_start: Option<Instant>,
    pub paused_elapsed: Duration,
    pub total_duration: Option<Duration>,
}

