bitflags::bitflags! {
    /// Automatic actions the wrapper performs on incoming events *once per block*, before
    /// `ClapPlugin::process` runs.
    ///
    /// Each flag is one thing the wrapper does *for* the plugin. A set flag means the
    /// wrapper consumes that event type; leaving it unset leaves it to the plugin,
    /// typically through the sample-accurate `zip_events` iterator. Because `zip_events`
    /// always yields every event, opting into auto-handling *and* reading the same type
    /// from `zip_events` acts on the event twice.
    ///
    /// See `ClapPlugin::EVENT_PRE_PROCESS` for the per-flag meaning and how to combine
    /// flags.
    pub struct EventPreProcess: u8 {
        /// Write incoming host parameter values into the parameter store.
        const PARAM_APPLY_CHANGE   = 1 << 0;
        /// Call `ClapPlugin::param_changed` for host parameter events.
        ///
        /// Wrapper ALWAYS notify on GUI change, wether this bit is set or not.
        const PARAM_NOTIFY_CHANGE  = 1 << 1;
        /// Call `ClapPlugin::on_midi_message` for MIDI events.
        const MIDI_NOTIFY_EVENT = 1 << 2;
    }
}
