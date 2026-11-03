//! Full-screen cutscenes — the opening movie New Game plays (`--intro-video`).
//!
//! The games ship their movies as Bink (`Data\Video\*.bik`), which nothing in
//! the workspace decodes. Like the Skyrim `.fuz` voice route
//! (`byroredux_audio::fuz`), decoding goes through the host's `ffmpeg`, probed
//! at use: with no `ffmpeg` on `PATH` the cutscene is skipped with one log line
//! and the game goes on.
//!
//! Two ffmpeg processes per movie:
//! - the soundtrack, decoded to a temp WAV before anything shows (a file, so it
//!   streams through the same kira path `PlayMusic` uses — and so the picture
//!   and the sound start on the same frame);
//! - the picture, piped as raw RGBA at a fixed rate into a bounded channel by
//!   a reader thread. The player shows the frame the wall clock says is due and
//!   drops the ones it is late for, so a slow upload never drifts off the sound.
//!
//! The world is paused while a cutscene is up (`DebugUiState::simulation_paused`)
//! and Escape, Space or Enter skips it.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::Instant;

use byroredux_core::ecs::World;

/// The rate the picture is resampled to. Bethesda's movies are 30 fps; a
/// fixed output rate is what lets the player pace frames without probing.
const FPS: f64 = 30.0;

/// Frames buffered ahead of the one on screen (~0.25 s at 720p: 30 MB).
const FRAME_QUEUE: usize = 8;

/// The picture size when the file is not a Bink whose header gives it.
const FALLBACK_SIZE: [usize; 2] = [1280, 720];

pub(crate) struct Cutscene {
    source: PathBuf,
    size: [usize; 2],
    stage: Stage,
}

enum Stage {
    /// The soundtrack is being decoded; nothing is shown but black.
    DecodingAudio(Receiver<Option<PathBuf>>),
    Playing {
        video: Child,
        frames: Receiver<Vec<u8>>,
        started: Instant,
        shown: u64,
        audio: Option<PathBuf>,
    },
}

/// What one tick of the player produced.
pub(crate) enum Tick {
    /// Still playing; show this frame if there is one.
    Frame(Option<Vec<u8>>),
    /// The picture ran out.
    Finished,
}

impl Cutscene {
    /// Start decoding `source`. `None` (logged) when the file is missing.
    pub(crate) fn start(source: PathBuf) -> Option<Self> {
        if !source.is_file() {
            log::warn!("cutscene: '{}' does not exist", source.display());
            return None;
        }
        let size = bink_size(&source).unwrap_or(FALLBACK_SIZE);
        let (tx, rx) = mpsc::channel();
        let audio_source = source.clone();
        std::thread::Builder::new()
            .name("cutscene-audio".into())
            .spawn(move || {
                let _ = tx.send(decode_audio(&audio_source));
            })
            .ok()?;
        log::info!("cutscene: '{}' ({}x{})", source.display(), size[0], size[1]);
        Some(Self { source, size, stage: Stage::DecodingAudio(rx) })
    }

    pub(crate) fn size(&self) -> [usize; 2] {
        self.size
    }

    /// Advance to the frame the clock says is due.
    pub(crate) fn tick(&mut self, world: &World) -> Tick {
        if let Stage::DecodingAudio(rx) = &self.stage {
            let audio = match rx.try_recv() {
                Ok(audio) => audio,
                Err(TryRecvError::Empty) => return Tick::Frame(None),
                Err(TryRecvError::Disconnected) => None,
            };
            let Some((video, frames)) = spawn_video(&self.source, self.size) else {
                remove_temp(audio.as_deref());
                return Tick::Finished;
            };
            if let Some(path) = &audio {
                play_soundtrack(world, path);
            }
            self.stage = Stage::Playing { video, frames, started: Instant::now(), shown: 0, audio };
        }
        let Stage::Playing { frames, started, shown, .. } = &mut self.stage else {
            return Tick::Finished;
        };
        let due = (started.elapsed().as_secs_f64() * FPS) as u64 + 1;
        let mut latest = None;
        while *shown < due {
            match frames.try_recv() {
                Ok(frame) => {
                    latest = Some(frame);
                    *shown += 1;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    return if latest.is_some() { Tick::Frame(latest) } else { Tick::Finished };
                }
            }
        }
        Tick::Frame(latest)
    }

    /// Stop both decoders and the soundtrack (skip, or the movie ended).
    pub(crate) fn stop(self, world: &World) {
        if let Stage::Playing { mut video, audio, .. } = self.stage {
            let _ = video.kill();
            let _ = video.wait();
            if let Some(mut sound) = world.try_resource_mut::<byroredux_audio::AudioWorld>() {
                sound.stop_script_music(0.5);
            }
            // kira streams the WAV from disk; on Windows an open file cannot be
            // deleted, so a temp still in use is left for the OS temp cleanup.
            remove_temp(audio.as_deref());
        }
    }
}

