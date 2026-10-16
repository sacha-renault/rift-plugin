use rift_plugin::prelude::*;

bitflags::bitflags! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ParamFlags: u32 {
        const NEED_REBUILD = 1 << 1;
    }
}

#[derive(Params)]
pub struct OscillatorParams {
    #[param(id = "WtPosition", default = 0)]
    pub wt_position: FloatParam,

    #[param(id = "Pan", default = 0, range = skew(-0.5, 0.5, 1))]
    pub pan: FloatParam,

    #[param(id = "Gain", default = 1, range = linear(0, 2))]
    pub gain: FloatParam,

    #[param(id = "Attack", range = skew(0.001, 4.0, 3), default = 0.01)]
    pub attack: SharedFloatParam,

    #[param(id = "Decay", range = skew(0.001, 4.0, 3), default = 0.1)]
    pub decay: SharedFloatParam,

    #[param(id = "Sustain", range = linear(0.0, 1.0), default = 1.0)]
    pub sustain: SharedFloatParam,

    #[param(id = "Release", range = skew(0.001, 4.0, 3), default = 0.2)]
    pub release: SharedFloatParam,
}

#[derive(Default, EnumValues)]
pub enum FilterKind {
    #[default]
    Lowpass,
    Highpass,
}

#[derive(Params)]
pub struct FilterParams {
    #[param(id = "Cutoff", range = linear(20, 20000), default = 440.0)]
    pub cutoff: SharedFloatParam,

    #[param(tags = ParamFlags::NEED_REBUILD)]
    pub kind: EnumParam<FilterKind>,
}

pub const OSC_COUNT: usize = 1;
pub const FILTER_COUNT: usize = 2;

#[derive(Params)]
pub struct FunDspParams {
    #[nested]
    pub filters: [FilterParams; FILTER_COUNT],

    #[nested]
    pub oscillators: [OscillatorParams; OSC_COUNT],
}
