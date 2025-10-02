// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use serde::Serialize;
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

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
            return Some(dir);
        }
    }
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

#[tauri::command]
fn list_media_files() -> Result<Vec<FrontendAudioFile>, String> {
    let base = find_testfiles_dir().ok_or_else(|| "testfiles directory not found".to_string())?;
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
            items.push(FrontendAudioFile {
                id,
                track,
                title,
                artist,
                album,
                duration,
            });
        }
    }

    // Sort by track then title for a stable order
    items.sort_by(|a, b| a.track.cmp(&b.track).then_with(|| a.title.cmp(&b.title)));
    Ok(items)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, list_media_files])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
