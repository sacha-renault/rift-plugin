use rift_plugin::prelude::*;

#[derive(Default, DeriveEnumValues)]
enum WaveType {
    #[default]
    Saw,
    Square,
}

params! {
    struct OscillatorParam {
        param frequency: FloatParam {
            default: 20f32,
            min: 0f32,
            max: 1f32,
        },
        param wave: EnumParam<WaveType> {
            default: WaveType::Square,
        },
        nested_params: {
            param gain: FloatParam {
                default: 1f32,
                min: 0f32,
                max: 2f32
            },
            param pan: FloatParam {
                default: 0f32,
                min: -1f32,
                max: 1f32,
            },
        }
    }

    struct ChannelParam {
        param gain: FloatParam {
            default: 1f32,
            min: 0f32,
            max: 1f32,
        },
    }

    struct EnvelopeParam {
        param attack: FloatParam {
            default: 0.1f32,
            min: 0f32,
            max: 1f32,
        },
        // `Array<N>` uses an anonymous element type, hoisted to
        // `EnvelopeParamStages`.
        stages: Array<3> {
            param level: FloatParam {
                default: 1f32,
                min: 0f32,
                max: 1f32,
            },
        },
        // `Array<N, Type>` uses an explicitly named element type.
        channels: Array<2, ChannelParam>,
    }

    // params {
    //     left: ChannelParam,
    //     right: ChannelParam,
    //     oscillator: Array<10, OscillatorParam>,
    // }
}

fn main() {}
