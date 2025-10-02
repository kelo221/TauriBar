// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use serde::Serialize;
use std::{
    ffi::OsStr,
    fs,
    io::BufReader,
    path::{Path, PathBuf},
    process::Command,
    sync::mpsc,
};
use once_cell::sync::OnceCell;
use rodio::{Decoder, OutputStream, Sink};
use tempfile::TempDir;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(Serialize, Clone, Debug)]
struct FrontendAudioFile {
    id: String,
    track: u32,
    title: String,
    artist: String,
    album: String,
    duration: String,
}

fn find_testfiles_dir() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("../testfiles"),
        PathBuf::from("../../testfiles"),
        PathBuf::from("testfiles"),
    ];
    for dir in candidates {
        if dir.exists() {
            println!("[backend] testfiles dir found at: {}", dir.to_string_lossy());
            return Some(dir);
        }
    }
    eprintln!("[backend] testfiles dir not found in expected locations");
    None
}

fn is_supported_audio(path: &Path) -> bool {
    match path.extension().and_then(OsStr::to_str).map(|s| s.to_lowercase()) {
        Some(ext) => matches!(
            ext.as_str(),
            "wv" | "wav" | "mp3" | "flac" | "ogg" | "m4a" | "aac" | "opus" | "aiff" | "caf"
        ),
        None => false,
    }
}

fn parse_track_and_title(file_stem: &str) -> (u32, String) {
    // Try to parse a leading track number like "01. Title" or "01 - Title" or "01 Title"
    let trimmed = file_stem.trim();
    let mut chars = trimmed.chars().peekable();
    let mut digits = String::new();
    while let Some(c) = chars.peek() {
        if c.is_ascii_digit() {
            digits.push(*c);
            chars.next();
        } else {
            break;
        }
    }
    let mut rest = chars.collect::<String>();
    // Remove common separators after track numbers
    if !digits.is_empty() {
        rest = rest.trim_start_matches(['.', '-', ' ']).trim_start().to_string();
    }
    let track = digits.parse::<u32>().unwrap_or(0);
    let title = if rest.is_empty() { trimmed.to_string() } else { rest };
    (track, title)
}

fn seconds_to_mmss(seconds: f64) -> String {
    if !seconds.is_finite() || seconds <= 0.0 {
        return String::new();
    }
    let total = seconds.round() as i64;
    let minutes = total / 60;
    let secs = total % 60;
    format!("{}:{:02}", minutes, secs)
}

fn ffprobe_duration(path: &Path) -> Option<String> {
    println!("[backend] ffprobe_duration for {}", path.to_string_lossy());
    // Attempt to get duration in seconds via ffprobe if available on PATH
    // Use a simple value-only output for easier parsing
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=nw=1:nk=1",
            path.as_os_str().to_string_lossy().as_ref(),
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        eprintln!("[backend] ffprobe_duration failed for {}", path.to_string_lossy());
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let value = stdout.trim();
    if value.is_empty() {
        return None;
    }
    // Some builds return floats (e.g., 233.123456)
    if let Ok(num) = value.parse::<f64>() {
        return Some(seconds_to_mmss(num));
    }
    None
}

