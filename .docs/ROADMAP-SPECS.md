# r_e_c_u_r (Rust port) — phase specs

Full specs per phase from the Execution Order table in `ROADMAP.md`. Each is a one-paragraph summary; the detailed design lives in `docs/superpowers/specs/`.

## 1 — r_e_c_u_r-core

**Spec**: [`docs/superpowers/specs/2026-05-12-recur-rust-phase1-core-design.md`](../docs/superpowers/specs/2026-05-12-recur-rust-phase1-core-design.md)

Single-binary Rust port of the core video sampler. GStreamer for decode (3-pipeline rotation: `last / current / next`). Per-slot loop in / out, rate, layer. Sampler modes: sequential, parallel, fixed-length, random-start. Banks of 10 slots each, persisted to `banks.toml`. Menus: Browser (file tree + slot annotation), Sampler (bank view), Settings (nested cycle-through). 17×48 char SPI LCD UI. Desktop input via `winit` keyboard (mac-friendly for local dev). GL composite output to HDMI/composite on Pi; window on desktop. Render backend, LCD status grid, and screen-stack framework copied from mandleROT.

**Done when**: `cargo test --lib` green; `cargo run -- --smoke-frames 60` plays the bundled SMPTE test clip; user can map → trigger → loop slots via keyboard; settings cycle persists; `cross build --features pi` runs on a real Pi.

## 2 — conjur (shader layer)

GLSL shader layer applied over the video sources. Each shader = one `.glsl` + `.toml` pair in `shaders/` (same scheme as mandleROT). Shader can read up to N video sources as `sampler2D` uniforms (`u_source_0`, `u_source_1`, …) plus the standard mandleROT uniforms (`u_time`, `u_resolution`, `u_audio.xyzw`, `u_param0..7`, etc.). New display modes: `SHADERS` (shader file browser) and `SHDR_BNK` (per-layer shader slot bank). Hot-reload via `notify`. Reuses mandleROT's `src/scene/` + `src/render/` shader-assembly pipeline.

**Depends on**: Phase 1 (need the player rack + composite output to feed shader inputs).

## 3 — detour (frame ring)

In-memory ring buffer of decoded RGBA frames (target ~500 frames at the configured render resolution; size-cap by megabytes, not frame count, so it scales sanely). Captures from `current` player. New display mode `FRAMES` shows: ring size, scrub position, start/end markers, mix amount, playback speed / direction (`detour_settings` from the original). New control mode `DetourScrub` re-maps inputs to scrub controls. Compose pass blends ring output with live `current` per `detour_mix`. Reuses Phase 1's GL composite.

**Depends on**: Phase 1.

## 4 — captur (live capture)

USB v4l2 / CSI live-capture as a video source, mappable into bank slots like a file. GStreamer source becomes `v4l2src device=/dev/videoN ! ...` instead of `uridecodebin uri=file://...`. Recording (`<REC>` indicator from the original) writes a file via `splitmuxsink` while still feeding the live preview. CSI camera supported via `libcamerasrc` on Pi 5 / Pi 4. Captur is intentionally near-last because the original notes it as fragile on Pi 3 B+ (USB-2 bus contention with Ethernet); we want the rest of the system rock-solid before adding it.

**Depends on**: Phase 1. Optional benefit from Phase 2 if shaders are used to clean up capture.

## 5 — Pi inputs (`pi` feature flag)

GPIO matrix scan for the original `i_n_c_u_r` PCB numpad via `rppal`. USB MIDI in via `midir` (note → SelectSlot, CC → CycleSetting / knob mapping). Analog pots via I2C ADC (ADS1115-class), feeding `RawEvent::Knob`. All behind `cargo build --features pi`; macOS / desktop builds keep `WinitSource` as the only input. `keymap.toml` extended with `[midi]` and `[gpio]` sections.

**Depends on**: Phase 1.
