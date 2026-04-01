use std::f32::consts::TAU;

pub(super) const fn compute_w0(f0: f32, samplerate: f32) -> f32 {
    TAU * f0 / samplerate
}

pub(super) fn compute_alpha(w0: f32, q: f32) -> f32 {
    w0.sin() / (2. * q)
}
