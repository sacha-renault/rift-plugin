use std::sync::Arc;

use fundsp::fft;
use fundsp::prelude32::*;

type PhaseFn = dyn Fn(u32) -> f64 + Send + Sync;
type AmplitudeFn = dyn Fn(f64, u32) -> f64 + Send + Sync;

/// A single wave, described by its partials.
#[derive(Clone)]
pub struct WaveShape {
    phase: Arc<PhaseFn>,
    amplitude: Arc<AmplitudeFn>,
}

impl WaveShape {
    /// Build a shape from a harmonic description.
    pub fn new<P, A>(phase: P, amplitude: A) -> Self
    where
        P: Fn(u32) -> f64 + Send + Sync + 'static,
        A: Fn(f64, u32) -> f64 + Send + Sync + 'static,
    {
        Self {
            phase: Arc::new(phase),
            amplitude: Arc::new(amplitude),
        }
    }

    /// Build a shape from a single-cycle sample.
    ///
    /// The length of `wave` must be a power of two between 2 and 32768.
    pub fn from_wave(wave: &[f32]) -> Self {
        let mut data = Vec::from(wave);
        let spectrum: Arc<[Complex32]> = fft::real_fft(&mut data).to_vec().into();

        let phases = spectrum.clone();
        let amplitudes = spectrum;

        Self::new(
            move |partial: u32| {
                phases
                    .get(partial as usize)
                    .map_or(0.0, |bin| bin.arg() as f64 / f64::TAU)
            },
            move |_pitch: f64, partial: u32| {
                amplitudes
                    .get(partial as usize)
                    .map_or(0.0, |bin| bin.norm() as f64)
            },
        )
    }

    /// Phase of `partial` in `0...1`.
    pub fn phase(&self, partial: u32) -> f64 {
        (self.phase)(partial)
    }

    /// Relative amplitude of `partial` for a fundamental at `pitch` Hz.
    pub fn amplitude(&self, pitch: f64, partial: u32) -> f64 {
        (self.amplitude)(pitch, partial)
    }

    /// Bake this shape into a band-limited [`Wavetable`].
    pub fn to_wavetable(
        &self,
        min_pitch: f64,
        max_pitch: f64,
        tables_per_octave: f64,
    ) -> Wavetable {
        Wavetable::new(
            min_pitch,
            max_pitch,
            tables_per_octave,
            &|partial| self.phase(partial),
            &|pitch, partial| self.amplitude(pitch, partial),
        )
    }

    /// Crossfade between this shape and the other shape.
    pub fn crossfade(&self, other: &Self, weight: f64) -> Self {
        let weight = weight.clamp(0.0, 1.0);
        let phase_from = self.clone();
        let phase_to = other.clone();
        let amplitude_from = self.clone();
        let amplitude_to = other.clone();

        Self::new(
            move |partial| {
                phase_from.phase(partial) * (1.0 - weight) + phase_to.phase(partial) * weight
            },
            move |pitch, partial| {
                amplitude_from.amplitude(pitch, partial) * (1.0 - weight)
                    + amplitude_to.amplitude(pitch, partial) * weight
            },
        )
    }

    /// Just a pure sine.
    pub fn sine() -> Self {
        Self::new(
            |_| 0.0,
            |_pitch, partial| if partial == 1 { 1.0 } else { 0.0 },
        )
    }

    /// Just a square.
    pub fn square() -> Self {
        Self::new(
            |_| 0.0,
            |_pitch, partial| {
                if partial.is_multiple_of(2) {
                    0.0
                } else {
                    1.0 / partial as f64
                }
            },
        )
    }

    /// Just a saw.
    pub fn saw() -> Self {
        Self::new(
            |partial| if partial.is_multiple_of(2) { 0.5 } else { 0.0 },
            |_pitch, partial| 1.0 / partial as f64,
        )
    }

    /// Just a triangle.
    pub fn triangle() -> Self {
        Self::new(
            |partial| if (partial & 3) == 3 { 0.5 } else { 0.0 },
            |_pitch, partial| {
                if partial.is_multiple_of(2) {
                    0.0
                } else {
                    1.0 / (partial * partial) as f64
                }
            },
        )
    }

