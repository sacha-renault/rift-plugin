//! https://cytomic.com/files/dsp/SvfLinearTrapOptimised2.pdf
//! Will Pirkle's books has a section on state variable filters

#[derive(Copy, Clone)]
pub enum SVFKind {
    LowPass,
    BandPass,
    HighPass,
    Notch,
    Peak,
    All,
}

pub struct SVFOutput {
    v0: f32,
    v1: f32,
    v2: f32,
}

#[derive(Clone, Copy)]
pub struct SVFCoeffs {
    a1: f32,
    a2: f32,
    a3: f32,
    k: f32,
    g: f32, // Keep G to be able to recalculate coeffs
}

pub struct SVFStates {
    ic1eq: f32,
    ic2eq: f32,
}

impl SVFStates {
    #[inline(always)]
    fn update(&mut self, v1: f32, v2: f32) {
        self.ic1eq = 2. * v1 - self.ic1eq;
        self.ic2eq = 2. * v2 - self.ic2eq;
    }
}

pub struct SVF {
    coeffs: SVFCoeffs,
    states: SVFStates,
    kind: SVFKind,
}

impl SVF {
    pub fn next(&mut self, v0: f32) -> SVFOutput {
        let SVFCoeffs { a1, a2, a3, .. } = self.coeffs;
        let SVFStates { ic1eq, ic2eq } = self.states;

        let temp = v0 - ic2eq;
        let v1 = a1 * ic1eq + a2 * temp;
        let v2 = ic2eq + a2 * ic1eq + a3 * temp;

        self.states.update(v1, v2);
        SVFOutput { v0, v1, v2 }
    }

    pub fn next_sample(&mut self, x: f32) -> f32 {
        use SVFKind::*;

        let SVFOutput { v0, v1, v2 } = self.next(x);

        match self.kind {
            LowPass => v2,
            HighPass => v0 - self.coeffs.k * v1 - v2,
            BandPass => v1,
            Notch => v0 - self.coeffs.k * v1,
            Peak => v0 - self.coeffs.k * v1 - 2. * v2,
            All => x - 2. * self.coeffs.k * v1,
        }
    }
}
