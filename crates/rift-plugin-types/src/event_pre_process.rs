bitflags::bitflags! {
    /// Actions the wrapper does with incoming events *before* `ClapPlugin::process` runs.
    pub struct EventPreProcess: u8 {
        /// Write incoming host parameter values into the parameter store.
        const PARAM_APPLY   = 1 << 0;
        /// Call `ClapPlugin::param_changed` for host parameter events.
        const PARAM_NOTIFY  = 1 << 1;
        /// Call `ClapPlugin::on_midi_message` for MIDI events.
        const MIDI_CALLBACK = 1 << 2;
    }
}
