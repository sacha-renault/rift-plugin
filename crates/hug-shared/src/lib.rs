use core::f64;
use std::{cell::RefCell, rc::Rc};

pub mod utils;

#[derive(Clone, Copy)]
pub struct ChannelsInfo {
    pub current: usize,
    pub total_channels: usize,
}

impl ChannelsInfo {
    pub fn is_last_channel(&self) -> bool {
        // idx starts at 0
        self.current + 1 == self.total_channels
    }
}

#[derive(Clone)]
pub struct BlockInfo {
    pub seconds: f64,
    pub beats: f64,
    pub samplerate: f64,
    pub seconds_per_beat: f64,
}

impl BlockInfo {
    /// Increments the internal timestamps based on a number of processed samples.
    pub fn advance_by_samples(&mut self, samples: usize) {
        let delta_seconds = samples as f64 / self.samplerate;

        // Advance seconds if they exist
        self.seconds += delta_seconds;
        self.beats += delta_seconds / self.seconds_per_beat;
    }
}

#[derive(Clone, Copy)]
pub struct BlockTime {
    /// This define the timing (seconds and beats) withing the song
    /// of the first beat of the BUFFER this block belongs to
    /// We might see many blocks with same seconds or beats if buffer_size > N
    seconds: f64,
    beats: f64,
}

impl BlockTime {
    #[inline]
    pub fn new(seconds: f64, beats: f64) -> Self {
        Self { seconds, beats }
    }

    #[inline]
    pub fn new_opt(seconds: Option<f64>, beats: Option<f64>) -> Self {
        Self {
            seconds: seconds.unwrap_or(f64::NAN),
            beats: beats.unwrap_or(f64::NAN),
        }
    }

    #[inline]
    pub fn none() -> Self {
        Self {
            seconds: f64::NAN,
            beats: f64::NAN,
        }
    }

    pub fn seconds(&self) -> Option<f64> {
        if self.seconds.is_nan() {
            None
        } else {
            Some(self.seconds)
        }
    }

    pub fn beats(&self) -> Option<f64> {
        if self.beats.is_nan() {
            None
        } else {
            Some(self.beats)
        }
    }

    #[inline]
    pub fn beat_phase(&self) -> Option<f64> {
        self.beats().map(|b| b.fract())
    }

    #[inline]
    pub fn beat_num(&self) -> Option<i64> {
        self.beats().map(|b| b.floor() as i64)
    }
}

pub type RcCell<T> = Rc<RefCell<T>>;
