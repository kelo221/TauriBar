// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use once_cell::sync::OnceCell;
use rodio::{OutputStream, Sink, Source};
use serde::Serialize;
use std::{
    ffi::OsStr,
    fs,
    io::{self, BufReader, Read},
    path::{Path, PathBuf},
    process::{Child, ChildStdout, Command, Stdio},
    sync::{
        mpsc::{self},
        Arc, Mutex,
    },
    time::{Duration, Instant},
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
            println!(
                "[backend] testfiles dir found at: {}",
                dir.to_string_lossy()
            );
            return Some(dir);
        }
    }
    eprintln!("[backend] testfiles dir not found in expected locations");
    None
}

fn is_supported_audio(path: &Path) -> bool {
    match path
        .extension()
        .and_then(OsStr::to_str)
        .map(|s| s.to_lowercase())
    {
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
        rest = rest
            .trim_start_matches(['.', '-', ' '])
            .trim_start()
            .to_string();
    }
    let track = digits.parse::<u32>().unwrap_or(0);
    let title = if rest.is_empty() {
        trimmed.to_string()
    } else {
        rest
    };
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
        eprintln!(
            "[backend] ffprobe_duration failed for {}",
            path.to_string_lossy()
        );
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

fn ffprobe_duration_seconds(path: &Path) -> Option<f64> {
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
    value.parse::<f64>().ok()
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
        eprintln!(
            "[backend] ffprobe_tags failed for {}",
            path.to_string_lossy()
        );
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

fn ffprobe_stream_info(path: &Path) -> Option<(String, u32, u32, u32)> {
    // Returns (codec_display, bitrate_kbps, sample_rate_hz, channels)
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "a:0",
            "-show_entries",
            "stream=codec_name,bit_rate,channels,sample_rate",
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
    let streams = value.get("streams")?.as_array()?;
    let s = streams.first()?.as_object()?;
    let codec_name = s.get("codec_name").and_then(|v| v.as_str()).unwrap_or("");
    let bit_rate = s
        .get("bit_rate")
        .and_then(|v| v.as_str())
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);
    let channels = s.get("channels").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let sample_rate = s
        .get("sample_rate")
        .and_then(|v| v.as_str())
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);
    // Pretty codec display
    let codec_display = if codec_name.is_empty() {
        String::new()
    } else {
        let mut c = codec_name.to_string();
        if let Some(first) = c.get_mut(0..1) {
            first.make_ascii_uppercase();
        }
        c
    };
    let kbps = (bit_rate / 1000) as u32;
    Some((codec_display, kbps, sample_rate, channels))
}

enum PlayerCommand {
    Play(String),
    Toggle,
    Stop,
    SetVolume(f32),
    SeekTo(f64),
}

static AUDIO_SENDER: OnceCell<mpsc::Sender<PlayerCommand>> = OnceCell::new();
static PLAYBACK_STATE: OnceCell<Arc<Mutex<PlaybackState>>> = OnceCell::new();

#[derive(Debug, Default)]
struct PlaybackState {
    duration_seconds: Option<f64>,
    started_at: Option<Instant>,
    accumulated_pause: Duration,
    paused_at: Option<Instant>,
    is_playing: bool,
    current_path: Option<String>,
    codec: Option<String>,
    bitrate_kbps: Option<u32>,
    sample_rate_hz: Option<u32>,
    channels: Option<u32>,
    finished: bool,
}

#[derive(Serialize)]
struct FrontendPlaybackState {
    position_seconds: f64,
    duration_seconds: f64,
    is_playing: bool,
    codec: String,
    bitrate_kbps: u32,
    sample_rate_hz: u32,
    channels: u32,
    finished: bool,
}

fn playback_state() -> &'static Arc<Mutex<PlaybackState>> {
    PLAYBACK_STATE.get_or_init(|| Arc::new(Mutex::new(PlaybackState::default())))
}

