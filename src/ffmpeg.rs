//! ffmpeg runner. the web app uses libav.js in a worker; on the desktop we run a
//! real ffmpeg with exactly the same arguments (`-progress` parsing included).

use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};

#[derive(Clone, Debug, Default)]
pub struct FfProgress {
    pub out_time_secs: Option<f64>,
    pub total_size: Option<u64>,
    pub ended: bool,
}

/// finds an ffmpeg binary: explicit setting -> next to the executable -> PATH
pub fn find_ffmpeg(custom: &str) -> Option<PathBuf> {
    if !custom.trim().is_empty() {
        let p = PathBuf::from(custom.trim());
        if p.is_file() {
            return Some(p);
        }
        return None;
    }
    let exe_name = if cfg!(windows) { "ffmpeg.exe" } else { "ffmpeg" };
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let candidate = dir.join(exe_name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    if let Ok(path) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path) {
            let candidate = dir.join(exe_name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn command(ffmpeg: &Path) -> tokio::process::Command {
    let mut cmd = tokio::process::Command::new(ffmpeg);
    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd.kill_on_drop(true);
    cmd
}

/// parses "Duration: 00:03:33.12" from `ffmpeg -i file` output
pub async fn probe_duration(ffmpeg: &Path, file: &Path) -> Option<f64> {
    let mut cmd = command(ffmpeg);
    cmd.args(["-hide_banner", "-nostdin", "-i"])
        .arg(file)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    let out = cmd.output().await.ok()?;
    let text = String::from_utf8_lossy(&out.stderr);
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("Duration:") {
            let ts = rest.trim().split(',').next()?.trim();
            return parse_timestamp(ts);
        }
    }
    None
}

fn parse_timestamp(ts: &str) -> Option<f64> {
    let parts: Vec<&str> = ts.split(':').collect();
    if parts.len() != 3 {
        return None;
    }
    let h: f64 = parts[0].parse().ok()?;
    let m: f64 = parts[1].parse().ok()?;
    let s: f64 = parts[2].parse().ok()?;
    Some(h * 3600.0 + m * 60.0 + s)
}

/// a probe like the web app's `probe()`: checks the file has streams at all
pub async fn probe_streams(ffmpeg: &Path, file: &Path) -> Result<(bool, bool), String> {
    let mut cmd = command(ffmpeg);
    cmd.args(["-hide_banner", "-nostdin", "-i"])
        .arg(file)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    let out = cmd.output().await.map_err(|e| e.to_string())?;
    let text = String::from_utf8_lossy(&out.stderr);
    if text.contains("Invalid data found") || text.contains("Unknown format") {
        return Err("queue.ffmpeg.probe_failed".into());
    }
    let has_video = text.lines().any(|l| l.contains("Stream #") && l.contains("Video:"));
    let has_audio = text.lines().any(|l| l.contains("Stream #") && l.contains("Audio:"));
    Ok((has_video, has_audio))
}

/// `LibAVWrapper.render()` equivalent:
/// ffmpeg -nostdin -y -loglevel error -progress pipe:1 -threads N [-i in]... [args]... output
pub async fn render(
    ffmpeg: &Path,
    inputs: &[PathBuf],
    args: &[String],
    output: &Path,
    duration_hint: Option<f64>,
    mut on_progress: impl FnMut(f32, u64),
) -> Result<(), String> {
    let threads = std::thread::available_parallelism().map(|n| n.get().min(4)).unwrap_or(2);
    let mut cmd = command(ffmpeg);
    cmd.args(["-nostdin", "-y", "-loglevel", "error", "-progress", "pipe:1", "-threads", &threads.to_string()]);
    for input in inputs {
        cmd.arg("-i").arg(input);
    }
    cmd.args(args);
    cmd.arg(output);
    cmd.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|_| "ffmpeg.crashed".to_string())?;
    let stdout = child.stdout.take().ok_or("ffmpeg.crashed")?;
    let stderr = child.stderr.take().ok_or("ffmpeg.crashed")?;

    let stderr_task = tokio::spawn(async move {
        let mut text = String::new();
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            text.push_str(&line);
            text.push('\n');
        }
        text
    });

    let mut lines = BufReader::new(stdout).lines();
    let mut current = FfProgress::default();
    while let Ok(Some(line)) = lines.next_line().await {
        if let Some((k, v)) = line.split_once('=') {
            match k.trim() {
                "out_time_ms" | "out_time_us" => {
                    if let Ok(us) = v.trim().parse::<i64>() {
                        current.out_time_secs = Some(us.max(0) as f64 / 1_000_000.0);
                    }
                }
                "out_time" => {
                    if current.out_time_secs.is_none() {
                        current.out_time_secs = parse_timestamp(v.trim());
                    }
                }
                "total_size" => {
                    current.total_size = v.trim().parse().ok();
                }
                "progress" => {
                    let ended = v.trim() == "end";
                    current.ended = ended;
                    let pct = match (current.out_time_secs, duration_hint) {
                        (Some(t), Some(d)) if d > 0.0 => ((t / d) * 100.0).clamp(0.0, 100.0) as f32,
                        _ => 0.0,
                    };
                    on_progress(if ended { 100.0 } else { pct }, current.total_size.unwrap_or(0));
                }
                _ => {}
            }
        }
    }

    let status = child.wait().await.map_err(|_| "ffmpeg.crashed".to_string())?;
    let err_text = stderr_task.await.unwrap_or_default();
    if !status.success() {
        let lower = err_text.to_lowercase();
        if lower.contains("does not contain any stream") || lower.contains("invalid data found") {
            return Err("ffmpeg.no_input_format".into());
        }
        if lower.contains("output file #0 does not contain any stream") || lower.contains("no audio") {
            return Err("ffmpeg.no_audio_channel".into());
        }
        if lower.contains("cannot allocate memory") {
            return Err("ffmpeg.out_of_memory".into());
        }
        eprintln!("ffmpeg failed:\n{err_text}");
        return Err("ffmpeg.crashed".into());
    }
    match std::fs::metadata(output) {
        Ok(m) if m.len() > 0 => Ok(()),
        _ => Err("ffmpeg.no_render".into()),
    }
}
