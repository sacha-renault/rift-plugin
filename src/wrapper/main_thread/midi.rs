use clack_extensions::note_ports::{NotePortInfoWriter, PluginNotePortsImpl};

use crate::prelude::*;
use crate::wrapper::ClapPlugin;

impl<'a, P: ClapPlugin> PluginNotePortsImpl for super::WrapperMainThread<'a, P> {
    fn count(&mut self, is_input: bool) -> u32 {
        log::debug!("{is_input}");
        P::MIDI_PORTS
            .iter()
            .filter(|p| p.is_input == is_input)
            .count() as u32
    }

    fn get(&mut self, index: u32, is_input: bool, writer: &mut NotePortInfoWriter) {
        log::debug!("{index}, {is_input}");
        let port_opt = P::MIDI_PORTS
            .iter()
            .filter(|p| p.is_input == is_input)
            .nth(index as usize);

        if let Some(port) = port_opt {
            writer.set(&port.into_note_port_info(index));
        } else {
            log::error!("Error getting midi port: (input: {is_input}, index: {index})")
        }
    }
}