    pub fn soft_saw() -> Self {
        Self::new(
            |partial| if (partial & 3) == 1 { 0.0 } else { 0.5 },
            |_pitch, partial| 1.0 / (partial * partial) as f64,
        )
    }
}

/// How two [`WaveShape`]s are combined while morphing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Blend {
    /// Crossfade the partials of both shapes by the morph weight.
    #[default]
    Crossfade,
}

impl Blend {
    fn apply(self, from: &WaveShape, to: &WaveShape, weight: f64) -> WaveShape {
        match self {
            Blend::Crossfade => from.crossfade(to, weight),
        }
    }
}

/// Builder turning a list of [`WaveShape`]s into a series of morphing,
/// band-limited [`Wavetable`]s.
///
/// Building a table runs an FFT per band, so this belongs in plugin setup, not
/// on the audio thread.
#[derive(Clone)]
pub struct WavetableMorph {
    min_pitch: f64,
    max_pitch: f64,
    tables_per_octave: f64,
    shapes: Vec<WaveShape>,
    blend: Blend,
}

impl WavetableMorph {
    /// Start a morph across the pitch range `min_pitch..max_pitch` Hz with
    /// `tables_per_octave` tables per octave, exactly as in [`Wavetable::new`].
    pub fn new(min_pitch: f64, max_pitch: f64, tables_per_octave: f64) -> Self {
        Self {
            min_pitch,
            max_pitch,
            tables_per_octave,
            shapes: Vec::new(),
            blend: Blend::default(),
        }
    }

    /// Start a morph from an initial set of shapes.
    pub fn from_shapes(
        min_pitch: f64,
        max_pitch: f64,
        tables_per_octave: f64,
        shapes: impl IntoIterator<Item = WaveShape>,
    ) -> Self {
        Self::new(min_pitch, max_pitch, tables_per_octave).shapes(shapes)
    }

    /// Add one shape to the end of the morph axis.
    pub fn shape(mut self, shape: WaveShape) -> Self {
        self.shapes.push(shape);
        self
    }

    /// Add several shapes to the end of the morph axis.
    pub fn shapes(mut self, shapes: impl IntoIterator<Item = WaveShape>) -> Self {
        self.shapes.extend(shapes);
        self
    }

    /// Set how adjacent shapes are combined.
    pub fn blend(mut self, blend: Blend) -> Self {
        self.blend = blend;
        self
    }

    /// The blended shape at `position` in `0...1`.
    pub fn shape_at(&self, position: f64) -> WaveShape {
        assert!(
            !self.shapes.is_empty(),
            "WavetableMorph needs at least one shape"
        );

        let position = position.clamp(0.0, 1.0);
        let segments = self.shapes.len() - 1;

        if segments == 0 {
            return self.shapes[0].clone();
        }

        let scaled = position * segments as f64;
        let index = scaled.floor().min(segments as f64 - 1.0) as usize;
        let weight = scaled - index as f64;

        self.blend.apply(&self.shapes[index], &self.shapes[index + 1], weight)
    }

    /// Build the table at `position` in `0...1`.
    pub fn at(&self, position: f64) -> Arc<Wavetable> {
        Arc::new(self.shape_at(position).to_wavetable(
            self.min_pitch,
            self.max_pitch,
            self.tables_per_octave,
        ))
    }

    /// Build `steps` evenly spaced tables spanning the whole morph axis.
    pub fn build(&self, steps: usize) -> Vec<Arc<Wavetable>> {
        assert!(steps > 0, "WavetableMorph::build needs at least one step");

        (0..steps)
            .map(|step| {
                let position = if steps == 1 {
                    0.0
                } else {
                    step as f64 / (steps - 1) as f64
                };
                self.at(position)
            })
            .collect()
    }
}

pub fn morph_wavetables(
    min_pitch: f64,
    max_pitch: f64,
    tables_per_octave: f64,
    from: &WaveShape,
    to: &WaveShape,
    blend: Blend,
    steps: usize,
) -> Vec<Arc<Wavetable>> {
    WavetableMorph::new(min_pitch, max_pitch, tables_per_octave)
        .shapes([from.clone(), to.clone()])
        .blend(blend)
        .build(steps)
}
