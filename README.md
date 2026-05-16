# r_e_c_u_r

Rust port of [langolierz/r_e_c_u_r](https://github.com/langolierz/r_e_c_u_r), a video sampler for Raspberry Pi. Targets Pi 3 B+ and Pi 5 as separate compile-time builds; macOS and Linux x86_64 supported for development.

## Install matrix

| Hardware | Recommended binary | Cargo invocation |
|---|---|---|
| Raspberry Pi 3 B+ | `recur-aarch64-pi3` | `cross build --release --no-default-features --features pi3 --target aarch64-unknown-linux-gnu` |
| Raspberry Pi 4 | `recur-aarch64-pi3` (works; not first-class) | same as Pi 3 B+ |
| Raspberry Pi 5 | `recur-aarch64-pi5` | `cross build --release --no-default-features --features pi5 --target aarch64-unknown-linux-gnu` |
| macOS / Linux x86_64 (dev) | `recur-x86_64-desktop` | `cargo build --release` |

The two Pi builds are mutually exclusive — choose `pi3` or `pi5` at build time. A `build.rs` mutex check fails fast if zero or multiple target features are enabled.

The legacy `pi` feature is retained as a deprecated alias for `pi3` for one release.

## Source format support

| Codec | pi3 | pi5 | desktop |
|---|---|---|---|
| H.264 | ✓ (hardware) | ✓ (hardware) | ✓ |
| H.265 / HEVC | ✗ | ✓ (hardware) | ✓ |
| VP9 | ✗ | ✓ | ✓ |
| AV1 | ✗ | ✓ (software) | ✓ |

Unsupported sources are visible in the Browser with an `[X]` marker but cannot be mapped into a slot. Re-encode with `ffmpeg -c:v libx264 -crf 20 -preset slow in.mkv out.mp4` if needed for `pi3`.

## Development

```sh
# desktop build (default features = pi5 shader parity)
cargo run

# desktop build emulating pi3 shader compatibility
cargo run -- --gles-profile pi3

# headless smoke run
cargo run -- --smoke-frames 60

# unit tests
cargo test --lib

# cross-build for Raspberry Pi
cross build --release --no-default-features --features pi3 --target aarch64-unknown-linux-gnu
cross build --release --no-default-features --features pi5 --target aarch64-unknown-linux-gnu
```

## License

MIT OR Apache-2.0.
