use rusqlite::{Connection, Result as SqlResult, params};
use tauri::{AppHandle, Manager};
use crate::models::{MusicFile, IndexedFolder};

pub fn get_db_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let app_data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    std::fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data dir: {}", e))?;
    Ok(app_data_dir.join("music_player.db"))
}

pub fn init_db(app: &AppHandle) -> Result<Connection, String> {
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
            artist TEXT,
            album TEXT,
            title TEXT,
            thumbnail TEXT,
            FOREIGN KEY (folder_id) REFERENCES indexed_folders(id) ON DELETE CASCADE
        )",
        [],
    ).map_err(|e| format!("Failed to create tracks table: {}", e))?;

    conn.execute(
        "ALTER TABLE tracks ADD COLUMN artist TEXT",
        [],
    ).ok();

    conn.execute(
        "ALTER TABLE tracks ADD COLUMN album TEXT",
        [],
    ).ok();

    conn.execute(
        "ALTER TABLE tracks ADD COLUMN title TEXT",
        [],
    ).ok();

    conn.execute(
        "ALTER TABLE tracks ADD COLUMN thumbnail TEXT",
        [],
    ).ok();

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_tracks_folder ON tracks(folder_id)",
        [],
    ).map_err(|e| format!("Failed to create index: {}", e))?;

    Ok(conn)
}

pub fn get_db_connection(app: &AppHandle) -> Result<Connection, String> {
    init_db(app)
}

pub fn save_folder(conn: &Connection, path: &str) -> Result<i64, String> {
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

    Ok(folder_id)
}

pub fn save_tracks(conn: &Connection, folder_id: i64, tracks: &[MusicFile]) -> Result<(), String> {
    conn.execute(
        "DELETE FROM tracks WHERE folder_id = ?1",
        params![folder_id],
    ).map_err(|e| format!("Failed to delete old tracks: {}", e))?;

    let mut stmt = conn.prepare(
        "INSERT INTO tracks (folder_id, path, name, artist, album, title, thumbnail) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
    ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

    for track in tracks {
        stmt.execute(params![
            folder_id,
            track.path,
            track.name,
            track.artist,
            track.album,
            track.title,
            track.thumbnail,
        ])
            .map_err(|e| format!("Failed to insert track: {}", e))?;
    }

    Ok(())
}

pub fn load_tracks(conn: &Connection) -> Result<Vec<MusicFile>, String> {
    let mut stmt = conn.prepare(
        "SELECT path, name, artist, album, title, thumbnail FROM tracks ORDER BY COALESCE(artist, ''), COALESCE(album, ''), COALESCE(title, name)"
    ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let tracks: Vec<MusicFile> = stmt.query_map([], |row| {
        Ok(MusicFile {
            path: row.get(0)?,
            name: row.get(1)?,
            artist: row.get(2)?,
            album: row.get(3)?,
            title: row.get(4)?,
            thumbnail: row.get(5)?,
        })
    })
    .map_err(|e| format!("Failed to query tracks: {}", e))?
    .collect::<SqlResult<Vec<_>>>()
    .map_err(|e| format!("Failed to collect tracks: {}", e))?;

    Ok(tracks)
}

pub fn get_indexed_folders(conn: &Connection) -> Result<Vec<IndexedFolder>, String> {
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

pub fn get_folder_paths(conn: &Connection) -> Result<Vec<String>, String> {
    let mut stmt = conn.prepare("SELECT path FROM indexed_folders")
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let folders: Vec<String> = stmt.query_map([], |row| {
        row.get::<_, String>(0)
    })
    .map_err(|e| format!("Failed to query folders: {}", e))?
    .collect::<SqlResult<Vec<_>>>()
    .map_err(|e| format!("Failed to collect folders: {}", e))?;

    Ok(folders)
}

pub fn get_folder_id(conn: &Connection, folder_path: &str) -> Result<i64, String> {
    let folder_id: i64 = conn.query_row(
        "SELECT id FROM indexed_folders WHERE path = ?1",
        params![folder_path],
        |row| row.get(0),
    ).map_err(|e| format!("Failed to get folder id: {}", e))?;

    Ok(folder_id)
}

pub fn get_track_count(conn: &Connection, folder_id: i64) -> Result<i64, String> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM tracks WHERE folder_id = ?1",
        params![folder_id],
        |row| row.get(0),
    ).map_err(|e| format!("Failed to count tracks: {}", e))?;

    Ok(count)
}

pub fn get_track_paths(conn: &Connection, folder_id: i64) -> Result<Vec<String>, String> {
    let mut stmt = conn.prepare("SELECT path FROM tracks WHERE folder_id = ?1")
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let paths: Vec<String> = stmt.query_map(params![folder_id], |row| {
        row.get::<_, String>(0)
    })
    .map_err(|e| format!("Failed to query tracks: {}", e))?
    .collect::<SqlResult<Vec<_>>>()
    .map_err(|e| format!("Failed to collect tracks: {}", e))?;

    Ok(paths)
}

pub fn remove_folder(conn: &Connection, folder_id: i64) -> Result<(), String> {
    conn.execute(
        "DELETE FROM indexed_folders WHERE id = ?1",
        params![folder_id],
    ).map_err(|e| format!("Failed to remove folder: {}", e))?;

    Ok(())
}
