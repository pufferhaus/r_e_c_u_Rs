---
id: TASK-001
type: task
title: GPIO matrix input (i_n_c_u_r PCB)
status: planned
priority: medium
phase: 5
created: "2026-05-22"
updated: "2026-05-23"
blocked:
    reason: waiting for i_n_c_u_r PCB — hardware not yet built
---

GPIO matrix scan for the original `i_n_c_u_r` PCB numpad via `rppal`. Behind `cargo build --features pi`; macOS / desktop builds keep `WinitSource` as the only input. `keymap.toml` extended with `[gpio]` section.

No divergence between `pi3` and `pi5` — `rppal` code path is identical on both.
