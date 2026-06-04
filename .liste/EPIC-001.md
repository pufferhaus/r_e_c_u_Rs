---
id: EPIC-001
type: epic
title: Phase 5 — Pi inputs
status: planned
priority: medium
phase: 5
created: "2026-05-22"
updated: "2026-05-22"
links:
    - type: parent-of
      target: TASK-001
    - type: parent-of
      target: TASK-002
    - type: parent-of
      target: TASK-003
---

GPIO matrix scan for the original `i_n_c_u_r` PCB numpad via `rppal`. USB MIDI in via `midir` (note → SelectSlot, CC → CycleSetting / knob mapping). Analog pots via I2C ADC (ADS1115-class), feeding `RawEvent::Knob`. All behind `cargo build --features pi`; macOS / desktop builds keep `WinitSource` as the only input. `keymap.toml` extended with `[midi]` and `[gpio]` sections.

**Dual-target rules:** No divergence between `pi3` and `pi5`. `rppal`, `midir`, and the ADS1115 I²C ADC code path are identical on both Pis. `keymap.toml` shape unchanged.

**Depends on:** Phase 1.
