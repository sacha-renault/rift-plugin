### TODO 
- [ ] Might have to many things in base lib, look into featuring some stuf.
- [ ] Add tons of debug assert
- [ ] doc is too small. There are things that should be documented better

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