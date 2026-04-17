### TODO 
- [ ] Might have to many things in base lib, look into featuring some stuf.
- [-] Add tons of debug assert
- [-] doc is too small. There are things that should be documented better
- [x] in audio consumer crate, it's easy to fuck up with All. Think of adding MonoConsumer
- [ ] BoundedVec capacity is weird to save, capacity should always be defined by what's written in the plugin, not in the saved state.
- [ ] Shared has a different meaning in clack and rift, yet the same name. Should change that because that's confusing asf.
- [ ] Don't use naive generator with infinite frequencies (i.e. if phase > 0.5 { -1. } else { 1. }) has it would create serious issues with later effects, like filters. 

## Tests
- Run all tests and collect coverage:
```
cargo llvm-cov --workspace --lcov --output-path ./target/lcov.info
```
- Run the test and show coverage on a single package
```
cargo llvm-cov -p <package-name>
```
- HTML Output
```
cargo llvm-cov --workspace --html
```

## Notes:
Rayon seems to make FLCrashes if rebuilding a new version of .CLAP without restarting FLStudio entirely. Using rayon in a release build is safe, but on dev it must not be used !!