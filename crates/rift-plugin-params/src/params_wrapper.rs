use std::collections::BTreeMap;

use clack_extensions::params::ParamInfo;
use clack_plugin::{plugin::PluginError, utils::ClapId};

use crate::{Param, ParamCollection, ParamPtr};

pub struct ParamsWrapper {
    params: BTreeMap<ClapId, ParamPtr>,
}

impl ParamsWrapper {
    pub fn new(params: Vec<ParamPtr>) -> Self {
        let mut map = BTreeMap::new();

        for param in params {
            let info = param.param_info();

            if map.insert(info.id, param).is_some() {
                panic!("Two or more parameters share the same id.")
            }
        }

        Self { params: map }
    }
}

impl ParamCollection for ParamsWrapper {
    fn count(&self) -> u32 {
        self.params.len() as u32
    }

    fn deserialize(&self, _: &mut dyn std::io::prelude::Read) -> Result<(), PluginError> {
        unimplemented!()
    }

    fn serialize(&self, _: &mut dyn std::io::prelude::Write) -> Result<(), PluginError> {
        unimplemented!()
    }

    fn get_param_info<'a>(&'a self, index: u32) -> Option<ParamInfo<'a>> {
        self.params
            .values()
            .nth(index as usize)
            .map(ParamPtr::param_info)
    }

    fn get_value(&self, id: ClapId) -> Option<f32> {
        self.params.get(&id).map(ParamPtr::get_raw)
    }

    fn set_value(&self, id: ClapId, value: f32) {
        if let Some(param) = self.params.get(&id) {
            param.set_raw(value);
        }
    }

    fn set_value_normalized(&self, id: ClapId, value: f32) {
        if let Some(param) = self.params.get(&id) {
            param.set_normalized(value);
        }
    }

    fn text_to_value(&self, id: ClapId, text: &std::ffi::CStr) -> Option<f32> {
        if let Some(param) = self.params.get(&id) {
            param.text_to_value(text)
        } else {
            None
        }
    }

    fn value_to_text(
        &self,
        id: ClapId,
        value: f32,
        writer: &mut clack_extensions::params::ParamDisplayWriter,
    ) -> std::fmt::Result {
        if let Some(param) = self.params.get(&id) {
            param.value_to_text(value, writer)
        } else {
            Err(::core::fmt::Error)
        }
    }
}
