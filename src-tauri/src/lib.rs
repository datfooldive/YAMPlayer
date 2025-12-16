use std::sync::{Arc, Mutex};
use std::collections::HashSet;
use rodio::{Decoder, OutputStreamHandle, Sink};
use std::fs::File;
use std::io::BufReader;
use walkdir::WalkDir;
use once_cell::sync::Lazy;
use rusqlite::{Connection, Result as SqlResult, params};
use tauri::{AppHandle, Manager};

static STREAM_HANDLE: Lazy<Mutex<Option<&'static OutputStreamHandle>>> = Lazy::new(|| Mutex::new(None));

fn get_stream_handle() -> Result<&'static OutputStreamHandle, String> {
    let mut handle_opt = STREAM_HANDLE.lock().unwrap();
    if handle_opt.is_none() {
        let (stream, handle) = rodio::OutputStream::try_default()
            .map_err(|e| format!("Failed to create output stream: {}", e))?;
        Box::leak(Box::new(stream));
        let handle_ref = Box::leak(Box::new(handle));
        *handle_opt = Some(handle_ref);
        Ok(handle_ref)
    } else {
        Ok(handle_opt.unwrap())
    }
}

struct AudioState {
    sink: Option<Sink>,
    current_track: Option<String>,
    tracks: Vec<MusicFile>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct MusicFile {
    path: String,
    name: String,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct IndexedFolder {
    id: i64,
    path: String,
    last_indexed: String,
}

static AUDIO_STATE: Mutex<Option<Arc<Mutex<AudioState>>>> = Mutex::new(None);

fn get_audio_state() -> Arc<Mutex<AudioState>> {
    let mut state = AUDIO_STATE.lock().unwrap();
    if state.is_none() {
        let audio_state = Arc::new(Mutex::new(AudioState {
            sink: None,
            current_track: None,
            tracks: Vec::new(),
        }));
        *state = Some(audio_state.clone());
        audio_state
    } else {
        state.as_ref().unwrap().clone()
    }
}

fn get_db_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let app_data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    std::fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data dir: {}", e))?;
    Ok(app_data_dir.join("music_player.db"))
}

fn init_db(app: &AppHandle) -> Result<Connection, String> {
    let db_path = get_db_path(app)?;
    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS indexed_folders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT UNIQUE NOT NULL,
            last_indexed TEXT NOT NULL
        )",
        [],
    ).map_err(|e| format!("Failed to create indexed_folders table: {}", e))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS tracks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            folder_id INTEGER NOT NULL,
            path TEXT UNIQUE NOT NULL,
            name TEXT NOT NULL,
            FOREIGN KEY (folder_id) REFERENCES indexed_folders(id) ON DELETE CASCADE
        )",
        [],
    ).map_err(|e| format!("Failed to create tracks table: {}", e))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_tracks_folder ON tracks(folder_id)",
        [],
    ).map_err(|e| format!("Failed to create index: {}", e))?;

    Ok(conn)
}

fn get_db_connection(app: &AppHandle) -> Result<Connection, String> {
    init_db(app)
}

