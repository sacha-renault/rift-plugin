use crate::dev_prelude::*;

pub const STYLESHEET: &str = concat!(
    include_str!("rift.css"),
    include_str!("rift-button.css"),
    include_str!("rift-dial.css"),
    include_str!("rift-panel.css"),
    include_str!("rift-selector.css"),
    include_str!("rift-toggle.css"),
    include_str!("rift-meter.css"),
);

/// Installs the Rift theme in the application.
pub(crate) fn install(cx: &mut Context) {
    cx.add_stylesheet(STYLESHEET)
        .expect("the Rift stylesheet is embedded in the binary");
}
