use std::io::{Read, Write};

use clack_extensions::state::PluginStateImpl;
use clack_plugin::prelude::*;
use clack_plugin::stream::{InputStream, OutputStream};

use crate::prelude::*;
use crate::wrapper::ClapPlugin;

impl<'a, P: ClapPlugin> PluginStateImpl for super::WrapperMainThread<'a, P> {
    fn load(&mut self, input: &mut InputStream) -> Result<(), PluginError> {
        log::debug!("load (state)");

        let mut bytes = Vec::new();
        input
            .read_to_end(&mut bytes)
            .map_err(|_| PluginError::Message("Failed to read state"))?;

        let root: serde_json::Map<String, serde_json::Value> = serde_json::from_slice(&bytes)
            .map_err(|_| PluginError::Message("Failed to parse state as JSON"))?;

        let params_map = root
            .get("params")
            .and_then(|v| v.as_object())
            .ok_or(PluginError::Message("Missing params"))?;

        let params = &self.shared.params;
        for (id, value) in params_map {
            let plain_id: u32 = id
                .parse()
                .map_err(|_| PluginError::Message("Invalid param id"))?;
            let value = value
                .as_f64()
                .ok_or(PluginError::Message("Invalid param value"))?;
            params.set_value(ClapId::new(plain_id), value);
        }

        Ok(())
    }

    fn save(&mut self, output: &mut OutputStream) -> Result<(), PluginError> {
        log::debug!("save (state)");

        let params = &self.shared.params;
        let count = params.count();

        let mut params_vec: Vec<(String, serde_json::Value)> = Vec::new();
        for i in 0..count {
            let info = params
                .get_param_info(i)
                .ok_or(PluginError::Message("Not supposed to happen"))?;
            let value = params
                .get_value(info.id)
                .ok_or(PluginError::Message("Not supposed to happen"))?;
            params_vec.push((info.id.to_string(), serde_json::Value::from(value)));
        }

        let mut root = serde_json::Map::new();
        root.insert(
            "version".to_string(),
            serde_json::Value::String(P::VERSION.to_string()),
        );
        root.insert(
            "params".to_string(),
            serde_json::Value::Object(params_vec.into_iter().collect()),
        );

        let json = serde_json::to_string(&root)
            .map_err(|_| PluginError::Message("Failed to serialize state"))?;

        output
            .write_all(json.as_bytes())
            .map_err(|_| PluginError::Message("Failed to write state"))?;

        Ok(())
    }
}