#[tauri::command]
fn index_folder(path: String, app: AppHandle) -> Result<Vec<MusicFile>, String> {
    let mut music_files = Vec::new();
    let supported_extensions = ["mp3", "wav", "flac", "ogg"];

    for entry in WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                if let Some(ext_str) = ext.to_str() {
                    if supported_extensions.contains(&ext_str.to_lowercase().as_str()) {
                        let file_path = entry.path().to_string_lossy().to_string();
                        let file_name = entry.file_name().to_string_lossy().to_string();
                        music_files.push(MusicFile {
                            path: file_path,
                            name: file_name,
                        });
                    }
                }
            }
        }
    }

    let conn = get_db_connection(&app)?;
    
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO indexed_folders (path, last_indexed) VALUES (?1, ?2)
         ON CONFLICT(path) DO UPDATE SET last_indexed = ?2",
        params![path, now],
    ).map_err(|e| format!("Failed to save folder: {}", e))?;

    let folder_id: i64 = conn.query_row(
        "SELECT id FROM indexed_folders WHERE path = ?1",
        params![path],
        |row| row.get(0),
    ).map_err(|e| format!("Failed to get folder id: {}", e))?;

    conn.execute(
        "DELETE FROM tracks WHERE folder_id = ?1",
        params![folder_id],
    ).map_err(|e| format!("Failed to delete old tracks: {}", e))?;

    let mut stmt = conn.prepare(
        "INSERT INTO tracks (folder_id, path, name) VALUES (?1, ?2, ?3)"
    ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

    for track in &music_files {
        stmt.execute(params![folder_id, track.path, track.name])
            .map_err(|e| format!("Failed to insert track: {}", e))?;
    }

    let state = get_audio_state();
    state.lock().unwrap().tracks = music_files.clone();

    Ok(music_files)
}

