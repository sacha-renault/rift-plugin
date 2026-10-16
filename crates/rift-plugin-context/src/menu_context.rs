use clack_plugin::utils::ClapId;

#[derive(bon::Builder)]
pub struct MenuContext {
    param: Option<ClapId>,
}

impl MenuContext {
    /// The parameter this menu targets, or `None` for the global menu.
    pub fn param(&self) -> Option<ClapId> {
        self.param
    }
}
