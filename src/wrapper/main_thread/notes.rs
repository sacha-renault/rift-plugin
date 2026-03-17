use clack_extensions::note_ports::{
    NoteDialect, NotePortInfo, NotePortInfoWriter, PluginNotePortsImpl,
};
use clack_plugin::utils::ClapId;

use crate::prelude::*;
use crate::wrapper::ClapPlugin;

impl<'a, P: ClapPlugin> PluginNotePortsImpl for super::WrapperMainThread<'a, P> {
    fn count(&mut self, is_input: bool) -> u32 {
        if is_input { 1 } else { 0 }
    }

    fn get(&mut self, index: u32, is_input: bool, writer: &mut NotePortInfoWriter) {
        if is_input && index == 0 {
            writer.set(&NotePortInfo {
                preferred_dialect: Some(NoteDialect::Midi),
                supported_dialects: NoteDialect::Midi.into(),
                id: ClapId::new(0),
                name: b"",
            });
        }
    }
}
