use clack_extensions::params::ParamInfoFlags;
use rift_plugin::prelude::*;

#[derive(Default, DeriveEnumValues)]
enum WaveType {
    #[default]
    Saw,
    Square,
}

#[derive(Params)]
pub struct OscillatorParam {
    #[param(name = "Frequency", range = 0..1, default = 20.0)]
    pub frequency: FloatParam,

    #[param(name = "Wave", default = WaveType::Square)]
    pub wave: EnumParam<WaveType>,

    #[nested]
    pub nested_params: NestedParams,
}

#[derive(Params)]
pub struct NestedParams {
    #[param(name = "Gain", range = 0..2, default = 1.0)]
    pub gain: FloatParam,

    #[param(name = "Pan", range = -1..1)]
    pub pan: FloatParam,
}

#[derive(Params)]
pub struct ChannelParam {
    #[param(name = "Gain", range = 0..1, default = 1.0)]
    pub gain: FloatParam,
}

#[derive(Params)]
pub struct StageParams {
    #[param(name = "Level", range = 0..1, default = 1.0)]
    pub level: FloatParam,
}

#[derive(Params)]
pub struct EnvelopeParam {
    #[param(name = "Attack", range = 0..1, default = 0.1)]
    pub attack: FloatParam,

    /// An array of nested structs (`[T; N]`), each element getting its own
    /// `stages[i]` module.
    #[nested]
    pub stages: [StageParams; 3],

    #[nested]
    pub channels: [ChannelParam; 2],
}

#[derive(Params)]
pub struct SomeParams {
    #[param(name = "Level")]
    pub level: FloatParam,

    #[param(name = "Gain", flags = ParamInfoFlags::IS_AUTOMATABLE)]
    pub gain: FloatParam,
}

#[derive(Params)]
pub struct DevParams {
    #[nested]
    pub left: ChannelParam,

    #[nested]
    pub right: ChannelParam,

    #[nested]
    pub oscillator: [OscillatorParam; 10],

    /// `#[nested(module = "...")]` overrides the module segment taken from the
    /// field name.
    #[nested(module = "some")]
    pub some: SomeParams,
}

fn main() {
    let params = DevParams::new();
    params.some.level.value();
}
