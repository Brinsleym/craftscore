# Building

See [`docs/development.md`](https://github.com/Brinsleym/enscore-cleanroom/blob/main/docs/development.md) for prerequisites, the build commands, fonts,
the web and mobile builds, and fast test loops. In short, once the workspace exists:

```sh
cargo run --release -p scorecraft -- score.pdf
cargo test --workspace
cargo xtask ci
```
