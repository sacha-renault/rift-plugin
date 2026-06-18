use rift_plugin::prelude::clack_extensions::params::ParamInfoFlags;
use rift_plugin::prelude::param_queue_impl::{ControlPoint, ControlPoints};
use rift_plugin::prelude::*;

#[derive(Default, DeriveEnumValues)]
pub enum MyEnum {
    #[default]
    None,

    #[enum_values(text = "123")]
    Zuper,
    BadBadBad,
}

#[derive(DeriveParams)]
pub struct PluginParams {
    #[param(name = "CLIP")]
    pub clip: FloatParam,

    #[param(name = "GAIN")]
    pub gain: FloatParam,

    #[param(name = "cutoff")]
    pub cutoff: FloatParam,

    #[param(name = "LFO TIME")]
    pub lfo_time: FloatParam,

    #[param(name = "MY_ENUM")]
    pub enum_p: EnumParam<MyEnum>,

    #[param(name = "SKIP")]
    pub skip: BoolParam,

    #[param(name = "LFO", persistent)]
    pub lfo: ParamQueue<ControlPoints>,
}

impl Default for PluginParams {
    fn default() -> Self {
        let clip = FloatParam::builder().default(1.0).max_value(2.0).build();
        let lfo_time = FloatParam::builder()
            .default(1.0)
            .min_value(1. / 256.)
            .max_value(25.)
            .build();

        let gain = FloatParam::builder()
            .default(1.0)
            .max_value(2.0)
            .mapping(RangeMapping::Skew(2.))
            .unit(" dB")
            .build();

        let enum_p = EnumParam::new(MyEnum::Zuper).with_flags(ParamInfoFlags::IS_AUTOMATABLE);
        let skip = BoolParam::builder().default(false).build();

        let points = vec![
            ControlPoint {
                x: 0.,
                y: 1.,
                tension: -10.,
            },
            // ControlPoint {
            //     x: 0.25,
            //     y: 0.25,
            //     tension: 0.0,
            // },
            // ControlPoint {
            //     x: 0.5,
            //     y: 0.5,
            //     tension: 10.,
            // },
            // ControlPoint {
            //     x: 1.,
            //     y: 1.,
            //     tension: 0.5,
            // },
        ];
        let cpoints = ControlPoints::with_value(points, 10);
        let lfo = ParamQueue::new(cpoints, 50);
        let cutoff = FloatParam::builder()
            .unit("Hz")
            .default(300.)
            .min_value(10.)
            .max_value(44100. / 2.)
            .build();

        Self {
            clip,
            lfo_time,
            enum_p,
            skip,
            gain,
            cutoff,
            lfo,
        }
    }
}
