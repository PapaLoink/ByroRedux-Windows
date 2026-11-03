//! Bounded FFmpeg subprocess playback of locally installed original movies.
use anyhow::{anyhow, bail, Context, Result};
use byroredux_audio::{Frame, Sound, SoundSettings};
use std::{io::Read, path::Path, process::{Child, Command, Stdio},
    sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}, mpsc::{self, Receiver, SyncSender}},
    thread::{self, JoinHandle}, time::{Duration, Instant}};

const MAX_AUDIO_BYTES: usize = 64 * 1024 * 1024;
const PROCESS_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Debug, Clone, Copy)]
pub(super) struct Metadata { pub width: u32, pub height: u32, pub fps: f64, pub duration: f64, pub audio: bool }
impl Metadata {
    fn parse(bytes: &[u8]) -> Result<Self> {
        let root: serde_json::Value = serde_json::from_slice(bytes)?;
        let streams = root["streams"].as_array().context("missing movie streams")?;
        let video = streams.iter().find(|v| v["codec_type"] == "video").context("movie has no video")?;
        let width = video["width"].as_u64().unwrap_or(0);
        let height = video["height"].as_u64().unwrap_or(0);
        let rate = video["r_frame_rate"].as_str().unwrap_or("");
        let (num, den) = rate.split_once('/').context("invalid movie frame rate")?;
        let fps = num.parse::<f64>()? / den.parse::<f64>()?;
        let duration = root["format"]["duration"].as_str().context("missing duration")?.parse::<f64>()?;
        if width == 0 || height == 0 || width > 1920 || height > 1080 ||
            !fps.is_finite() || !(1.0..=120.0).contains(&fps) ||
            !duration.is_finite() || !(0.0..=600.0).contains(&duration) {
            bail!("unsupported movie dimensions/rate/duration");
        }
        Ok(Self { width: width as u32, height: height as u32, fps, duration,
            audio: streams.iter().any(|v| v["codec_type"] == "audio") })
    }
}

pub(super) enum Event { Ready(Metadata, Option<Sound>), Frame(f64, Vec<u8>), End, Error(String) }

#[derive(Default)]
struct Control {
    child: Mutex<Option<Child>>,
    cancelled: AtomicBool,
    reading: AtomicBool,
    activity: Mutex<Option<Instant>>,
    done: AtomicBool,
}
impl Control {
    fn kill(&self) {
        if let Some(child) = self.child.lock().unwrap().as_mut() { let _ = child.kill(); }
    }
    fn spawn(&self, program: &str, arguments: &[String]) -> Result<std::process::ChildStdout> {
        if self.cancelled.load(Ordering::Acquire) { bail!("movie cancelled"); }
        let mut command = Command::new(program);
        command.args(arguments).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null());
        #[cfg(windows)] {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000); // CREATE_NO_WINDOW, decoder only
        }
        let mut child = command.spawn().with_context(|| format!("start {program}; install FFmpeg for original Bink movies"))?;
        let stdout = child.stdout.take().context("missing decoder stdout")?;
        *self.child.lock().unwrap() = Some(child);
        self.reading.store(false, Ordering::Release);
        Ok(stdout)
    }
    fn begin_read(&self) {
        *self.activity.lock().unwrap() = Some(Instant::now());
        self.reading.store(true, Ordering::Release);
    }
    fn finish(&self) -> Result<()> {
        self.begin_read();
        loop {
            let status = self.child.lock().unwrap().as_mut().context("missing decoder child")?.try_wait()?;
            if let Some(status) = status {
                self.child.lock().unwrap().take();
                self.reading.store(false, Ordering::Release);
                if !status.success() { bail!("movie decoder exited with {status}"); }
                return Ok(());
            }
            thread::sleep(Duration::from_millis(5));
        }
    }
    fn capture(&self, program: &str, arguments: &[String], max: usize) -> Result<Vec<u8>> {
        let stdout = self.spawn(program, arguments)?;
        self.begin_read();
        let mut bytes = Vec::new();
        stdout.take(max as u64 + 1).read_to_end(&mut bytes)?;
        self.reading.store(false, Ordering::Release);
        if bytes.len() > max { self.kill(); bail!("decoder output exceeds {max} bytes"); }
        self.finish()?;
        Ok(bytes)
    }
}

