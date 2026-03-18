use std::io::{Read, Write};

use clack_extensions::state::PluginStateImpl;
use clack_plugin::prelude::*;
use clack_plugin::stream::{InputStream, OutputStream};

use crate::prelude::*;

impl<'a, P: ClapPlugin> PluginStateImpl for super::WrapperMainThread<'a, P> {
    fn save(&mut self, output: &mut OutputStream) -> Result<(), PluginError> {
        log::debug!("save (state)");

        let mut root = serde_json::Map::new();
        root.insert(
            "version".to_string(),
            serde_json::Value::String(P::VERSION.to_string()),
        );

        let mut state_buf = Vec::new();
        self.shared.params.serialize(&mut state_buf)?;
        let state: serde_json::Value = serde_json::from_slice(&state_buf)
            .map_err(|_| PluginError::Message("Failed to serialize state"))?;
        root.insert("params".to_string(), state);

        let json = serde_json::to_string(&root)
            .map_err(|_| PluginError::Message("Failed to serialize state"))?;
        output
            .write_all(json.as_bytes())
            .map_err(|_| PluginError::Message("Failed to write state"))?;

        Ok(())
    }

    fn load(&mut self, input: &mut InputStream) -> Result<(), PluginError> {
        log::debug!("load (state)");

        let mut bytes = Vec::new();
        input
            .read_to_end(&mut bytes)
            .map_err(|_| PluginError::Message("Failed to read state"))?;

        let root: serde_json::Map<String, serde_json::Value> = serde_json::from_slice(&bytes)
            .map_err(|_| PluginError::Message("Failed to parse state as JSON"))?;

        let params_value = root
            .get("params")
            .ok_or(PluginError::Message("Missing params"))?;

        let params_buf = serde_json::to_vec(params_value)
            .map_err(|_| PluginError::Message("Failed to deserialize params"))?;
        self.shared.params.deserialize(&mut params_buf.as_slice())?;

        Ok(())
    }
}
