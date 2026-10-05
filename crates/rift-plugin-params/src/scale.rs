#[derive(Debug, Default, Clone, Copy)]
pub enum Scale {
    #[default]
    Linear,

    /// Power curve: more resolution at bottom (skew > 1) or top (skew < 1)
    Skew(f32),
}

impl Scale {
    pub fn denormalize(&self, normalized: f32, min: f32, max: f32) -> f32 {
        let t = match self {
            Self::Linear => normalized,
            Self::Skew(s) => normalized.powf(*s),
        };
        min + t * (max - min)
    }

    pub fn normalize(&self, value: f32, min: f32, max: f32) -> f32 {
        match self {
            Self::Linear => (value - min) / (max - min),
            Self::Skew(s) => ((value - min) / (max - min)).powf(1.0 / s),
        }
    }
}

#[cfg(test)]
mod tests {}