fn ffprobe_tags(path: &Path) -> Option<(String, String)> {
    println!("[backend] ffprobe_tags for {}", path.to_string_lossy());
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format_tags=artist,album:stream_tags=artist,album",
            "-of",
            "json",
            path.as_os_str().to_string_lossy().as_ref(),
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        eprintln!("[backend] ffprobe_tags failed for {}", path.to_string_lossy());
        return None;
    }

    let value: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;

    fn find_case_insensitive_tag(
        map: &serde_json::Map<String, serde_json::Value>,
        key: &str,
    ) -> Option<String> {
        for (k, v) in map.iter() {
            if k.eq_ignore_ascii_case(key) {
                if let Some(s) = v.as_str() {
                    return Some(s.to_string());
                }
            }
        }
        None
    }

    let mut artist = String::new();
    let mut album = String::new();

    if let Some(tags) = value
        .get("format")
        .and_then(|f| f.get("tags"))
        .and_then(|t| t.as_object())
    {
        if let Some(a) = find_case_insensitive_tag(tags, "artist") {
            artist = a;
        }
        if let Some(a) = find_case_insensitive_tag(tags, "album") {
            album = a;
        }
    }

    if artist.is_empty() || album.is_empty() {
        if let Some(streams) = value.get("streams").and_then(|s| s.as_array()) {
            for stream in streams {
                if let Some(tags) = stream.get("tags").and_then(|t| t.as_object()) {
                    if artist.is_empty() {
                        if let Some(a) = find_case_insensitive_tag(tags, "artist") {
                            artist = a;
                        }
                    }
                    if album.is_empty() {
                        if let Some(a) = find_case_insensitive_tag(tags, "album") {
                            album = a;
                        }
                    }
                    if !artist.is_empty() && !album.is_empty() {
                        break;
                    }
                }
            }
        }
    }

    if artist.is_empty() && album.is_empty() {
        None
    } else {
        Some((artist, album))
    }
}

enum PlayerCommand {
    Play(String),
    Toggle,
    Stop,
}

static AUDIO_SENDER: OnceCell<mpsc::Sender<PlayerCommand>> = OnceCell::new();

fn ffmpeg_decode_to_wav(input: &Path) -> Result<(TempDir, PathBuf), String> {
    println!(
        "[backend] ffmpeg_decode_to_wav input={}",
        input.to_string_lossy()
    );
    let dir = tempfile::tempdir().map_err(|e| format!("tempdir error: {}", e))?;
    let output = dir.path().join("decoded.wav");

    // Run ffmpeg to decode to signed 16-bit PCM wav, stereo, 48kHz
    // Windows users might have ffmpeg.exe in PATH; rely on PATH resolution
    let status = Command::new("ffmpeg")
        .args([
            "-y", // overwrite
            "-v",
            "error",
            "-i",
        ])
        .arg(input.as_os_str())
        .args([
            "-f",
            "wav",
            "-ac",
            "2",
            "-ar",
            "48000",
        ])
        .arg(&output)
        .status()
        .map_err(|e| {
            format!(
                "failed to run ffmpeg: {}. Ensure ffmpeg is installed and on PATH.",
                e
            )
        })?;

    if !status.success() {
        eprintln!("[backend] ffmpeg exited with failure for {}", input.to_string_lossy());
        return Err("ffmpeg failed to decode audio".to_string());
    }
    println!(
        "[backend] ffmpeg decode success -> {}",
        output.to_string_lossy()
    );
    Ok((dir, output))
}

