use clack_extensions::note_ports::{NoteDialect, NoteDialects, NotePortInfo};
use clack_plugin::utils::ClapId;

/// A declarative description of a MIDI/note port, used to tell the host
/// what note dialects the plugin supports on a given port.
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct MidiPort<'a> {
    pub(crate) name: &'a [u8],
    pub(crate) supported_dialects: NoteDialects,
    pub(crate) preferred_dialect: Option<NoteDialect>,
    pub(crate) is_input: bool,
}

impl<'a> MidiPort<'a> {
    pub(crate) const fn new(name: &'a [u8], is_input: bool) -> Self {
        Self {
            name,
            supported_dialects: NoteDialects::empty(),
            preferred_dialect: None,
            is_input,
        }
    }

    /// Creates a note input port with the given `name`.
    pub const fn input(name: &'a [u8]) -> Self {
        Self::new(name, true)
    }

    /// Creates a note output port with the given `name`.
    pub const fn output(name: &'a [u8]) -> Self {
        Self::new(name, false)
    }

    /// Sets the dialect the host should prefer when sending events to this port.
    pub const fn preferred_dialect(mut self, dialect: NoteDialect) -> Self {
        self.preferred_dialect = Some(dialect);
        self
    }

    /// Adds a dialect to the set of dialects this port can handle.
    /// Call multiple times to support more than one.
    pub const fn add_supported_dialect(mut self, dialect: NoteDialects) -> Self {
        self.supported_dialects = self.supported_dialects.union(dialect);
        self
    }

    /// Converts this builder-like instance into an owned [`AudioPortInfo`] tagged with a plugin-local ID.
    pub fn into_note_port_info(&self, index: u32) -> NotePortInfo<'a> {
        NotePortInfo {
            id: ClapId::new(index),
            name: self.name,
            supported_dialects: self.supported_dialects,
            preferred_dialect: self.preferred_dialect,
        }
    }
}
