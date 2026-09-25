//! Helpers to build fundsp [`AtomicTable`]s.
//!
//! `AtomicTable::new` only takes a ready-made `&[f32]`. Two constructors are
//! missing upstream: one from a harmonic description, and one from a band of a
//! band-limited [`Wavetable`].

use std::borrow::Borrow;

use fundsp::prelude32::*;
use fundsp::wavetable::Wavetable;

/// Resolution used when flattening a [`Wavetable`] band.
///
/// `Wavetable` builds each band with `clamp(32, 8192, 4 * harmonics)`, so it
/// exposes no single length and no way to query one. 8192 is the upper bound
/// that formula can produce: sampling any band at this length can only ever
/// *upsample*, never lose detail. `Wavetable::at` interpolates, so the shorter
/// (higher-pitch) bands come out smooth.
const BAND_LEN: usize = 8192;

/// Synthesize a single-cycle, band-limited wave from a harmonic description.
///
/// Copy of fundsp's private `wavetable::make_wave`, which is not exported.
fn make_wave<P, A>(pitch: f64, phase: &P, amplitude: &A) -> Vec<f32>
where
    P: Fn(u32) -> f64,
    A: Fn(f64, u32) -> f64,
{
    // Fade out upper harmonics starting from 20 kHz.
    const MAX_F: f64 = 22_000.0;
    const FADE_F: f64 = 20_000.0;

    let harmonics = floor(MAX_F / pitch) as usize;

    // Target at least 4x oversampling when choosing wave table length.
    let target_len = 4 * harmonics;

    let length = clamp(32, 8192, target_len.next_power_of_two());

    let mut a = vec![Complex32::ZERO; length];

    for (i, bin) in a.iter_mut().enumerate().skip(1).take(harmonics) {
        let f = pitch * i as f64;

        // Get harmonic amplitude.
        let w = amplitude(pitch, i as u32);
        // Fade out high frequencies.
        let w = w * smooth5(clamp01(delerp(MAX_F, FADE_F, f)));
        // Insert partial.
        if w > 0.0 {
            *bin = Complex32::from_polar(w as f32, (f64::TAU * phase(i as u32)) as f32);
        }
    }

    fft::inverse_fft(&mut a);

    let z = length as f32;
    a.iter().map(|x| x.im * z).collect()
}

pub trait AtomicTableExt {
    /// Build a band-limited table from a harmonic description.
    ///
    /// `phase(i)` is the phase of partial `i` in `0...1`, and
    /// `amplitude(pitch, i)` its relative amplitude.
    fn from_harmonics<P, A>(pitch: f64, phase: &P, amplitude: &A) -> Self
    where
        P: Fn(u32) -> f64,
        A: Fn(f64, u32) -> f64;

    // fn update_from_harmonics<P, A>(&self, pitch: f64, phase: &P, amplitude: &A)
    // where
    //     P: Fn(u32) -> f64,
    //     A: Fn(f64, u32) -> f64;

    /// Flatten one band of a band-limited [`Wavetable`] into a mutable table.
    ///
    /// `band` indexes the wavetable's pitch-ordered bands: `0` is the lowest
    /// pitch, i.e. the band with the most harmonics.
    ///
    /// Accepts anything that borrows a [`Wavetable`], so both `&wt` and the
    /// `Arc<Wavetable>` returned by `*_table()` work.
    fn from_wavetable<W: Borrow<Wavetable>>(wavetable: W, band: usize) -> Self;
}

impl AtomicTableExt for AtomicTable {
    fn from_harmonics<P, A>(pitch: f64, phase: &P, amplitude: &A) -> Self
    where
        P: Fn(u32) -> f64,
        A: Fn(f64, u32) -> f64,
    {
        Self::new(&make_wave(pitch, phase, amplitude))
    }

    // fn update_from_harmonics<P, A>(&self, pitch: f64, phase: &P, amplitude: &A)
    // where
    //     P: Fn(u32) -> f64,
    //     A: Fn(f64, u32) -> f64,
    // {
    // }

    fn from_wavetable<W: Borrow<Wavetable>>(wavetable: W, band: usize) -> Self {
        let wavetable = wavetable.borrow();

        let wave: Vec<f32> = (0..BAND_LEN)
            .map(|i| wavetable.at(band, i as f32 / BAND_LEN as f32))
            .collect();

        Self::new(&wave)
    }
}
