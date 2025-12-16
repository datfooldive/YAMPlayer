use std::sync::{Arc, Mutex};
use rodio::{Decoder, OutputStreamHandle, Sink};
use std::fs::File;
use std::io::BufReader;
use walkdir::WalkDir;
use once_cell::sync::Lazy;

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

#[tauri::command]
fn index_folder(path: String) -> Result<Vec<MusicFile>, String> {
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

    let state = get_audio_state();
    state.lock().unwrap().tracks = music_files.clone();

    Ok(music_files)
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
