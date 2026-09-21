//! Name-derived parameter ids.

use clack_plugin::utils::ClapId;

const FNV_OFFSET_BASIS: u32 = 0x811c_9dc5;
const FNV_PRIME: u32 = 0x0100_0193;

/// FNV-1a over `bytes`, continuing from `hash`.
const fn fnv1a(hash: u32, bytes: &[u8]) -> u32 {
    let mut hash = hash;
    let mut i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u32;
        hash = hash.wrapping_mul(FNV_PRIME);
        i += 1;
    }
    hash
}

/// Stable [`ClapId`] derived from a parameter's `module` and `name`.
///
/// Hashes `module.name` (or just `name` when `module` is empty) with FNV-1a.
/// Because it is derived from the name rather than from a positional counter,
/// the id survives adding, removing or reordering parameters, which keeps
/// automation and saved state valid across plugin updates.
///
/// This is a `const fn`, so it can be evaluated at compile time for literal
/// modules and at runtime for indexed/array modules.
pub const fn param_id(module: &str, name: &str) -> ClapId {
    let mut hash = fnv1a(FNV_OFFSET_BASIS, module.as_bytes());
    if !module.is_empty() {
        hash = fnv1a(hash, b".");
    }
    hash = fnv1a(hash, name.as_bytes());
    ClapId::new(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_stable_and_distinguishing() {
        assert_eq!(param_id("left", "gain"), param_id("left", "gain"));
        assert_ne!(param_id("left", "gain"), param_id("right", "gain"));
        assert_ne!(param_id("left", "gain"), param_id("left", "pan"));
        assert_ne!(param_id("", "gain"), param_id("left", "gain"));
    }

    #[test]
    fn empty_module_hashes_bare_name() {
        assert_eq!(
            param_id("", "gain"),
            ClapId::new(fnv1a(FNV_OFFSET_BASIS, b"gain"))
        );
    }
}
