pub struct OscillatorPosition {
    /// position inside of the oscillator
    position: f32,

    /// 1. / samplerate
    sr_recip: f32,
    frequency: Option<f32>,
}

impl OscillatorPosition {
    pub fn new(samplerate: f32) -> Self {
        Self {
            position: 0.,
            sr_recip: samplerate.recip(),
            frequency: None,
        }
    }

    pub fn trigger_with_phase<F>(&mut self, frequency: f32, phase_generator: F)
    where
        F: FnOnce() -> f32,
    {
        self.frequency = Some(frequency);
        self.position = phase_generator();
    }

    pub fn trigger(&mut self, frequency: f32) {
        self.frequency = Some(frequency);
        self.position = 0.;
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = Some(frequency);
    }

    pub fn deactivate(&mut self) {
        self.frequency = None;
    }

    #[inline]
    pub fn get_current(&self) -> f32 {
        self.position
    }

    #[inline]
    pub fn get_next(&mut self) -> f32 {
        self.advance_by(1);
        self.get_current()
    }

    pub fn advance_by(&mut self, sample_count: usize) {
        if let Some(frequency) = self.frequency {
            let pos = self.position + self.sr_recip * frequency * (sample_count as f32);

            // todo!()
            // this might drift because of f32 precision
            // a way to avoid that is calculating position from absolute position
            // this would requires knowing frame it was turned on and the actual frame
            // Note: this would be annoying if transport stop playing, or if changing frequency etc ...
            // maybe this is "good enough" actually. Needs to be checked anyway ...
            self.position = pos.rem_euclid(1.);
        }
    }

    pub fn is_active(&self) -> bool {
        self.frequency.is_some()
    }
}
