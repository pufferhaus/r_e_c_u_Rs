---
id: TASK-003
type: task
title: Analog ADC over I2C (pi-base feature)
status: planned
priority: medium
phase: 5
created: "2026-05-22"
updated: "2026-05-23"
blocked:
    reason: Pi 5 available but not set up; also needs ADC wiring
---

Analog pots via I2C ADC (ADS1115-class), feeding `RawEvent::Knob`. Behind `cargo build --features pi`; macOS / desktop builds keep `WinitSource` as the only input.

No divergence between `pi3` and `pi5` — the ADS1115 I²C ADC code path is identical on both Pis.
