> **⚠️ Status: Archived / discontinued.**
> This was a library for building CLAP audio plugins in Rust.
> Development got pretty far and I was happy with it. but I've since found
> [**truce**](https://github.com/truce-audio/truce).
>
> The code below is left up as-is for reference. It is **not maintained** and
> the TODO list will not be completed.

---

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
