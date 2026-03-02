pub struct TimedAudioBlock<const N: usize> {
    raw: [f32; N],
    slice_length: usize,
    seconds: f64,
    beats: f64,
    // No need to carry tempo
}

impl<const N: usize> TimedAudioBlock<N> {
    pub fn new(slice: &[f32], seconds: f64, beats: f64) -> Self {
        let slice_length = slice.len();
        assert!(slice_length <= N);

        let mut raw = [0.0; N];
        raw[..slice_length].copy_from_slice(slice);
        TimedAudioBlock {
            raw,
            slice_length,
            seconds,
            beats,
        }
    }

    pub fn as_slice(&self) -> &[f32] {
        &self.raw[..self.slice_length]
    }

    pub fn iter(&self) -> impl Iterator<Item = &f32> {
        self.as_slice().iter()
    }

    pub fn len(&self) -> usize {
        self.slice_length
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
}
