//! Polyphonic oscillator engine.
//!
//! This module provides a two-layer abstraction for audio oscillators:
//!
//! [`OscillatorPosition`] tracks the phase of a single voice, advancing it
//! sample by sample according to a frequency and sample rate.
//!
//! [`Oscillator`] manages a bank of 127 voices (one per MIDI note) and
//! combines their output through a caller-supplied waveform function.
mod oscillator;
mod oscillator_position;

pub use oscillator::Oscillator;
pub use oscillator_position::OscillatorPosition;
