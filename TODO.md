# TODO

## General

- [ ] Might have too many things in base lib, look into featuring some stuff.
- [ ] Add tons of debug assert
- [ ] doc is too small. There are things that should be documented better
- [x] in audio consumer crate, it's easy to fuck up with All. Think of adding MonoConsumer
- [x] BoundedVec capacity is weird to save, capacity should always be defined by what's written in the plugin, not in the saved state.
- [x] Shared has a different meaning in clack and rift, yet the same name. Should change that because that's confusing asf.
- [x] Don't use naive generator with infinite frequencies (i.e. if phase > 0.5 { -1. } else { 1. }) has it would create serious issues with later effects, like filters. (resolution ... Just use fundsp xx)
- [ ] Need to add a way to do background jobs. (usage: a param has changed and a reset / init requires allocation and therefore needs to be processed on background task.)
- [x] Harden the graveyard (RT-safe off-thread drop). `crates/rift-plugin-utils/src/graveyard.rs`: `drop_rt_safe` now `debug_assert!`s the graveyard is initialized, and tests cover: destructors run off the caller thread, concurrent producers, values exactly at the `SLOT_SIZE` / `SLOT_ALIGN` limits (plus `compile_fail` doctests for over-size / over-align), a full queue and an uninitialized graveyard both leaking, and an `assert_no_alloc` run. The routing decision is factored into `dispose`, so the leak paths are tested without touching the global.
- [ ] Run the graveyard tests under Miri. Needs a nightly toolchain and a CI job; the unsafe slot write/drop is exactly what Miri should be checking.
- [ ] Find a way graveyard can be init with user defined value.

## Parameters

- [x] Enum text formatting: `ParamKind::write_text` now lets an enum print its variant name (was showing the index).
- [ ] Persist notifications the host must not see. Persist changes must only go through `GuiParamEventKind::ValueLess`. A `Value(_)` or gesture for a persist id leaks a `ParamValueEvent` through `GuiParamEvent::maybe_to_raw()` (`crates/rift-plugin-wrapper/src/processor.rs:106`), for an id the host does not own. Add a dedicated entry point, for example `GuiContext::persist_changed(id)`.
- [ ] Unique ids across params and persist. Both use `param_id(module, name)`, but `ParamsWrapper::new` only panics on duplicate param ids (`crates/rift-plugin-params/src/params_wrapper.rs:20`). A persist field and a param can collide and make `on_param_change` ambiguous. Validate the union in the derive and keep persist out of `all_params()`.
- [ ] Notify the audio thread on state load. `state.rs::load` writes params and persist directly, so the audio thread sees them but nothing tells it the derived persist state changed. Push `ValueLess` for the persisted ids on load, or set a dirty flag the process loop checks.
- [ ] Redesign the persist API. `Persistent::create` and its identity methods duplicate `Param` and do not match the `bon` builders. Rework it so users can shape their own construction API and share identity with `Param`.
- [ ] Richer `Scale`. `Linear`, `Skew` and `Exponential` exist today, reachable as `range = linear(min, max)`, `range = skew(min, max, factor)` and `range = exp(min, max, factor)`. Add dB, bipolar and reversed ranges, plus matching `value_to_text`.
- [ ] Validate `Scale` at construction. `Scale` is a public enum, so a zero, negative or infinite factor compiles and makes `normalize` / `denormalize` return `inf` or `NaN` (`crates/rift-plugin-params/src/scale.rs`). Make a bad curve unrepresentable with a private field and `const fn` constructors that assert, and guard `min == max` separately since the assert cannot see the range. The derive already covers literal factors, but not a `const VALUE: f32`.
- [x] `BlobParam` (bytes). Host invisible `Persistent` sibling of `StringParam` for binary data such as wavetables, IRs or serialized graphs. Back it with `ArcSwap<Arc<[u8]>>` so audio thread reads stay lock free.
- [x] Path or file param. A `StringParam` flavour holding a path, with a change notification so the plugin can reload, for example a sample or an IR. Probably a thin flavour over `StringParam` or `BlobParam`.