#[tauri::command]
fn load_from_db(app: AppHandle) -> Result<Vec<MusicFile>, String> {
    let conn = get_db_connection(&app)?;
    
    let mut stmt = conn.prepare(
        "SELECT path, name FROM tracks ORDER BY name"
    ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let tracks: Vec<MusicFile> = stmt.query_map([], |row| {
        Ok(MusicFile {
            path: row.get(0)?,
            name: row.get(1)?,
        })
    })
    .map_err(|e| format!("Failed to query tracks: {}", e))?
    .collect::<SqlResult<Vec<_>>>()
    .map_err(|e| format!("Failed to collect tracks: {}", e))?;

    let state = get_audio_state();
    state.lock().unwrap().tracks = tracks.clone();

    Ok(tracks)
}

#[tauri::command]
fn get_indexed_folders(app: AppHandle) -> Result<Vec<IndexedFolder>, String> {
    let conn = get_db_connection(&app)?;
    
    let mut stmt = conn.prepare(
        "SELECT id, path, last_indexed FROM indexed_folders ORDER BY last_indexed DESC"
    ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let folders: Vec<IndexedFolder> = stmt.query_map([], |row| {
        Ok(IndexedFolder {
            id: row.get(0)?,
            path: row.get(1)?,
            last_indexed: row.get(2)?,
        })
    })
    .map_err(|e| format!("Failed to query folders: {}", e))?
    .collect::<SqlResult<Vec<_>>>()
    .map_err(|e| format!("Failed to collect folders: {}", e))?;

    Ok(folders)
}

#[tauri::command]
fn check_for_changes(app: AppHandle) -> Result<bool, String> {
    let conn = get_db_connection(&app)?;
    
    let mut stmt = conn.prepare("SELECT path FROM indexed_folders")
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;
    
    let folders: Vec<String> = stmt.query_map([], |row| {
        row.get::<_, String>(0)
    })
    .map_err(|e| format!("Failed to query folders: {}", e))?
    .collect::<SqlResult<Vec<_>>>()
    .map_err(|e| format!("Failed to collect folders: {}", e))?;

    let supported_extensions = ["mp3", "wav", "flac", "ogg"];
    
    for folder_path in folders {
        let mut current_files = HashSet::new();
        for entry in WalkDir::new(&folder_path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension() {
                    if let Some(ext_str) = ext.to_str() {
                        if supported_extensions.contains(&ext_str.to_lowercase().as_str()) {
                            current_files.insert(entry.path().to_string_lossy().to_string());
                        }
                    }
                }
            }
        }

        let folder_id: i64 = conn.query_row(
            "SELECT id FROM indexed_folders WHERE path = ?1",
            params![folder_path],
            |row| row.get(0),
        ).map_err(|e| format!("Failed to get folder id: {}", e))?;

        let stored_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM tracks WHERE folder_id = ?1",
            params![folder_id],
            |row| row.get(0),
        ).map_err(|e| format!("Failed to count tracks: {}", e))?;

        if current_files.len() as i64 != stored_count {
            return Ok(true);
        }

        let mut stmt2 = conn.prepare("SELECT path FROM tracks WHERE folder_id = ?1")
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;
        
        let stored_paths: Vec<String> = stmt2.query_map(params![folder_id], |row| {
            row.get::<_, String>(0)
        })
        .map_err(|e| format!("Failed to query tracks: {}", e))?
        .collect::<SqlResult<Vec<_>>>()
        .map_err(|e| format!("Failed to collect tracks: {}", e))?;

        for stored_path in stored_paths {
            if !current_files.contains(&stored_path) {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

#[tauri::command]
fn remove_folder(folder_id: i64, app: AppHandle) -> Result<(), String> {
    let conn = get_db_connection(&app)?;
    
    conn.execute(
        "DELETE FROM indexed_folders WHERE id = ?1",
        params![folder_id],
    ).map_err(|e| format!("Failed to remove folder: {}", e))?;

    let state = get_audio_state();
    let mut stmt = conn.prepare("SELECT path, name FROM tracks ORDER BY name")
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;
    
    let tracks: Vec<MusicFile> = stmt.query_map([], |row| {
        Ok(MusicFile {
            path: row.get(0)?,
            name: row.get(1)?,
        })
    })
    .map_err(|e| format!("Failed to query tracks: {}", e))?
    .collect::<SqlResult<Vec<_>>>()
    .map_err(|e| format!("Failed to collect tracks: {}", e))?;

    state.lock().unwrap().tracks = tracks;

    Ok(())
}

#[tauri::command]
fn list_music() -> Result<Vec<MusicFile>, String> {
    let state = get_audio_state();
    let tracks = state.lock().unwrap().tracks.clone();
    Ok(tracks)
}

#[tauri::command]
fn play_music(path: String) -> Result<(), String> {
    let state = get_audio_state();
    let mut audio_state = state.lock().unwrap();

    if let Some(sink) = &audio_state.sink {
        sink.stop();
    }

    let stream_handle = get_stream_handle()?;

    let file = File::open(&path)
        .map_err(|e| format!("Failed to open file: {}", e))?;

    let source = Decoder::new(BufReader::new(file))
        .map_err(|e| format!("Failed to decode audio: {}", e))?;

    let sink = Sink::try_new(stream_handle)
        .map_err(|e| format!("Failed to create sink: {}", e))?;

    sink.append(source);
    sink.play();

    audio_state.sink = Some(sink);
    audio_state.current_track = Some(path);

    Ok(())
}

#[tauri::command]
fn pause_music() -> Result<(), String> {
    let state = get_audio_state();
    let audio_state = state.lock().unwrap();
    if let Some(sink) = &audio_state.sink {
        sink.pause();
    }
    Ok(())
}

#[tauri::command]
fn resume_music() -> Result<(), String> {
    let state = get_audio_state();
    let audio_state = state.lock().unwrap();
    if let Some(sink) = &audio_state.sink {
        sink.play();
    }
    Ok(())
}

#[tauri::command]
fn stop_music() -> Result<(), String> {
    let state = get_audio_state();
    let audio_state = state.lock().unwrap();
    if let Some(sink) = &audio_state.sink {
        sink.stop();
    }
    Ok(())
}

#[tauri::command]
fn get_current_track() -> Result<Option<String>, String> {
    let state = get_audio_state();
    let audio_state = state.lock().unwrap();
    Ok(audio_state.current_track.clone())
}

#[tauri::command]
fn is_playing() -> Result<bool, String> {
    let state = get_audio_state();
    let audio_state = state.lock().unwrap();
    if let Some(sink) = &audio_state.sink {
        Ok(!sink.is_paused() && sink.len() > 0)
    } else {
        Ok(false)
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            index_folder,
            list_music,
            load_from_db,
            get_indexed_folders,
            check_for_changes,
            remove_folder,
            play_music,
            pause_music,
            resume_music,
            stop_music,
            get_current_track,
            is_playing
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
