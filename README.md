# rift

A library for building CLAP audio plugins in Rust, built on [clack](https://github.com/prokopyl/clack).

Early development: the API changes without notice and saved plugin state is not stable yet.

Current work is mostly the parameter system (host-visible params and host-invisible `#[persist]` state), the wrapper, and the vizia GUI. Open items are in [TODO.md](TODO.md).

## Tests

Run all tests and collect coverage:

```
cargo llvm-cov --workspace --lcov --output-path ./target/lcov.info
```

Run the tests and show coverage for a single package:

```
cargo llvm-cov -p <package-name>
```

HTML output:

```
cargo llvm-cov --workspace --html
```

## Notes

Rayon makes FL Studio crash when rebuilding a `.clap` without restarting FL Studio first. It is safe in release builds, but must not be used in dev builds.