pub(super) struct Decoder {
    receiver: Option<Receiver<Event>>,
    control: Arc<Control>,
    worker: Option<JoinHandle<()>>,
    watchdog: Option<JoinHandle<()>>,
}
impl Decoder {
    pub fn start(path: &Path) -> Result<Self> {
        let (sender, receiver) = mpsc::sync_channel(2);
        let control = Arc::new(Control::default());
        let worker_control = Arc::clone(&control);
        let path = path.to_string_lossy().into_owned();
        let worker = thread::Builder::new().name("original-movie-decoder".into()).spawn(move || {
            let result = decode(&worker_control, &sender, &path);
            if let Err(error) = result { let _ = sender.send(Event::Error(format!("{error:#}"))); }
            worker_control.kill();
            if let Some(mut child) = worker_control.child.lock().unwrap().take() { let _ = child.wait(); }
            worker_control.done.store(true, Ordering::Release);
        })?;
        let watch = Arc::clone(&control);
        let watchdog = thread::Builder::new().name("movie-decoder-watchdog".into()).spawn(move || {
            while !watch.done.load(Ordering::Acquire) {
                let expired = watch.reading.load(Ordering::Acquire) && watch.activity.lock().unwrap()
                    .is_some_and(|t| t.elapsed() > PROCESS_TIMEOUT);
                if watch.cancelled.load(Ordering::Acquire) || expired { watch.kill(); }
                thread::sleep(Duration::from_millis(20));
            }
        });
        let watchdog = match watchdog {
            Ok(watchdog) => watchdog,
            Err(error) => {
                control.cancelled.store(true, Ordering::Release); control.kill();
                drop(receiver); let _ = worker.join(); return Err(error.into());
            }
        };
        Ok(Self { receiver: Some(receiver), control, worker: Some(worker), watchdog: Some(watchdog) })
    }
    pub fn poll(&self) -> Option<Event> { self.receiver.as_ref()?.try_recv().ok() }
}
impl Drop for Decoder {
    fn drop(&mut self) {
        self.control.cancelled.store(true, Ordering::Release);
        self.receiver.take(); // unblock a producer waiting on the bounded frame queue
        self.control.kill();
        if let Some(worker) = self.worker.take() { let _ = worker.join(); }
        if let Some(watchdog) = self.watchdog.take() { let _ = watchdog.join(); }
    }
}

fn decode(control: &Control, sender: &SyncSender<Event>, path: &str) -> Result<()> {
    let args = |parts: &[&str]| parts.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let metadata = control.capture("ffprobe", &args(&["-v", "error", "-show_entries",
        "stream=codec_type,width,height,r_frame_rate:format=duration", "-of", "json", path]), 64*1024)?;
    let meta = Metadata::parse(&metadata)?;
    let audio = if meta.audio {
        let bytes = control.capture("ffmpeg", &args(&["-v", "error", "-nostdin", "-i", path,
            "-map", "0:a:0", "-vn", "-ac", "2", "-ar", "44100", "-f", "f32le", "pipe:1"]), MAX_AUDIO_BYTES)?;
        if bytes.len() % 8 != 0 { bail!("partial stereo PCM frame"); }
        let frames: Vec<_> = bytes.chunks_exact(8).map(|f| Frame { left: byroredux_core::bytes::le_f32_at(f,0).unwrap(),
            right: byroredux_core::bytes::le_f32_at(f,4).unwrap() }).collect();
        if frames.iter().any(|f| !f.left.is_finite() || !f.right.is_finite()) { bail!("non-finite movie audio"); }
        Some(Sound { sample_rate: 44100, frames: frames.into(), settings: SoundSettings::default(), slice: None })
    } else { None };
    sender.send(Event::Ready(meta, audio)).map_err(|_| anyhow!("movie cancelled"))?;
    let mut stdout = control.spawn("ffmpeg", &args(&["-v", "error", "-nostdin", "-i", path,
        "-an", "-fps_mode", "passthrough", "-f", "rawvideo", "-pix_fmt", "rgba", "pipe:1"]))?;
    let frame_bytes = meta.width as usize * meta.height as usize * 4;
    let mut index = 0u64;
    loop {
        if control.cancelled.load(Ordering::Acquire) { return Ok(()); }
        let mut pixels = vec![0; frame_bytes];
        control.begin_read();
        let mut filled = 0;
        while filled < frame_bytes {
            let n = stdout.read(&mut pixels[filled..])?;
            if n == 0 { break; }
            filled += n;
        }
        control.reading.store(false, Ordering::Release);
        if filled == 0 { break; }
        if filled != frame_bytes { bail!("truncated decoded video frame"); }
        if index as f64 / meta.fps > meta.duration + 1.0 { bail!("video exceeds declared duration"); }
        sender.send(Event::Frame(index as f64 / meta.fps, pixels)).map_err(|_| anyhow!("movie cancelled"))?;
        index += 1;
    }
    control.finish()?;
    if index == 0 { bail!("video decoded no frames"); }
    let _ = sender.send(Event::End);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn front_end_cancels_a_producer_blocked_on_the_bounded_frame_queue() {
        let (sender, receiver)=mpsc::sync_channel(2);
        let control=Arc::new(Control::default());
        let worker=thread::spawn(move || {
            while sender.send(Event::Frame(0.0,vec![0;4])).is_ok() {}
        });
        let decoder=Decoder { receiver:Some(receiver),control,worker:Some(worker),watchdog:None };
        drop(decoder); // must drop the receiver before joining the blocked worker
    }

    #[test]
    fn front_end_rejects_unbounded_or_invalid_movie_metadata() {
        let metadata = |w,rate| format!(r#"{{"streams":[{{"codec_type":"video","width":{w},"height":720,"r_frame_rate":"{rate}"}}],"format":{{"duration":"25"}}}}"#);
        assert!(Metadata::parse(metadata(1280,"30/1").as_bytes()).is_ok());
        for (width, rate) in [(0,"30/1"),(999999,"30/1"),(1280,"30/0"),(1280,"999/1")] {
            assert!(Metadata::parse(metadata(width,rate).as_bytes()).is_err());
        }
    }
}
