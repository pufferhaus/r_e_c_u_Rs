---
id: TASK-002
type: task
title: USB MIDI input
status: planned
priority: medium
phase: 5
created: "2026-05-22"
updated: "2026-05-23"
blocked:
    reason: Pi 5 available but not set up
---

USB MIDI in via `midir` (note → SelectSlot, CC → CycleSetting / knob mapping). Behind `cargo build --features pi`; macOS / desktop builds keep `WinitSource` as the only input. `keymap.toml` extended with `[midi]` section.

No divergence between `pi3` and `pi5` — `midir` code path is identical on both.