/// Resolve `--intro-video <file>`: an existing path as given, else a file
/// under the data folder's `Video` directory.
pub(crate) fn intro_video_path(args: &[String]) -> Option<PathBuf> {
    let name = crate::cli_args::parse_string_arg(args, "--intro-video")?;
    let direct = PathBuf::from(&name);
    if direct.is_file() {
        return Some(direct);
    }
    let (_, data) = crate::cli_args::esm_discovery_root(args).ok()?;
    crate::systems::script_requests::find_case_insensitive(&data, &format!("Video\\{name}"))
        .or(Some(data.join("Video").join(name)))
}

/// The width and height a Bink 1/2 header declares (`BIK`/`KB2` magic,
/// little-endian u32 at bytes 20 and 24).
fn bink_size(path: &Path) -> Option<[usize; 2]> {
    let mut header = [0u8; 28];
    std::fs::File::open(path).ok()?.read_exact(&mut header).ok()?;
    if &header[..3] != b"BIK" && &header[..3] != b"KB2" {
        return None;
    }
    let width = u32::from_le_bytes(header[20..24].try_into().ok()?) as usize;
    let height = u32::from_le_bytes(header[24..28].try_into().ok()?) as usize;
    (width > 0 && height > 0 && width <= 7680 && height <= 4320).then_some([width, height])
}

fn ffmpeg() -> Command {
    let mut command = Command::new("ffmpeg");
    command
        .args(["-nostdin", "-hide_banner", "-loglevel", "error"])
        .stdin(Stdio::null());
    command
}

/// The soundtrack as a fresh temp WAV, or `None` (logged) when there is no
/// `ffmpeg` or the movie has no sound.
fn decode_audio(source: &Path) -> Option<PathBuf> {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or(0);
    let target = std::env::temp_dir().join(format!("byroredux-cutscene-{}-{stamp}.wav", std::process::id()));
    // `-n`: never overwrite — a path that already exists is refused, not followed.
    let output = ffmpeg()
        .arg("-n")
        .arg("-i")
        .arg(source)
        .args(["-vn", "-acodec", "pcm_s16le"])
        .arg(&target)
        .stdout(Stdio::null())
        .output();
    match output {
        Ok(output) if output.status.success() && target.is_file() => Some(target),
        Ok(output) => {
            log::warn!(
                "cutscene: no soundtrack for '{}': {}",
                source.display(),
                String::from_utf8_lossy(&output.stderr).trim()
            );
            remove_temp(Some(&target));
            None
        }
        Err(error) => {
            log::warn!("cutscene: could not start ffmpeg ({error}); install ffmpeg to see cutscenes");
            None
        }
    }
}

/// The picture decoder and the channel its reader thread fills.
fn spawn_video(source: &Path, size: [usize; 2]) -> Option<(Child, Receiver<Vec<u8>>)> {
    let filter = format!("fps={FPS},scale={}:{}", size[0], size[1]);
    let mut child = match ffmpeg()
        .arg("-i")
        .arg(source)
        .args(["-an", "-vf", &filter, "-f", "rawvideo", "-pix_fmt", "rgba", "pipe:1"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            log::warn!("cutscene: could not start ffmpeg ({error}); install ffmpeg to see cutscenes");
            return None;
        }
    };
    let mut stdout = child.stdout.take()?;
    let frame_bytes = size[0] * size[1] * 4;
    let (tx, rx) = mpsc::sync_channel(FRAME_QUEUE);
    std::thread::Builder::new()
        .name("cutscene-video".into())
        .spawn(move || loop {
            let mut frame = vec![0u8; frame_bytes];
            if stdout.read_exact(&mut frame).is_err() || tx.send(frame).is_err() {
                break;
            }
        })
        .ok()?;
    Some((child, rx))
}

fn play_soundtrack(world: &World, path: &Path) {
    match byroredux_audio::load_streaming_sound_from_file(path) {
        Ok(sound) => {
            if let Some(mut audio) = world.try_resource_mut::<byroredux_audio::AudioWorld>() {
                audio.play_script_music(sound, 1.0, 0.0, false);
            }
        }
        Err(error) => log::warn!("cutscene: soundtrack failed to decode: {error}"),
    }
}

fn remove_temp(path: Option<&Path>) {
    if let Some(path) = path {
        let _ = std::fs::remove_file(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_size_a_bink_header_declares() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("intro.bik");
        let mut header = vec![0u8; 44];
        header[..4].copy_from_slice(b"BIKi");
        header[20..24].copy_from_slice(&1280u32.to_le_bytes());
        header[24..28].copy_from_slice(&720u32.to_le_bytes());
        std::fs::write(&path, &header).unwrap();
        assert_eq!(bink_size(&path), Some([1280, 720]));
    }

    #[test]
    fn a_file_that_is_not_bink_has_no_header_size() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("intro.mp4");
        std::fs::write(&path, vec![0u8; 44]).unwrap();
        assert_eq!(bink_size(&path), None);
    }

    #[test]
    fn a_missing_movie_does_not_start() {
        assert!(Cutscene::start(PathBuf::from("no/such/movie.bik")).is_none());
    }
}
