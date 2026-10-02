use std::io::{Read, Write};

use clack_extensions::state::PluginStateImpl;
use clack_plugin::prelude::*;
use clack_plugin::stream::{InputStream, OutputStream};

use crate::ClapPlugin;
use rift_plugin_params::UserParams;

impl<'a, P: ClapPlugin> PluginStateImpl for super::WrapperMainThread<'a, P> {
    fn save(&mut self, output: &mut OutputStream) -> Result<(), PluginError> {
        log::debug!("save (state)");

        let mut params = serde_json::Map::new();
        self.shared.params.serialize_params(&mut params)?;

        let mut persist = serde_json::Map::new();
        self.shared.params.serialize_persist(&mut persist)?;

        let mut root = serde_json::Map::new();
        root.insert(
            "version".to_string(),
            serde_json::Value::String(P::VERSION.to_string()),
        );
        root.insert("params".to_string(), serde_json::Value::Object(params));
        root.insert("persist".to_string(), serde_json::Value::Object(persist));

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

        let params = root
            .get("params")
            .and_then(serde_json::Value::as_object)
            .ok_or(PluginError::Message("Missing params"))?;
        self.shared.params.deserialize_params(params)?;

        // Absent in state saved before `persist` existed, so treat it as empty.
        if let Some(persist) = root.get("persist").and_then(serde_json::Value::as_object) {
            self.shared.params.deserialize_persist(persist)?;
        }

        Ok(())
    }
}
