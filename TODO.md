# TODO

## General

- [ ] Might have too many things in base lib, look into featuring some stuff.
- [ ] Add tons of debug assert
- [ ] doc is too small. There are things that should be documented better
- [x] in audio consumer crate, it's easy to fuck up with All. Think of adding MonoConsumer
- [x] BoundedVec capacity is weird to save, capacity should always be defined by what's written in the plugin, not in the saved state.
- [x] Shared has a different meaning in clack and rift, yet the same name. Should change that because that's confusing asf.
- [x] Don't use naive generator with infinite frequencies (i.e. if phase > 0.5 { -1. } else { 1. }) has it would create serious issues with later effects, like filters. (resolution ... Just use fundsp xx)

## Parameters

- [ ] **Host-invisible persist notifications.** Persist changes must only go through `GuiParamEventKind::ValueLess`. `Value(_)` and the gesture variants are pushed to the host via `GuiParamEvent::maybe_to_raw()` (`crates/rift-plugin-wrapper/src/processor.rs:106`), so a persist id sent as `Value(persist_id, x)` would leak a `ParamValueEvent` for an id the host doesn't own. Add a dedicated, documented entry point (e.g. `GuiContext::persist_changed(id)`, or a distinct `GuiParamEvent` variant) so it is impossible to send a value/gesture for a persist id by accident.

- [ ] **Unique ids across params and persist.** Both params and persist fields use `param_id(module, name)`, but `ParamsWrapper::new` only panics on duplicate *param* ids (`crates/rift-plugin-params/src/params_wrapper.rs:20`). A persist field and a param sharing a module+name would collide, making the `id == params.some_data.id()` branch in `on_param_change` ambiguous. Validate uniqueness over the union in the derive (or hash persist ids with a distinct salt), and keep persist entries out of `all_params()` / `ParamCollection` so the host stays blind.

- [ ] **Notify the audio thread on state load.** `state.rs::load` writes params directly (atomics, the audio thread sees them) and now `deserialize_persist`, but nothing tells the audio thread that its derived persist state is stale. Push `ValueLess` notifications for the persisted ids on load, or use a dirty/generation flag checked in the process loop.

- [ ] **Redesign the persist API.** `Persistent::create`/identity (`name`/`module`/`path`/`id`) is inconsistent with the params `bon` builders and duplicate `Param`; it needs a rework that lets users shape their own construction API (and share identity with `Param` instead of copying it).

- [ ] **Richer `RangeMapping`.** Only `Linear` and `Skew` exist today; add the bread-and-butter mappings so plugins stop hand-rolling them: dB/exponential, bipolar, and reversed ranges, plus matching `value_to_text` formatting.

- [ ] **`BlobParam` (bytes).** Host-invisible `Persistent` sibling of `StringParam` for arbitrary binary data (wavetables, impulse responses, serialized graphs): id-keyed, saved in the `persist` section, likely backed by `ArcSwap<Arc<[u8]>>` so audio-thread reads stay lock-free and allocation-free.

- [ ] **Path / file param.** A `StringParam`-flavored value holding a file path, with a change notification so the plugin can reload (e.g. a sample or IR). Probably a thin flavor over `StringParam`/`BlobParam` rather than a new trait.
