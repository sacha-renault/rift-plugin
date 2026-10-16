use std::sync::Arc;

use fundsp::prelude32::*;
use rift_plugin::prelude::*;

use crate::params::{FilterKind, FunDspParams};

/// Owns the fundsp graph and rebuilds it on a background thread.
pub struct Shared {
    pub(crate) tasks: BackgroundExecutor<f64, Box<dyn AudioUnit>>,
}

impl FromParams<FunDspParams> for Shared {
    fn from_params(params: Arc<FunDspParams>) -> Self {
        let tasks = BackgroundExecutor::new(16, 16, move |samplerate: f64| {
            build_net(samplerate, &params)
        });
        Self { tasks }
    }
}

fn build_filter(kind: FilterKind) -> Net {
    match kind {
        FilterKind::Lowpass => Net::wrap(Box::new(lowpass())),
        FilterKind::Highpass => Net::wrap(Box::new(highpass())),
    }
}

pub fn build_net(samplerate: f64, params: &FunDspParams) -> Box<dyn AudioUnit> {
    fn filter(kind: FilterKind, cutoff: An<Var>) -> Net {
        Net::wrap(Box::new(pass() | cutoff | dc(0.5))) >> build_filter(kind)
    }

    // stereo passthrough
    let mut graph = Net::wrap(Box::new(pass() | pass()));

    for fparam in params.filters.iter() {
        let kind = fparam.kind.value();
        let cutoff = &fparam.cutoff;
        graph = graph >> (filter(kind, cutoff.var()) | filter(kind, cutoff.var()));
    }

    let mut unit = Box::new(graph) as Box<dyn AudioUnit>;
    unit.set_sample_rate(samplerate);
    unit.allocate();
    unit
}
