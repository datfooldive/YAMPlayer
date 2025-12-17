use std::collections::HashSet;
use walkdir::WalkDir;
use lofty::read_from_path;
use lofty::file::TaggedFileExt;
use lofty::tag::Accessor;
use crate::models::MusicFile;
use crate::db;

const SUPPORTED_EXTENSIONS: &[&str] = &["mp3", "wav", "flac", "ogg"];

fn extract_metadata(file_path: &str) -> (Option<String>, Option<String>, Option<String>) {
    match read_from_path(file_path) {
        Ok(tagged_file) => {
            let tag = tagged_file.primary_tag();
            let tag = tag.or_else(|| tagged_file.first_tag());
            
            if let Some(tag) = tag {
                let artist = tag.artist().map(|s| s.to_string());
                let album = tag.album().map(|s| s.to_string());
                let title = tag.title().map(|s| s.to_string());
                (artist, album, title)
            } else {
                (None, None, None)
            }
        }
        Err(_) => (None, None, None),
    }
}

pub fn scan_folder(path: &str) -> Vec<MusicFile> {
    let mut music_files = Vec::new();

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                if let Some(ext_str) = ext.to_str() {
                    if SUPPORTED_EXTENSIONS.contains(&ext_str.to_lowercase().as_str()) {
                        let file_path = entry.path().to_string_lossy().to_string();
                        let file_name = entry.file_name().to_string_lossy().to_string();
                        let (artist, album, title) = extract_metadata(&file_path);
                        music_files.push(MusicFile {
                            path: file_path,
                            name: file_name,
                            artist,
                            album,
                            title,
                        });
                    }
                }
            }
        }
    }

    music_files
}

pub fn check_for_changes(app: &tauri::AppHandle) -> Result<bool, String> {
    let conn = db::get_db_connection(app)?;
    let folders = db::get_folder_paths(&conn)?;

    for folder_path in folders {
        let mut current_files = HashSet::new();
        for entry in WalkDir::new(&folder_path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension() {
                    if let Some(ext_str) = ext.to_str() {
                        if SUPPORTED_EXTENSIONS.contains(&ext_str.to_lowercase().as_str()) {
                            current_files.insert(entry.path().to_string_lossy().to_string());
                        }
                    }
                }
            }
        }

        let folder_id = db::get_folder_id(&conn, &folder_path)?;
        let stored_count = db::get_track_count(&conn, folder_id)?;

        if current_files.len() as i64 != stored_count {
            return Ok(true);
        }

        let stored_paths = db::get_track_paths(&conn, folder_id)?;

        for stored_path in stored_paths {
            if !current_files.contains(&stored_path) {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

