use std::io::{Read, Write};

use clack_extensions::params::{HostParams, ParamRescanFlags};
use clack_extensions::state::PluginStateImpl;
use clack_plugin::prelude::*;
use clack_plugin::stream::{InputStream, OutputStream};
use rift_plugin_gui::{GuiEvent, ParamSync};
use serde_json::{Map, Value};

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
        let result = self.inner_load(root);

        // Ping GUI and tell him it has to resync
        if matches!(self.gui.param_sync(), ParamSync::Push) {
            self.shared.states.gui_tasks().post(GuiEvent::ParamRescan);
        }

        // host must rescan param after load
        if let Some(ext) = self.host.get_extension::<HostParams>() {
            ext.rescan(&mut self.host, ParamRescanFlags::VALUES);
        }

        result
    }
}

impl<'a, P: ClapPlugin> super::WrapperMainThread<'a, P> {
    fn inner_load(&mut self, root: Map<String, Value>) -> Result<(), PluginError> {
        let params = root
            .get("params")
            .and_then(serde_json::Value::as_object)
            .ok_or(PluginError::Message("Missing params"))?;
        self.shared.params.deserialize_params(params)?;

        let persist = root
            .get("persist")
            .and_then(serde_json::Value::as_object)
            .ok_or(PluginError::Message("Missing persist"))?;
        self.shared.params.deserialize_persist(persist)?;

        Ok(())
    }
}
