mod gui;
mod params;
mod plugin;
mod shared;

use rift_plugin::prelude::*;

export_clap_plugin!(plugin::Plugin);
