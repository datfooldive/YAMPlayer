use std::sync::{Arc, Mutex};
use rodio::{Decoder, OutputStreamHandle, Sink};
use std::fs::File;
use std::io::BufReader;
use once_cell::sync::Lazy;
use crate::models::{AudioState, MusicFile};

static STREAM_HANDLE: Lazy<Mutex<Option<&'static OutputStreamHandle>>> = Lazy::new(|| Mutex::new(None));

pub fn get_stream_handle() -> Result<&'static OutputStreamHandle, String> {
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

static AUDIO_STATE: Mutex<Option<Arc<Mutex<AudioState>>>> = Mutex::new(None);

pub fn get_audio_state() -> Arc<Mutex<AudioState>> {
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

pub fn set_tracks(tracks: Vec<MusicFile>) {
    let state = get_audio_state();
    state.lock().unwrap().tracks = tracks;
}

pub fn play_music(path: String) -> Result<(), String> {
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

pub fn pause_music() -> Result<(), String> {
    let state = get_audio_state();
    let audio_state = state.lock().unwrap();
    if let Some(sink) = &audio_state.sink {
        sink.pause();
    }
    Ok(())
}

pub fn resume_music() -> Result<(), String> {
    let state = get_audio_state();
    let audio_state = state.lock().unwrap();
    if let Some(sink) = &audio_state.sink {
        sink.play();
    }
    Ok(())
}

pub fn stop_music() -> Result<(), String> {
    let state = get_audio_state();
    let audio_state = state.lock().unwrap();
    if let Some(sink) = &audio_state.sink {
        sink.stop();
    }
    Ok(())
}

pub fn get_current_track() -> Result<Option<String>, String> {
    let state = get_audio_state();
    let audio_state = state.lock().unwrap();
    Ok(audio_state.current_track.clone())
}

pub fn is_playing() -> Result<bool, String> {
    let state = get_audio_state();
    let audio_state = state.lock().unwrap();
    if let Some(sink) = &audio_state.sink {
        Ok(!sink.is_paused() && sink.len() > 0)
    } else {
        Ok(false)
    }
}

pub fn list_music() -> Result<Vec<MusicFile>, String> {
    let state = get_audio_state();
    let tracks = state.lock().unwrap().tracks.clone();
    Ok(tracks)
}

