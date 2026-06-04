# r_e_c_u_r — USB Numpad Reference

The Pi build is driven by a cheap 19-key USB numpad, mounted **rotated 90° CCW**.
All bindings live in `keymap.toml` (`[bindings]`, `[fn_bindings]`,
`[detour_bindings]`, `[shdrbnk_bindings]`).

## Physical layout & slot order

Slots follow **physical reading order** — top-left key is slot 1, across and
down to slot 9, the lone key is slot 0. The slot a key fires is *not* the digit
printed on it:

```
operator view (pad rotated 90° CCW):

   ×    9    6    3    .          ×/÷  = nav,  −/+ = bank,  . = play
   ÷    8    5    2   000         000  = FN (triple-tap 0)
 NumLk  7    4    1    0          Enter / Bksp on the top edge

   digit key:   9 6 3  8 5 2  7 4 1  0
   fires slot:  1 2 3  4 5 6  7 8 9  0
```

## Normal layer

| Key | Action |
|---|---|
| digit keys | trigger video slot (reading order above) |
| `×` | nav up |
| `/` | nav down |
| `−` | previous bank |
| `+` | next bank |
| `.` | play / pause |
| `Enter` | enter / confirm / map (in Browser) |
| `Bksp` | back |
| `NumLk` | now / next player |

## FN layer — hold `000` (triple-tap `0`), then one key

| Key | Action | Key | Action |
|---|---|---|---|
| slot 1 (`9`) | → Sampler | slot 7 (`7`) | loop in |
| slot 2 (`6`) | → Browser | slot 8 (`4`) | clear loop |
| slot 3 (`3`) | → Settings | slot 9 (`1`) | loop out |
| slot 4 (`8`) | → Shaders | slot 0 (`0`) | add capture |
| slot 5 (`5`) | → Shader bank | `×` | seek forward |
| slot 6 (`2`) | → Detour | `/` | seek back |
| `−` | feedback toggle | `+` | record toggle |
| `.` | reload | `Enter` | detour exit |
| `Bksp` | **PANIC** (reset all players) | | |

The FN arm is one-shot: it applies to the very next key, then clears (≈600 ms
timeout if no key follows).

## Auto-context layers

The digit keys change meaning automatically by screen, so every function is
reachable without extra keys.

### In Detour (scrub) — digits become scrub controls

| Key | Action | Key | Action |
|---|---|---|---|
| slot 4 (`8`) | scrub −1 | slot 6 (`2`) | scrub +1 |
| slot 8 (`4`) | cycle speed | slot 2 (`6`) | toggle direction |
| slot 7 (`7`) | set start marker | slot 9 (`1`) | set end marker |
| slot 5 (`5`) | clear markers | slot 1 (`9`) | cycle mix |
| slot 3 (`3`) / `.` | toggle scrub play | `Enter` | exit detour |

(`×`/`/` still nav, `−`/`+` still bank.)

### In Shader bank — digits trigger shader slots

| Key | Action |
|---|---|
| digit keys | trigger shader slot (reading order) |
| `×` / `/` | navigate shader slots |

## Mapping a clip

1. `000`+`6` → Browser.
2. `×` / `/` to highlight a file.
3. `Enter` → maps it into the first empty slot of the current bank.

## Notes

- `NumLock` on some cheap pads is firmware-flaky; the driver suppresses the
  spurious NumLock presses that wrap digit keys.
- This mapping is verified end-to-end in `tests/sim.rs`
  (`simulate_numpad_mapping`).
