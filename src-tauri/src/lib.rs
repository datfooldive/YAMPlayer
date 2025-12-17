mod models;
mod db;
mod audio;
mod indexing;

use tauri::AppHandle;
use crate::models::MusicFile;
use crate::models::IndexedFolder;

#[tauri::command]
fn index_folder(path: String, app: AppHandle) -> Result<Vec<MusicFile>, String> {
    let music_files = indexing::scan_folder(&path);
    
    let conn = db::get_db_connection(&app)?;
    let folder_id = db::save_folder(&conn, &path)?;
    db::save_tracks(&conn, folder_id, &music_files)?;
    
    audio::set_tracks(music_files.clone());
    
    Ok(music_files)
}

#[tauri::command]
fn load_from_db(app: AppHandle) -> Result<Vec<MusicFile>, String> {
    let conn = db::get_db_connection(&app)?;
    let tracks = db::load_tracks(&conn)?;
    audio::set_tracks(tracks.clone());
    Ok(tracks)
}

#[tauri::command]
fn get_indexed_folders(app: AppHandle) -> Result<Vec<IndexedFolder>, String> {
    let conn = db::get_db_connection(&app)?;
    db::get_indexed_folders(&conn)
}

#[tauri::command]
fn check_for_changes(app: AppHandle) -> Result<bool, String> {
    indexing::check_for_changes(&app)
}

#[tauri::command]
fn remove_folder(folder_id: i64, app: AppHandle) -> Result<(), String> {
    let conn = db::get_db_connection(&app)?;
    db::remove_folder(&conn, folder_id)?;
    
    let tracks = db::load_tracks(&conn)?;
    audio::set_tracks(tracks);
    
    Ok(())
}

#[tauri::command]
fn list_music() -> Result<Vec<MusicFile>, String> {
    audio::list_music()
}

#[tauri::command]
fn play_music(path: String) -> Result<(), String> {
    audio::play_music(path)
}

#[tauri::command]
fn pause_music() -> Result<(), String> {
    audio::pause_music()
}

#[tauri::command]
fn resume_music() -> Result<(), String> {
    audio::resume_music()
}

#[tauri::command]
fn stop_music() -> Result<(), String> {
    audio::stop_music()
}

#[tauri::command]
fn get_current_track() -> Result<Option<String>, String> {
    audio::get_current_track()
}

#[tauri::command]
fn get_current_track_info() -> Result<Option<MusicFile>, String> {
    audio::get_current_track_info()
}

#[tauri::command]
fn is_playing() -> Result<bool, String> {
    audio::is_playing()
}

#[tauri::command]
fn set_volume(volume: f32) -> Result<(), String> {
    audio::set_volume(volume)
}

#[tauri::command]
fn get_volume() -> Result<f32, String> {
    audio::get_volume()
}

#[tauri::command]
fn get_playback_position() -> Result<(f64, Option<f64>), String> {
    audio::get_playback_position()
}

#[tauri::command]
fn seek(position_secs: f64) -> Result<(), String> {
    audio::seek(position_secs)
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
            get_current_track_info,
            is_playing,
            set_volume,
            get_volume,
            get_playback_position,
            seek
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
