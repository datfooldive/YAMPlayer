use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use rodio::{Decoder, OutputStreamHandle, Sink, Source};
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
            volume: 0.5,
            playback_start: None,
            paused_elapsed: Duration::ZERO,
            total_duration: None,
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

    if let Some(sink) = audio_state.sink.take() {
        sink.stop();
    }

    let stream_handle = get_stream_handle()?;

    let file = File::open(&path)
        .map_err(|e| format!("Failed to open file: {}", e))?;
    let source = Decoder::new(BufReader::new(file))
        .map_err(|e| format!("Failed to decode audio: {}", e))?;

    let total_duration = source.total_duration();
    let volume = audio_state.volume;

    let sink = Sink::try_new(stream_handle)
        .map_err(|e| format!("Failed to create sink: {}", e))?;

    sink.set_volume(volume);
    sink.append(source);
    sink.play();

    audio_state.sink = Some(sink);
    audio_state.current_track = Some(path);
    audio_state.playback_start = Some(Instant::now());
    audio_state.paused_elapsed = Duration::ZERO;
    audio_state.total_duration = total_duration;

    Ok(())
}

pub fn seek(position_secs: f64) -> Result<(), String> {
    let state = get_audio_state();
    let mut audio_state = state.lock().unwrap();

    if let Some(path) = audio_state.current_track.clone() {
        let was_playing = audio_state.sink.as_ref().map_or(false, |s| !s.is_paused());

        if let Some(sink) = audio_state.sink.take() {
            sink.stop();
        }

        let stream_handle = get_stream_handle()?;
        let file = File::open(&path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        let mut source = Decoder::new(BufReader::new(file))
            .map_err(|e| format!("Failed to decode audio: {}", e))?;

        let seek_duration = Duration::from_secs_f64(position_secs);
        if source.try_seek(seek_duration).is_err() {
            // if seek fails, we effectively restart the track.
        }

        let total_duration = source.total_duration();
        let volume = audio_state.volume;

        let sink = Sink::try_new(stream_handle)
            .map_err(|e| format!("Failed to create sink: {}", e))?;

        sink.set_volume(volume);
        sink.append(source);

        if was_playing {
            sink.play();
            audio_state.playback_start = Some(Instant::now());
        } else {
            audio_state.playback_start = None;
        }
        
        audio_state.sink = Some(sink);
        audio_state.paused_elapsed = seek_duration;
        audio_state.total_duration = total_duration;
    }

    Ok(())
}

pub fn pause_music() -> Result<(), String> {
    let state = get_audio_state();
    let mut audio_state = state.lock().unwrap();
    if let Some(sink) = &audio_state.sink {
        sink.pause();
        if let Some(start) = audio_state.playback_start {
            audio_state.paused_elapsed += start.elapsed();
            audio_state.playback_start = None;
        }
    }
    Ok(())
}

pub fn resume_music() -> Result<(), String> {
    let state = get_audio_state();
    let mut audio_state = state.lock().unwrap();
    if let Some(sink) = &audio_state.sink {
        if sink.empty() {
            if let Some(path) = audio_state.current_track.clone() {
                drop(audio_state);
                return play_music(path);
            }
        } else {
            sink.play();
            audio_state.playback_start = Some(Instant::now());
        }
    }
    Ok(())
}

pub fn stop_music() -> Result<(), String> {
    let state = get_audio_state();
    let mut audio_state = state.lock().unwrap();
    if let Some(sink) = &audio_state.sink {
        sink.stop();
    }
    audio_state.playback_start = None;
    audio_state.paused_elapsed = Duration::ZERO;
    Ok(())
}

pub fn get_current_track() -> Result<Option<String>, String> {
    let state = get_audio_state();
    let audio_state = state.lock().unwrap();
    Ok(audio_state.current_track.clone())
}

pub fn get_current_track_info() -> Result<Option<MusicFile>, String> {
    let state = get_audio_state();
    let audio_state = state.lock().unwrap();

    if let Some(current_path) = &audio_state.current_track {
        if let Some(track) = audio_state.tracks.iter().find(|t| &t.path == current_path) {
            return Ok(Some(track.clone()));
        }
    }

    Ok(None)
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

pub fn set_volume(volume: f32) -> Result<(), String> {
    let state = get_audio_state();
    let mut audio_state = state.lock().unwrap();
    audio_state.volume = volume;
    if let Some(sink) = &audio_state.sink {
        sink.set_volume(volume);
    }
    Ok(())
}

pub fn get_volume() -> Result<f32, String> {
    let state = get_audio_state();
    let audio_state = state.lock().unwrap();
    Ok(audio_state.volume)
}

pub fn get_playback_position() -> Result<(f64, Option<f64>), String> {
    let state = get_audio_state();
    let mut audio_state = state.lock().unwrap();

    let mut elapsed = audio_state.paused_elapsed.as_secs_f64();
    if let Some(start) = audio_state.playback_start {
        elapsed += start.elapsed().as_secs_f64();
    }

    let total = audio_state.total_duration.map(|d| d.as_secs_f64());

    let mut next_track_path: Option<String> = None;

    if let Some(sink) = &audio_state.sink {
        if sink.empty() {
            if let Some(current_path) = &audio_state.current_track {
                if audio_state.playback_start.is_some() {
                    let tracks = &audio_state.tracks;
                    if let Some(current_index) = tracks.iter().position(|t| t.path == *current_path) {
                        if current_index < tracks.len() - 1 {
                            next_track_path = Some(tracks[current_index + 1].path.clone());
                            audio_state.playback_start = None;
                        }
                    }
                }
            }
        }
    }

    if let Some(path) = next_track_path {
        drop(audio_state);
        let _ = play_music(path);
    }

    Ok((elapsed, total))
}

pub fn play_next() -> Result<(), String> {
    let state = get_audio_state();
    let audio_state = state.lock().unwrap();

    if let Some(current_path) = audio_state.current_track.clone() {
        let tracks = audio_state.tracks.clone();
        if let Some(current_index) = tracks.iter().position(|t| t.path == current_path) {
            if current_index < tracks.len() - 1 {
                let next_track_path = tracks[current_index + 1].path.clone();
                drop(audio_state);
                play_music(next_track_path)?;
            }
        }
    }

    Ok(())
}

pub fn play_previous() -> Result<(), String> {
    let state = get_audio_state();
    let audio_state = state.lock().unwrap();

    if let Some(current_path) = audio_state.current_track.clone() {
        let tracks = audio_state.tracks.clone();
        if let Some(current_index) = tracks.iter().position(|t| t.path == current_path) {
            if current_index > 0 {
                let prev_track_path = tracks[current_index - 1].path.clone();
                drop(audio_state);
                play_music(prev_track_path)?;
            }
        }
    }

    Ok(())
}
