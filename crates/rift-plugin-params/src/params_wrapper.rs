use std::collections::HashMap;

use clack_extensions::params::ParamInfo;
use clack_plugin::{plugin::PluginError, utils::ClapId};

use crate::{Param, ParamCollection, ParamPtr};

pub struct ParamsWrapper {
    params: Box<[ParamPtr]>,
    mapping: HashMap<ClapId, usize>,
}

impl ParamsWrapper {
    pub fn new(params: Vec<ParamPtr>) -> Self {
        let mut mapping = HashMap::new();

        for (idx, param) in params.iter().enumerate() {
            let info = param.param_info();

            if mapping.insert(info.id, idx).is_some() {
                panic!("Two or more parameters share the same id.")
            }
        }

        Self {
            params: params.into_boxed_slice(),
            mapping,
        }
    }

    fn get(&self, id: ClapId) -> Option<ParamPtr> {
        if let Some(&idx) = self.mapping.get(&id) {
            Some(self.params[idx])
        } else {
            None
        }
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
        self.params.get(index as usize).map(ParamPtr::param_info)
    }

    fn get_value(&self, id: ClapId) -> Option<f32> {
        self.get(id).map(|ptr| ptr.plain())
    }

    fn set_value(&self, id: ClapId, value: f32) {
        if let Some(param) = self.get(id) {
            param.set_plain(value);
        }
    }

    fn set_value_normalized(&self, id: ClapId, value: f32) {
        if let Some(param) = self.get(id) {
            param.set_normalized(value);
        }
    }

    fn text_to_value(&self, id: ClapId, text: &std::ffi::CStr) -> Option<f32> {
        if let Some(param) = self.get(id) {
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
        if let Some(param) = self.get(id) {
            param.value_to_text(value, writer)
        } else {
            Err(::core::fmt::Error)
        }
    }
}
