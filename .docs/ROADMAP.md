# r_e_c_u_r (Rust port) — Roadmap

Rust re-imagining of [langolierz/r_e_c_u_r](https://github.com/langolierz/r_e_c_u_r). Targets Pi 3 B+ / Pi 4 / Pi 5, with macOS + Linux x86_64 dev. Same render backend + LCD pathway as `/Users/cody/Dev/mandleROT`.

The original Python source is preserved at `.old/` (gitignored).

## Bugs / Blockers

_(none yet — project not started)_

## Recently Shipped

_(none yet)_

## Design Notes

- **Decoder**: GStreamer (`gstreamer-rs`). `uridecodebin ! videoconvert ! glupload ! glsinkbin`. On Pi `v4l2h264dec` auto-selects, dmabuf → EGLImage zero-copy.
- **Render**: copy mandleROT's `src/render/` verbatim. Same `desktop` / `pi` feature split (`glow` + `winit` + `glutin` vs `khronos-egl` + `gbm` + `drm`).
- **UI**: 17×48 amber text grid on SPI LCD. Reuse mandleROT's `src/status/` and `src/ui/` (Screen trait + ScreenStack).
- **Playback model**: 3 GStreamer pipelines rotated as `last / current / next` to hide load latency. Mirrors the Python original's player rotation.
- **State files**: TOML in `user_state_dir()` (precedence `$RECUR_STATE_DIR` → `<exec>/.config/recur/` → `./.config/recur/`).
- **Panic semantics**: `panic = "abort"`; systemd restarts on Pi. Esc / Backspace ×2 ≤ 400 ms = `Action::Panic` resets the rack.

## Execution Order

Each phase = its own design spec + implementation plan + ship cycle.

| ID | Phase | Status | Key files / dirs |
|---|---|---|---|
| 1 | **r_e_c_u_r-core** — file playback, sample bank, loop points, sampler modes, Browser/Sampler/Settings menus, desktop keyboard control | ☐ | `src/video/`, `src/sample/`, `src/menu/`, `src/input/winit_src.rs` |
| 2 | **conjur** — GLSL shader layer over video sources; scene system from mandleROT | ☐ | `src/shader/`, `shaders/`, `scenes/` |
| 3 | **detour** — in-memory frame ring (~500 frames), scrubbing, speed/direction control | ☐ | `src/detour/` |
| 4 | **captur** — USB v4l2 / CSI live-capture as a video source, slot-mapped | ☐ | `src/video/capture.rs` |
| 5 | **Pi inputs** — GPIO matrix (`i_n_c_u_r` PCB), USB MIDI, analog ADC over I2C (`pi` feature flag) | ☐ | `src/input/{gpio,midi,adc}.rs` |

Active phase = lowest-numbered row with ☐.

## Backlog

- `recur import-old-banks <dir>` — migrate from original Python `json_objects/`.
- Bench composite output to native composite TRRS jack on Pi 3 B+ vs Pi 5.
- Auto-discover `paths_to_browser` from common mount points (`/media/*`, USB).
- MIDI clock sync for `LoopType::Parallel` (line up loop restarts to bar).