fn start_audio_thread() -> mpsc::Sender<PlayerCommand> {
    let (tx, rx) = mpsc::channel::<PlayerCommand>();
    std::thread::spawn(move || {
        println!("[backend] audio thread started");
        let (stream, handle) = match OutputStream::try_default() {
            Ok(v) => v,
            Err(e) => {
                eprintln!("failed to open default audio output: {}", e);
                return;
            }
        };
        // Keep stream alive for the lifetime of the thread
        let _stream = stream;

        let mut current_sink: Option<Sink> = None;
        let mut _current_decoded_dir: Option<TempDir> = None;

        while let Ok(cmd) = rx.recv() {
            match cmd {
                PlayerCommand::Play(path) => {
                    println!("[backend] received Play for {}", path);
                    // Stop current if any
                    if let Some(s) = current_sink.take() {
                        s.stop();
                        println!("[backend] stopped previous sink");
                    }
                    _current_decoded_dir = None;

                    let input_path = PathBuf::from(&path);
                    if let Ok((dir, wav_path)) = ffmpeg_decode_to_wav(&input_path) {
                        let file = match std::fs::File::open(&wav_path) {
                            Ok(f) => f,
                            Err(e) => {
                                eprintln!("failed to open decoded file: {}", e);
                                continue;
                            }
                        };
                        let reader = BufReader::new(file);
                        let source = match Decoder::new(reader) {
                            Ok(s) => s,
                            Err(e) => {
                                eprintln!("failed to decode WAV for playback: {}", e);
                                continue;
                            }
                        };
                        match Sink::try_new(&handle) {
                            Ok(sink) => {
                                sink.append(source);
                                sink.play();
                                current_sink = Some(sink);
                                _current_decoded_dir = Some(dir);
                                println!("[backend] playback started");
                            }
                            Err(e) => {
                                eprintln!("failed to create sink: {}", e);
                            }
                        }
                    } else {
                        eprintln!("ffmpeg failed to decode audio for {}", path);
                    }
                }
                PlayerCommand::Toggle => {
                    println!("[backend] received Toggle");
                    if let Some(sink) = current_sink.as_ref() {
                        if sink.is_paused() {
                            println!("[backend] resuming playback");
                            sink.play();
                        } else {
                            println!("[backend] pausing playback");
                            sink.pause();
                        }
                    } else {
                        println!("[backend] toggle requested but no current sink");
                    }
                }
                PlayerCommand::Stop => {
                    println!("[backend] received Stop");
                    if let Some(s) = current_sink.take() {
                        s.stop();
                        println!("[backend] playback stopped");
                    }
                    _current_decoded_dir = None;
                }
            }
        }
    });
    tx
}

fn audio_sender() -> &'static mpsc::Sender<PlayerCommand> {
    AUDIO_SENDER.get_or_init(|| start_audio_thread())
}

#[tauri::command]
fn play_audio(path: String) -> Result<(), String> {
    println!("[backend] play_audio invoked: {}", path);
    if !Path::new(&path).exists() {
        eprintln!("[backend] file does not exist: {}", path);
        return Err("file does not exist".into());
    }
    audio_sender()
        .send(PlayerCommand::Play(path))
        .map_err(|e| format!("failed to send play command: {}", e))
}

#[tauri::command]
fn toggle_play_pause() -> Result<(), String> {
    println!("[backend] toggle_play_pause invoked");
    audio_sender()
        .send(PlayerCommand::Toggle)
        .map_err(|e| format!("failed to send toggle command: {}", e))
}

#[tauri::command]
fn stop_audio() -> Result<(), String> {
    println!("[backend] stop_audio invoked");
    audio_sender()
        .send(PlayerCommand::Stop)
        .map_err(|e| format!("failed to send stop command: {}", e))
}

#[tauri::command]
fn list_media_files() -> Result<Vec<FrontendAudioFile>, String> {
    println!("[backend] list_media_files invoked");
    let base = find_testfiles_dir().ok_or_else(|| "testfiles directory not found".to_string())?;
    println!("[backend] listing media files under {}", base.to_string_lossy());
    let mut items: Vec<FrontendAudioFile> = Vec::new();

    let entries = fs::read_dir(&base).map_err(|e| format!("failed to read dir: {}", e))?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() && is_supported_audio(&path) {
            let file_stem = path
                .file_stem()
                .and_then(OsStr::to_str)
                .unwrap_or_default()
                .to_string();
            let (track, title) = parse_track_and_title(&file_stem);
            let duration = ffprobe_duration(&path).unwrap_or_default();
            let (artist, album) = ffprobe_tags(&path).unwrap_or((String::new(), String::new()));
            let id = path.to_string_lossy().to_string();
            let item = FrontendAudioFile {
                id,
                track,
                title,
                artist,
                album,
                duration,
            };
            println!("[backend] found media: {}", item.id);
            items.push(item);
        }
    }

    // Sort by track then title for a stable order
    items.sort_by(|a, b| a.track.cmp(&b.track).then_with(|| a.title.cmp(&b.title)));
    println!("[backend] returning {} media items", items.len());
    Ok(items)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    println!("[backend] Tauri run starting");
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            list_media_files,
            play_audio,
            toggle_play_pause,
            stop_audio
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