/// Spawns ffmpeg to decode audio to signed 16-bit PCM (s16le), stereo, 48kHz, streamed to stdout.
fn spawn_ffmpeg_pcm_stream(
    input: &Path,
    start_seconds: Option<f64>,
) -> Result<(Child, ChildStdout), String> {
    println!(
        "[backend] spawn_ffmpeg_pcm_stream input={} start={:?}",
        input.to_string_lossy(),
        start_seconds
    );

    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-v").arg("error");
    if let Some(ss) = start_seconds {
        cmd.arg("-ss").arg(format!("{}", ss));
    }
    cmd.arg("-i").arg(input.as_os_str());
    cmd.arg("-f")
        .arg("s16le")
        .arg("-ac")
        .arg("2")
        .arg("-ar")
        .arg("48000")
        .arg("pipe:1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());

    let mut child = cmd.spawn().map_err(|e| {
        format!(
            "failed to spawn ffmpeg: {}. Ensure ffmpeg is installed and on PATH.",
            e
        )
    })?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "failed to capture ffmpeg stdout".to_string())?;
    Ok((child, stdout))
}

/// A rodio Source that yields i16 samples read from a blocking reader of raw s16le PCM data.
struct PcmStreamSource {
    reader: BufReader<ChildStdout>,
    channels: u16,
    sample_rate: u32,
    buf: Vec<u8>,
    buf_pos: usize,
    ended: bool,
}

impl PcmStreamSource {
    fn new(stdout: ChildStdout, channels: u16, sample_rate: u32) -> Self {
        Self {
            reader: BufReader::new(stdout),
            channels,
            sample_rate,
            buf: Vec::with_capacity(8192),
            buf_pos: 0,
            ended: false,
        }
    }

    fn refill_buffer(&mut self) -> io::Result<()> {
        self.buf.clear();
        self.buf_pos = 0;
        // Read a multiple of 2 bytes to align on i16 sample boundaries
        let mut tmp = [0u8; 8192];
        let n = self.reader.read(&mut tmp)?;
        if n == 0 {
            self.ended = true;
            return Ok(());
        }
        // Ensure even number of bytes for i16
        let even = n & !1;
        self.buf.extend_from_slice(&tmp[..even]);
        Ok(())
    }
}

impl Iterator for PcmStreamSource {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ended {
            return None;
        }
        // If buffer exhausted, try to refill
        if self.buf_pos + 2 > self.buf.len() {
            if let Err(e) = self.refill_buffer() {
                eprintln!("[backend] PCM stream read error: {}", e);
                self.ended = true;
                return None;
            }
            if self.ended {
                return None;
            }
        }
        // Read little-endian i16
        let lo = self.buf[self.buf_pos] as u16;
        let hi = self.buf[self.buf_pos + 1] as u16;
        self.buf_pos += 2;
        let sample = ((hi << 8) | lo) as i16;
        Some(sample)
    }
}

