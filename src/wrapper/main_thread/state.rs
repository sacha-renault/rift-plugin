use std::collections::HashMap;
use std::io::{Read, Write};

use clack_extensions::state::PluginStateImpl;
use clack_plugin::prelude::*;
use clack_plugin::stream::{InputStream, OutputStream};
use tinyjson::JsonValue;

use crate::prelude::*;
use crate::wrapper::ClapPlugin;

impl<'a, P: ClapPlugin> PluginStateImpl for super::WrapperMainThread<'a, P> {
    fn load(&mut self, input: &mut InputStream) -> Result<(), PluginError> {
        let mut bytes = Vec::new();
        input
            .read(&mut bytes)
            .map_err(|_| PluginError::Message("Failed to read state"))?;

        let json: JsonValue = String::from_utf8(bytes)
            .map_err(|_| PluginError::Message("Failed to read state as UTF-8"))?
            .parse()
            .map_err(|_| PluginError::Message("Failed to parse state as JSON"))?;

        let root = json
            .get::<HashMap<_, _>>()
            .ok_or(PluginError::Message("Expected JSON object"))?;

        let params_map = root
            .get("params")
            .and_then(|v| v.get::<HashMap<_, _>>())
            .ok_or(PluginError::Message("Missing params"))?;

        let params = &self.shared.params;
        for (id, value) in params_map {
            let plain_id: u32 = id
                .parse()
                .map_err(|_| PluginError::Message("Invalid param id"))?;
            let id = ClapId::new(plain_id);

            let value = value
                .get::<f64>()
                .ok_or(PluginError::Message("Invalid param value"))?;

            params.set_value(id, *value);
        }

        Ok(())
    }

    fn save(&mut self, output: &mut OutputStream) -> Result<(), PluginError> {
        let params = &self.shared.params;
        let count = params.count();

        let mut params_map: HashMap<String, JsonValue> = HashMap::new();
        for i in 0..count {
            let info = params
                .get_param_info(i)
                .ok_or(PluginError::Message("Not supposed to happen"))?;
            let id = info.id;
            let value = params
                .get_value(id)
                .ok_or(PluginError::Message("Not supposed to happen"))?;
            params_map.insert(id.to_string(), JsonValue::Number(value));
        }

        let mut root: HashMap<String, JsonValue> = HashMap::new();
        root.insert(
            "version".to_string(),
            JsonValue::String(P::VERSION.to_string()),
        );
        root.insert("params".to_string(), JsonValue::Object(params_map));

        let json = JsonValue::Object(root);
        output
            .write_all(
                json.stringify()
                    .map_err(|_| PluginError::Message("Failed to serialize state"))?
                    .as_bytes(),
            )
            .map_err(|_| PluginError::Message("Failed to write state"))?;

        Ok(())
    }
}
