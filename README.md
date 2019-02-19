# waved
Fast, modal wave editor

## Contributing
### Using live-reload
1. Install `cargo-watch`:
```
cargo install cargo-watch
```
2. Run the following command at the root of the repository (do not launch the process in a debugger, reloading will not work):
```
cargo run --features "live-reload"
```
3. Run the following cargo-watch commands in parallel in the `waved-core` directory:
```
cargo watch -w src -x build -s 'touch .trigger-watch'
cargo watch --no-gitignore -w .trigger-watch -s 'mv -f target/debug/libwaved_core.dylib ../target/debug/libwaved_core.dylib'
```

Make sure to use the right extension for your platform (`.so` on Linux, `.dylib` on macOS and `.dll` on Windows) and to remove the `lib` prefix on Windows.

Live-reload also works in release by simply replacing the occurrences of `debug` with `release` in the `cargo watch` commands, as well as adding the `--release` flag to the `cargo run` command.
