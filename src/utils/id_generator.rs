use std::hash::{DefaultHasher, Hash, Hasher};

use clack_plugin::utils::ClapId;

pub fn hash_name_into_id(name: &'static str) -> ClapId {
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    let hash = hasher.finish();

    // todo!() can we handle collision ?
    // Truncate the last 32 bits
    ClapId::new(hash as u32)
}
