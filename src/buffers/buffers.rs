use clack_plugin::plugin::PluginError;
use clack_plugin::prelude::ChannelPair;
use clack_plugin::process::Audio;

use crate::prelude::MainAudioPort;

use super::Buffer;

/// Handles audio buffer management for a plugin instance.
///
/// Wraps [`Audio`] and [`MainAudioPort`] to provide access to inputs, outputs, and the main I/O port.
/// Centralizes logic for retrieving buffers while accounting for host limitations (e.g., copying input to output).
///
/// # Note:
/// (and todo!()) since accessing any port requires a mutable reference, it isn't possible to use main and auxiliary port
/// in the same time. The plugin needs to hold a scratch buffer (allocated during activation) and copy required auxiliary port
/// into it.
pub struct Buffers<'a> {
    audio: Audio<'a>,
    main_config: MainAudioPort,
    is_main_copied: bool,
}

impl<'a> Buffers<'a> {
    /// Create a new view on [`clack_plugin::process::Audio`] struct.
    pub(crate) fn new(audio: Audio<'a>, main_config: MainAudioPort) -> Self {
        Self {
            audio,
            main_config,
            is_main_copied: false,
        }
    }

    /// Get the (not shifted by main port) input at `index`.
    fn get_input(&mut self, index: usize) -> Result<Buffer<'_>, PluginError> {
        let data = self
            .audio
            .input_port(index)
            .ok_or(PluginError::Message("No input ports found"))?
            .channels()?
            .into_f32()
            .ok_or(PluginError::Message("Expected f32 input"))?;

        Ok(Buffer::input(data))
    }

    /// Get the (not shifted by main port) output at `index`.
    fn get_output(&mut self, index: usize) -> Result<Buffer<'_>, PluginError> {
        let data = self
            .audio
            .output_port(index)
            .ok_or(PluginError::Message("No output ports found"))?
            .channels()?
            .into_f32()
            .ok_or(PluginError::Message("Expected f32 output"))?;

        Ok(Buffer::output(data))
    }

    /// Retrieve port pair 0 and copy, if needed, input into output.
    fn main_input_into_output(&mut self) -> Result<(), PluginError> {
        if self.is_main_copied {
            return Ok(());
        }

        let mut port_pair = self
            .audio
            .port_pair(0)
            .ok_or(PluginError::Message("No input/output ports found"))?;

        let mut paired_channels = port_pair
            .channels()?
            .into_f32()
            .ok_or(PluginError::Message("Expected f32 input/output"))?;

        self.is_main_copied = true;
        for paired in paired_channels.iter_mut() {
            // There is 4 cases
            // either InputOutput => handled with copy
            // Input only, should never happens
            // Output only, should never happens
            // Inplace ... In that case the output bfr
            // is already correct
            if let ChannelPair::InputOutput(i, o) = paired {
                o.copy_from_slice(i)
            }
        }

        Ok(())
    }

    /// First copy input into output and return the main output.
    ///
    /// This function must be called only in the case of [`MainAudioPort::InputOutput`].
    fn get_main_io(&mut self) -> Result<Buffer<'_>, PluginError> {
        self.main_input_into_output()?;
        self.get_output(0)
    }

    /// Get the declared main buffer.
    ///
    /// Depeding on [`MainAudioPort`], it can return different kinds of buffers. Wrapped in a
    /// convinient struct for consistant api calls.
    /// - [`MainAudioPort::InputOnly`]: the input channels.
    /// - [`MainAudioPort::OutputOnly`]: the output channels. This channels has no
    ///   certainty to be empty, so if you don't process it, you might want to set 0s at least
    ///   to avoid noizy output.
    /// - [`MainAudioPort::InputOutput`]: copies input into output then returns
    ///   the output buffer. If the host provides an in-place buffer, no copy
    ///   is performed and the single buffer is returned directly.
    pub fn main(&mut self) -> Result<Buffer<'_>, PluginError> {
        match self.main_config {
            MainAudioPort::InputOnly(_) => self.get_input(0),
            MainAudioPort::OutputOnly(_) => self.get_output(0),
            MainAudioPort::InputOutput(_) => self.get_main_io(),
        }
    }

    /// Same as [`Buffers::main`], but will panic if it returns a [`Result::Err`].
    pub fn main_unchecked(&mut self) -> Buffer<'_> {
        self.main().unwrap()
    }

    /// Returns the auxiliary input at `index`.
    ///
    /// As main ports are always 0, index may be shifted by one.
    pub fn input_aux(&mut self, index: usize) -> Result<Buffer<'_>, PluginError> {
        let start_idx = match self.main_config {
            MainAudioPort::OutputOnly(_) => 0,
            _ => 1,
        };

        self.get_input(start_idx + index)
    }

    /// Same as [`Buffers::input_aux`], but will panic if it returns a [`Result::Err`].
    pub fn input_aux_unchecked(&mut self, index: usize) -> Buffer<'_> {
        self.input_aux(index).unwrap()
    }

    /// Returns the auxiliary output at `index`.
    ///
    /// As main ports are always 0, index may be shifted by one.
    pub fn output_aux(&mut self, index: usize) -> Result<Buffer<'_>, PluginError> {
        let start_idx = match self.main_config {
            MainAudioPort::InputOnly(_) => 0,
            _ => 1,
        };

        self.get_output(start_idx + index)
    }

    /// Same as [`Buffers::output_aux`], but will panic if it returns a [`Result::Err`].
    pub fn output_aux_unchecked(&mut self, index: usize) -> Buffer<'_> {
        self.output_aux(index).unwrap()
    }
}

impl<'a> Drop for Buffers<'a> {
    fn drop(&mut self) {
        match self.main_config {
            MainAudioPort::InputOutput(_) if !self.is_main_copied => {
                let _ = self.main_input_into_output();
            }

            // Nothing to do on input only
            // We might wanna add later something
            // in the case output only, if it wasn't cleared might
            _ => {}
        }
    }
}
