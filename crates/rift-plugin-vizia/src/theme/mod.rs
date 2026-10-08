use crate::dev_prelude::*;

pub const STYLESHEET: &str = include_str!("rift.css");

/// Installs the Rift theme in the application.
pub(crate) fn install(cx: &mut Context) {
    cx.add_stylesheet(STYLESHEET)
        .expect("the Rift stylesheet is embedded in the binary");
}
