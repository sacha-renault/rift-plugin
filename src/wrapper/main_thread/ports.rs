pub use clack_extensions::audio_ports::*;
pub use clack_extensions::params::*;

use crate::prelude::*;
use crate::type_wrapper::PAIR_PORT_ID;
use crate::wrapper::ClapPlugin;

impl<'a, P: ClapPlugin> PluginAudioPortsImpl for super::WrapperMainThread<'a, P> {
    /// Returns the total number of audio ports for the given direction.
    ///
    /// Counts 1 for the main port (if it exists in the requested direction)
    /// plus any auxiliary ports declared in `P::AUX_AUDIO_PORTS`.
    fn count(&mut self, is_input: bool) -> u32 {
        let aux_number = P::AUX_AUDIO_PORTS
            .iter()
            .filter(|port| port.is_input == is_input)
            .count() as u32;

        match (P::MAIN_AUDIO_PORTS, is_input) {
            (MainAudioPort::InputOnly(_), true) => aux_number + 1,
            (MainAudioPort::OutputOnly(_), false) => aux_number + 1,
            (MainAudioPort::InputOutput(_), _) => aux_number + 1,
            _ => aux_number,
        }
    }

    /// Returns the port info for the given index and direction.
    ///
    /// Port indices are laid out as follows:
    /// - Index 0: the main port (input or output depending on `is_input`), if it exists
    ///   in the requested direction. For `InputOutput` plugins, both directions share
    ///   the same channel count and are marked as an in-place pair.
    /// - Index 1+: auxiliary ports from `P::AUX_AUDIO_PORTS`, filtered by direction,
    ///   in declaration order.
    ///
    /// # Panics
    /// Panics if `index` is out of range for the given direction. This should never
    /// happen if the host respects the count returned by [`count`]
    fn get(&mut self, index: u32, is_input: bool, writer: &mut AudioPortInfoWriter) {
        let main_port = match (P::MAIN_AUDIO_PORTS, is_input) {
            (MainAudioPort::InputOnly(channels), true) => {
                Some(AudioPort::input(b"Input", channels).set_port_flags(AudioPortFlags::IS_MAIN))
            }
            (MainAudioPort::OutputOnly(channels), false) => {
                Some(AudioPort::output(b"Output", channels).set_port_flags(AudioPortFlags::IS_MAIN))
            }
            (MainAudioPort::InputOutput(channels), _) => Some(
                AudioPort::new(
                    if is_input { b"Input" } else { b"Output" },
                    channels,
                    is_input,
                )
                .set_port_flags(AudioPortFlags::IS_MAIN)
                .set_in_place(PAIR_PORT_ID),
            ),
            _ => None,
        };

        // Create an iterator that optionally starts with the main port, then all matching aux ports
        let mut all_ports = main_port
            .iter()
            .chain(P::AUX_AUDIO_PORTS.iter().filter(|p| p.is_input == is_input));

        if let Some(port_info) = all_ports.nth(index as usize) {
            writer.set(&port_info.into_audio_port_info(index))
        } else {
            panic!("Invalid port index {} for is_input={}", index, is_input);
        }
    }
}