impl Source for PcmStreamSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
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
        let mut current_ffmpeg_child: Option<Child> = None;
        let mut current_input_path: Option<PathBuf> = None;
        let mut current_volume: f32 = 1.0;

        while let Ok(cmd) = rx.recv() {
            match cmd {
                PlayerCommand::Play(path) => {
                    println!("[backend] received Play for {}", path);
                    // Stop current if any
                    if let Some(s) = current_sink.take() {
                        s.stop();
                        println!("[backend] stopped previous sink");
                    }
                    if let Some(mut ch) = current_ffmpeg_child.take() {
                        let _ = ch.kill();
                        let _ = ch.wait();
                        println!("[backend] killed previous ffmpeg child");
                    }

                    let input_path = PathBuf::from(&path);
                    if let Ok((child, stdout)) = spawn_ffmpeg_pcm_stream(&input_path, None) {
                        let source = PcmStreamSource::new(stdout, 2, 48000);
                        match Sink::try_new(&handle) {
                            Ok(sink) => {
                                sink.append(source);
                                sink.play();
                                sink.set_volume(current_volume);
                                current_sink = Some(sink);
                                current_ffmpeg_child = Some(child);
                                current_input_path = Some(input_path);
                                println!("[backend] playback started");
                                // Update playback state
                                {
                                    let mut st = playback_state().lock().unwrap();
                                    st.started_at = Some(Instant::now());
                                    st.accumulated_pause = Duration::from_millis(0);
                                    st.paused_at = None;
                                    st.is_playing = true;
                                    st.current_path = Some(path.clone());
                                }
                                // Gather metadata without holding the lock
                                let meta_duration = ffprobe_duration_seconds(Path::new(&path));
                                let meta_stream = ffprobe_stream_info(Path::new(&path));
                                {
                                    let mut st = playback_state().lock().unwrap();
                                    st.duration_seconds = meta_duration;
                                    if let Some((codec, kbps, sr, ch)) = meta_stream {
                                        st.codec = Some(codec);
                                        st.bitrate_kbps = Some(kbps);
                                        st.sample_rate_hz = Some(sr);
                                        st.channels = Some(ch);
                                    } else {
                                        st.codec = None;
                                        st.bitrate_kbps = None;
                                        st.sample_rate_hz = None;
                                        st.channels = None;
                                    }
                                    st.finished = false;
                                }
                            }
                            Err(e) => {
                                eprintln!("failed to create sink: {}", e);
                            }
                        }
                    } else {
                        eprintln!("ffmpeg failed to start stream for {}", path);
                    }
                }
                PlayerCommand::Toggle => {
                    println!("[backend] received Toggle");
                    if let Some(sink) = current_sink.as_ref() {
                        if sink.is_paused() {
                            println!("[backend] resuming playback");
                            sink.play();
                            let mut st = playback_state().lock().unwrap();
                            if let Some(paused_at) = st.paused_at.take() {
                                st.accumulated_pause += Instant::now() - paused_at;
                            }
                            st.is_playing = true;
                        } else {
                            println!("[backend] pausing playback");
                            sink.pause();
                            let mut st = playback_state().lock().unwrap();
                            st.paused_at = Some(Instant::now());
                            st.is_playing = false;
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
                    if let Some(mut ch) = current_ffmpeg_child.take() {
                        let _ = ch.kill();
                        let _ = ch.wait();
                    }
                    current_input_path = None;
                    let mut st = playback_state().lock().unwrap();
                    st.started_at = None;
                    st.accumulated_pause = Duration::from_millis(0);
                    st.paused_at = None;
                    st.is_playing = false;
                    st.current_path = None;
                    st.duration_seconds = None;
                    st.codec = None;
                    st.bitrate_kbps = None;
                    st.sample_rate_hz = None;
                    st.channels = None;
                    st.finished = true;
                }
                PlayerCommand::SetVolume(vol) => {
                    println!("[backend] received SetVolume {}", vol);
                    current_volume = vol.max(0.0);
                    if let Some(sink) = current_sink.as_ref() {
                        sink.set_volume(current_volume);
                    }
                }
                PlayerCommand::SeekTo(sec) => {
                    println!("[backend] received SeekTo {}s", sec);
                    if let Some(input_path) = current_input_path.as_ref() {
                        let duration_limit = playback_state().lock().unwrap().duration_seconds;
                        let target_sec = if let Some(d) = duration_limit {
                            sec.max(0.0).min(d - 0.001)
                        } else {
                            sec.max(0.0)
                        };
                        if let Some(s) = current_sink.take() {
                            s.stop();
                            println!("[backend] stopped sink before seek");
                        }
                        if let Some(mut ch) = current_ffmpeg_child.take() {
                            let _ = ch.kill();
                            let _ = ch.wait();
                        }
                        if let Ok((child, stdout)) =
                            spawn_ffmpeg_pcm_stream(input_path, Some(target_sec))
                        {
                            let source = PcmStreamSource::new(stdout, 2, 48000);
                            match Sink::try_new(&handle) {
                                Ok(sink) => {
                                    sink.append(source);
                                    sink.play();
                                    sink.set_volume(current_volume);
                                    current_sink = Some(sink);
                                    current_ffmpeg_child = Some(child);
                                    println!("[backend] playback started after seek");
                                    let mut st = playback_state().lock().unwrap();
                                    st.started_at =
                                        Some(Instant::now() - Duration::from_secs_f64(target_sec));
                                    st.accumulated_pause = Duration::from_millis(0);
                                    st.paused_at = None;
                                    st.is_playing = true;
                                    st.finished = false;
                                }
                                Err(e) => {
                                    eprintln!("failed to create sink (seek): {}", e);
                                }
                            }
                        } else {
                            eprintln!("ffmpeg failed to start stream for seek");
                        }
                    } else {
                        println!("[backend] SeekTo requested but no current input path");
                    }
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
    // Update duration and reset timeline state
    {
        let mut st = playback_state().lock().unwrap();
        st.duration_seconds = ffprobe_duration_seconds(Path::new(&path));
        st.started_at = Some(Instant::now());
        st.accumulated_pause = Duration::from_millis(0);
        st.paused_at = None;
        st.is_playing = true;
        st.current_path = Some(path.clone());
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
fn set_volume(volume: f32) -> Result<(), String> {
    println!("[backend] set_volume invoked: {}", volume);
    audio_sender()
        .send(PlayerCommand::SetVolume(volume))
        .map_err(|e| format!("failed to send set_volume command: {}", e))
}

#[tauri::command]
fn seek_to(seconds: f64) -> Result<(), String> {
    println!("[backend] seek_to invoked: {}s", seconds);
    // Clamp on the way in to avoid no-op seeks beyond duration
    let target = {
        let st = playback_state().lock().unwrap();
        if let Some(d) = st.duration_seconds {
            seconds.max(0.0).min((d - 0.001).max(0.0))
        } else {
            seconds.max(0.0)
        }
    };
    audio_sender()
        .send(PlayerCommand::SeekTo(target))
        .map_err(|e| format!("failed to send seek_to command: {}", e))
}

#[tauri::command]
fn get_playback_state() -> Result<FrontendPlaybackState, String> {
    let st = playback_state().lock().unwrap();
    let duration = st.duration_seconds.unwrap_or(0.0);
    let mut position = 0.0;
    if let Some(started) = st.started_at {
        let mut now_pos = (Instant::now() - started).as_secs_f64();
        if !st.is_playing {
            if let Some(paused_at) = st.paused_at {
                now_pos = (paused_at - started).as_secs_f64();
            }
        }
        let paused_total = st.accumulated_pause.as_secs_f64();
        position = (now_pos - paused_total).max(0.0);
    }
    if duration > 0.0 {
        position = position.min(duration);
    }
    Ok(FrontendPlaybackState {
        position_seconds: position,
        duration_seconds: duration,
        is_playing: st.is_playing,
        codec: st.codec.clone().unwrap_or_default(),
        bitrate_kbps: st.bitrate_kbps.unwrap_or(0),
        sample_rate_hz: st.sample_rate_hz.unwrap_or(0),
        channels: st.channels.unwrap_or(0),
        finished: st.finished,
    })
}

#[tauri::command]
fn list_media_files() -> Result<Vec<FrontendAudioFile>, String> {
    println!("[backend] list_media_files invoked");
    let base = find_testfiles_dir().ok_or_else(|| "testfiles directory not found".to_string())?;
    println!(
        "[backend] listing media files under {}",
        base.to_string_lossy()
    );
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

/// Recursively traverse a directory collecting supported audio files
fn collect_media_from_dir_recursive(
    dir: &Path,
    items: &mut Vec<FrontendAudioFile>,
) -> Result<(), String> {
    let mut stack: Vec<PathBuf> = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        let entries = fs::read_dir(&current)
            .map_err(|e| format!("failed to read dir '{}': {}", current.to_string_lossy(), e))?;
        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() && is_supported_audio(&path) {
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
    }
    Ok(())
}

#[tauri::command]
fn list_media_files_from_paths(paths: Vec<String>) -> Result<Vec<FrontendAudioFile>, String> {
    println!(
        "[backend] list_media_files_from_paths invoked with {} path(s)",
        paths.len()
    );
    for p in &paths {
        println!("[backend] path: {}", p);
    }
    let mut items: Vec<FrontendAudioFile> = Vec::new();

    for p in paths {
        let pb = PathBuf::from(p);
        if pb.is_dir() {
            collect_media_from_dir_recursive(&pb, &mut items)?;
        } else if pb.is_file() {
            if is_supported_audio(&pb) {
                let file_stem = pb
                    .file_stem()
                    .and_then(OsStr::to_str)
                    .unwrap_or_default()
                    .to_string();
                let (track, title) = parse_track_and_title(&file_stem);
                let duration = ffprobe_duration(&pb).unwrap_or_default();
                let (artist, album) = ffprobe_tags(&pb).unwrap_or((String::new(), String::new()));
                let id = pb.to_string_lossy().to_string();
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
            } else {
                println!("[backend] skipped non-audio file: {}", pb.to_string_lossy());
            }
        } else {
            println!("[backend] skipped missing path: {}", pb.to_string_lossy());
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
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            greet,
            list_media_files,
            list_media_files_from_paths,
            play_audio,
            toggle_play_pause,
            stop_audio,
            set_volume,
            seek_to,
            get_playback_state
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
