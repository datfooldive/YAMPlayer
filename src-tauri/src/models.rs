use rodio::Sink;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct MusicFile {
    pub path: String,
    pub name: String,
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
}

