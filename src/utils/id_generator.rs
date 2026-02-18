use std::sync::atomic::{AtomicU32, Ordering};

use clack_plugin::utils::ClapId;

static NEXT_PARAM_ID: AtomicU32 = AtomicU32::new(0);

pub fn get_next_param_id() -> ClapId {
    let id = NEXT_PARAM_ID.fetch_add(1, Ordering::Relaxed);
    ClapId::new(id)
}
