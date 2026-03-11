### TODO 
- [ ] Might have to many things in base lib, look into featuring some stuf.

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
cargo llvm-cov --workspace --html --output-path ./target/cover.html
```