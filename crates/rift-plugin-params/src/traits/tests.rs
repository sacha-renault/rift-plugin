use clack_plugin::utils::ClapId;

use crate::range::FloatRange;
use crate::{FloatParam, Param, ParamPtr, UserParams};

/// A hand-written stand-in for a `#[derive(Params)]` struct.
struct MockParams {
    gain: FloatParam,
    level: FloatParam,
}

impl UserParams for MockParams {
    fn all_params(&self) -> Vec<ParamPtr> {
        vec![self.gain.as_ptr(), self.level.as_ptr()]
    }
}

fn mock() -> MockParams {
    let build = |id: u32, name: &str| {
        FloatParam::builder()
            .id(ClapId::new(id))
            .name(name.to_string())
            .range(FloatRange::linear(0.0, 1.0))
            .build()
    };

    MockParams {
        gain: build(1, "gain"),
        level: build(2, "level"),
    }
}

#[test]
fn params_roundtrip() {
    let params = mock();
    params.gain.set_plain(0.25);
    params.level.set_plain(0.75);

    let mut section = serde_json::Map::new();
    params.serialize_params(&mut section).unwrap();

    let restored = mock();
    restored.deserialize_params(&section).unwrap();

    assert_eq!(restored.gain.plain(), 0.25);
    assert_eq!(restored.level.plain(), 0.75);
}

#[test]
fn missing_and_unknown_keys_are_tolerated() {
    let params = mock();
    params.level.set_plain(0.7);

    // Key "1" (gain) is present, key "2" (level) is absent, and "999" is unknown.
    let section: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(r#"{"1": 0.5, "999": 0.123}"#).unwrap();

    params.deserialize_params(&section).unwrap();

    assert_eq!(params.gain.plain(), 0.5);
    // An absent key leaves the parameter untouched rather than resetting it.
    assert_eq!(params.level.plain(), 0.7);
}

#[test]
fn persist_sections_default_to_empty_noops() {
    let params = mock();
    params.gain.set_plain(0.42);

    let mut section = serde_json::Map::new();
    params.serialize_persist(&mut section).unwrap();
    assert!(section.is_empty());

    // Deserializing an unrelated section is a no-op and does not error.
    let section: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(r#"{"anything": 1}"#).unwrap();
    params.deserialize_persist(&section).unwrap();
    assert_eq!(params.gain.plain(), 0.42);
}
