use rift_plugin::prelude::*;

#[derive(Default, DeriveEnumValues)]
enum WaveType {
    #[default]
    Saw,
    Square,
}

params! {
    struct OscillatorParam {
        frequency: FloatParam {
            default: 20f32,
            min: 0f32,
            max: 1f32,
        },
        wave: EnumParam<WaveType> {
            default: WaveType::Square,
        },
        nested_params: {
            gain: FloatParam {
                default: 1f32,
                min: 0f32,
                max: 2f32
            },
            pan: FloatParam {
                default: 0f32,
                min: -1f32,
                max: 1f32,
            },
        }
    }

    struct ChannelParam {
        gain: FloatParam {
            default: 1f32,
            min: 0f32,
            max: 1f32,
        },
    }

    struct EnvelopeParam {
        attack: FloatParam {
            default: 0.1f32,
            min: 0f32,
            max: 1f32,
        },
        // `Array<N>` uses an anonymous element type, hoisted to
        // `EnvelopeParamStages`.
        stages: Array<3> {
            level: FloatParam {
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
