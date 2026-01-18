use anyhow::Context;
use rodio::{Decoder, OutputStreamHandle, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

pub struct Player {
    stream_handle: OutputStreamHandle,
    sink: Option<Sink>,
    current_track: Option<PathBuf>,
    state: PlaybackState,
    started_at: Option<Instant>,
    position_at_start: Duration,
    duration: Option<Duration>,
    volume: f32,
    muted: bool,
    volume_before_mute: f32,
}

impl Player {
    pub fn new(stream_handle: OutputStreamHandle, volume: f32) -> Self {
        Self {
            stream_handle,
            sink: None,
            current_track: None,
            state: PlaybackState::Stopped,
            started_at: None,
            position_at_start: Duration::ZERO,
            duration: None,
            volume: volume.clamp(0.0, 1.0),
            muted: false,
            volume_before_mute: volume.clamp(0.0, 1.0),
        }
    }

    pub fn state(&self) -> PlaybackState {
        self.state
    }

    pub fn current_track(&self) -> Option<&PathBuf> {
        self.current_track.as_ref()
    }

    pub fn duration(&self) -> Option<Duration> {
        self.duration
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn is_muted(&self) -> bool {
        self.muted
    }

    pub fn position(&self) -> Duration {
        match (self.state, self.started_at) {
            (PlaybackState::Playing, Some(started_at)) => {
                self.position_at_start.saturating_add(started_at.elapsed())
            }
            _ => self.position_at_start,
        }
    }

    pub fn play_track(&mut self, path: PathBuf) -> anyhow::Result<()> {
        self.play_track_from(path, Duration::ZERO, PlaybackState::Playing)
    }

    fn play_track_from(
        &mut self,
        path: PathBuf,
        start_at: Duration,
        state: PlaybackState,
    ) -> anyhow::Result<()> {
        self.stop();

        let sink = Sink::try_new(&self.stream_handle).context("create audio sink")?;
        sink.set_volume(if self.muted { 0.0 } else { self.volume });

        let (source, duration) = load_source(&path, start_at)?;
        sink.append(source);

        match state {
            PlaybackState::Paused => sink.pause(),
            PlaybackState::Playing => sink.play(),
            PlaybackState::Stopped => sink.stop(),
        }

        self.duration = duration;
        self.current_track = Some(path);
        self.sink = Some(sink);
        self.state = state;
        self.position_at_start = start_at;
        self.started_at = (state == PlaybackState::Playing).then(Instant::now);
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(sink) = self.sink.take() {
            sink.stop();
        }
        self.current_track = None;
        self.state = PlaybackState::Stopped;
        self.started_at = None;
        self.position_at_start = Duration::ZERO;
        self.duration = None;
    }

    pub fn toggle_pause(&mut self) {
        match self.state {
            PlaybackState::Playing => {
                if let Some(sink) = self.sink.as_ref() {
                    sink.pause();
                }
                self.position_at_start = self.position();
                self.started_at = None;
                self.state = PlaybackState::Paused;
            }
            PlaybackState::Paused => {
                if let Some(sink) = self.sink.as_ref() {
                    sink.play();
                }
                self.started_at = Some(Instant::now());
                self.state = PlaybackState::Playing;
            }
            PlaybackState::Stopped => {}
        }
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
        if !self.muted {
            if let Some(sink) = self.sink.as_ref() {
                sink.set_volume(self.volume);
            }
        }
    }

    pub fn adjust_volume(&mut self, delta: f32) {
        self.set_volume(self.volume + delta);
    }

    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
        if self.muted {
            self.volume_before_mute = self.volume;
            if let Some(sink) = self.sink.as_ref() {
                sink.set_volume(0.0);
            }
        } else {
            self.volume = self.volume_before_mute;
            if let Some(sink) = self.sink.as_ref() {
                sink.set_volume(self.volume);
            }
        }
    }

    pub fn seek_relative(&mut self, delta: Duration, forward: bool) -> anyhow::Result<()> {
        let Some(track) = self.current_track.clone() else {
            return Ok(());
        };

        let current_pos = self.position();
        let desired = if forward {
            current_pos.saturating_add(delta)
        } else {
            current_pos.saturating_sub(delta)
        };

        let clamped = match self.duration {
            Some(total) => desired.min(total),
            None => desired,
        };

        let next_state = self.state;
        self.play_track_from(track, clamped, next_state)?;
        Ok(())
    }

    pub fn is_finished(&self) -> bool {
        matches!(self.state, PlaybackState::Playing)
            && self.sink.as_ref().is_some_and(|sink| sink.empty())
    }
}

fn load_source(
    path: &Path,
    seek_to: Duration,
) -> anyhow::Result<(Box<dyn Source<Item = f32> + Send>, Option<Duration>)> {
    let file = File::open(path).with_context(|| format!("open '{}'", path.display()))?;
    let decoder = Decoder::new(BufReader::new(file))
        .with_context(|| format!("decode '{}'", path.display()))?;

    let duration = decoder.total_duration();
    let source: Box<dyn Source<Item = f32> + Send> = if seek_to > Duration::ZERO {
        Box::new(decoder.skip_duration(seek_to).convert_samples::<f32>())
    } else {
        Box::new(decoder.convert_samples::<f32>())
    };

    let source = fade_in(source, Duration::from_millis(20));
    Ok((source, duration))
}

fn fade_in(
    source: Box<dyn Source<Item = f32> + Send>,
    duration: Duration,
) -> Box<dyn Source<Item = f32> + Send> {
    let sample_rate = source.sample_rate();
    let fade_samples = (duration.as_secs_f64() * f64::from(sample_rate)).round() as u64;
    Box::new(FadeIn {
        inner: source,
        fade_samples: fade_samples.max(1),
        emitted: 0,
    })
}

struct FadeIn {
    inner: Box<dyn Source<Item = f32> + Send>,
    fade_samples: u64,
    emitted: u64,
}

impl Iterator for FadeIn {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let sample = self.inner.next()?;
        if self.emitted >= self.fade_samples {
            return Some(sample);
        }

        let gain = (self.emitted as f32 / self.fade_samples as f32).clamp(0.0, 1.0);
        self.emitted = self.emitted.saturating_add(1);
        Some(sample * gain)
    }
}

impl Source for FadeIn {
    fn current_frame_len(&self) -> Option<usize> {
        self.inner.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.inner.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.inner.total_duration()
    }
}
