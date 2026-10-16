//! A small stereo synth demonstrating `rift-plugin`: MIDI-driven wavetable
//! oscillators feeding a rebuildable fundsp filter graph.

mod graph;
mod gui;
mod params;
mod plugin;

use rift_plugin::prelude::*;

rift_plugin::prelude::export_clap_plugin!(plugin::FunDspPlugin);
