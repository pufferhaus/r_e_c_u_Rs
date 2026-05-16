# r_e_c_u_r (Rust port) — Roadmap

Rust re-imagining of [langolierz/r_e_c_u_r](https://github.com/langolierz/r_e_c_u_r). Targets Pi 3 B+ / Pi 4 / Pi 5, with macOS + Linux x86_64 dev. Same render backend + LCD pathway as `/Users/cody/Dev/mandleROT`.

The original Python source is preserved at `.old/` (gitignored).

## Bugs / Blockers

_(none)_

## Recently Shipped

- **Phase 2 sub-plan B — conjur UI + persistence** (2026-05-16): SHADERS browser, SHDR_BNK shader bank, `shader_banks.toml`, `--gles-profile` CLI flag, hot-reload via `notify`, 4 starter shaders (color_shift, pixelate, kaleidoscope, rgb_glitch). Codec probe remains in sub-plan C.
- **Phase 2 sub-plan A — shader infrastructure** (2026-05-16): `src/shader/` module, GLES-split preludes, shader_assembly, ShaderPipeline (FBO + compile cache), wired into both render backends, passthrough demo. See `docs/superpowers/specs/2026-05-16-conjur-design.md`.
- **Dual-target spec** (2026-05-16): pi3 + pi5 cargo features, per-shader GLES gating rules, byte-budgeted detour ring rules, desktop dev defaults to pi5 parity. Foundation only — phase-2/3 implementation pending. See `docs/superpowers/specs/2026-05-16-pi5-target-revision-design.md`.

## Design Notes

- **Decoder**: GStreamer (`gstreamer-rs`). `uridecodebin ! videoconvert ! glupload ! glsinkbin`. On Pi `v4l2h264dec` auto-selects, dmabuf → EGLImage zero-copy.
- **Render**: copy mandleROT's `src/render/` verbatim. `desktop` / `pi3` / `pi5` feature split, sharing the `pi-base` deps (`khronos-egl` + `gbm` + `drm`) on both Pi targets. Desktop uses `glow` + `winit` + `glutin`.
- **UI**: 17×48 amber text grid on SPI LCD. Reuse mandleROT's `src/status/` and `src/ui/` (Screen trait + ScreenStack).
- **Playback model**: 3 GStreamer pipelines rotated as `last / current / next` to hide load latency. Mirrors the Python original's player rotation.
- **State files**: TOML in `user_state_dir()` (precedence `$RECUR_STATE_DIR` → `<exec>/.config/recur/` → `./.config/recur/`).
- **Panic semantics**: `panic = "abort"`; systemd restarts on Pi. Esc / Backspace ×2 ≤ 400 ms = `Action::Panic` resets the rack.
- **Targets**: `pi3` (baseline, original r_e_c_u_r replacement) and `pi5` (forward path). Compile-time feature split; `pi-base` shared deps; `build.rs` enforces exactly-one-of. Deprecated `pi` alias maps to `pi3` for one release.

## Execution Order

Each phase = its own design spec + implementation plan + ship cycle.

| ID | Phase | Status | Key files / dirs |
|---|---|---|---|
| 1 | **r_e_c_u_r-core** — file playback, sample bank, loop points, sampler modes, Browser/Sampler/Settings menus, desktop keyboard control | ✅ | `src/video/`, `src/sample/`, `src/menu/`, `src/input/winit_src.rs` |
| 2 | **conjur** — GLSL shader layer over video sources; scene system from mandleROT. pi3 (GLSL 1.00) + pi5 (GLSL 3.10) per-shader gating. *Sub-plans A + B shipped; C (codec probe) pending* | ◐ | `src/shader/`, `shaders/`, `src/menu/{shaders,shdr_bnk,param}.rs` |
| 3 | **detour** — in-memory frame ring (~500 frames), scrubbing, speed/direction control | ☐ | `src/detour/` |
| 4 | **captur** — USB v4l2 / CSI live-capture as a video source, slot-mapped | ☐ | `src/video/capture.rs` |
| 5 | **Pi inputs** — GPIO matrix (`i_n_c_u_r` PCB), USB MIDI, analog ADC over I2C (`pi-base` feature) | ☐ | `src/input/{gpio,midi,adc}.rs` |

Active phase = lowest-numbered row with ☐.

## Backlog

- `recur import-old-banks <dir>` — migrate from original Python `json_objects/`.
- Bench composite output to native composite TRRS jack on Pi 3 B+ vs Pi 5.
- Auto-discover `paths_to_browser` from common mount points (`/media/*`, USB).
- MIDI clock sync for `LoopType::Parallel` (line up loop restarts to bar).
