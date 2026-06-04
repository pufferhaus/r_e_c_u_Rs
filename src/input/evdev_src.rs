//! Pi USB-HID numpad input via `/dev/input/event*` using the `evdev` crate.
//!
//! Designed for a cheap 19-key USB numpad mounted rotated 90° CCW. Key mapping
//! is driven by `keymap.toml`: `[bindings]` (normal), `[fn_bindings]` (FN held),
//! plus the `detour`/`shdrbnk` auto-context overlays.
//!
//! The FN modifier is the **`.`/Del key held down** — KP_DOT with NumLock on,
//! Delete with it off (handled both). A clean tap of that key fires its normal
//! action (play/pause); holding it turns the next key(s) into FN-layer actions.
//! This replaces an earlier `000` triple-tap scheme that the production pad's
//! NumLock-wrapping firmware made unreliable. `×`/`÷` are unaffected by NumLock,
//! so FN-mode-navigation works regardless of the pad's NumLock state.

use std::time::{Duration, Instant};

use evdev::{Device, EventType, KeyCode};

use crate::action::Action;
use crate::input::keymap::Keymap;
use crate::state::{ControlMode, DisplayMode};

/// NumLock auto-wrap suppression window — some firmware issues a NumLock
/// press/release around digit keys. We drop NumLock events within this window
/// of a digit press.
const NUMLOCK_WRAP_WINDOW: Duration = Duration::from_millis(250);

pub struct EvdevSource {
    devices: Vec<Device>,
    keymap: Keymap,
    /// True while the FN modifier key (`.`/Del) is physically held.
    fn_held: bool,
    /// Set when another key is pressed while FN is held, so releasing the FN
    /// key doesn't also fire its tap action (play/pause).
    fn_consumed: bool,
    /// Timestamp of the last digit press (for NumLock wrap detection).
    last_digit_press: Option<Instant>,
    /// Deferred NumLock press awaiting wrap-pair classification.
    numlock_pending: Option<Instant>,
}

impl EvdevSource {
    pub fn empty() -> Self {
        Self {
            devices: Vec::new(),
            keymap: Keymap::default(),
            fn_held: false,
            fn_consumed: false,
            last_digit_press: None,
            numlock_pending: None,
        }
    }

    pub fn open_all(keymap: Keymap) -> crate::error::Result<Self> {
        let mut devices = Vec::new();
        for entry in std::fs::read_dir("/dev/input")? {
            let entry = entry?;
            let path = entry.path();
            if !path
                .file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.starts_with("event"))
                .unwrap_or(false)
            {
                continue;
            }
            match Device::open(&path) {
                Ok(mut d) => {
                    if d.supported_keys().is_some() {
                        let _ = d.set_nonblocking(true);
                        let _ = d.grab();
                        devices.push(d);
                    }
                }
                Err(e) => tracing::debug!("skip {path:?}: {e}"),
            }
        }
        if devices.is_empty() {
            return Err(crate::error::Error::Other("no input devices found".into()));
        }
        Ok(Self {
            devices,
            keymap,
            fn_held: false,
            fn_consumed: false,
            last_digit_press: None,
            numlock_pending: None,
        })
    }

    /// Non-blocking poll — returns translated `Action` values.
    pub fn poll(&mut self, mode: ControlMode, display_mode: DisplayMode) -> Vec<Action> {
        let mut out = Vec::new();
        let now = Instant::now();

        for dev in &mut self.devices {
            let events: Vec<_> = match dev.fetch_events() {
                Ok(it) => it.collect(),
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
                Err(e) => {
                    tracing::warn!("evdev fetch: {e}");
                    continue;
                }
            };
            for evt in events {
                if evt.event_type() != EventType::KEY {
                    continue;
                }
                let key = KeyCode::new(evt.code());
                let down = evt.value() == 1;
                let repeat = evt.value() == 2;

                // ── key release ──
                if !down && !repeat {
                    if is_fn_key(key) {
                        // Clean tap (no combo while held) → the normal `.`
                        // action (play/pause). If it was used as a modifier,
                        // swallow it.
                        if self.fn_held && !self.fn_consumed {
                            if let Some(a) =
                                self.keymap.resolve("NumpadDecimal", display_mode, mode)
                            {
                                out.push(a);
                            }
                        }
                        self.fn_held = false;
                    } else if let Some(slot) = kp_digit_slot(key) {
                        out.push(Action::SlotRelease(slot));
                    }
                    continue;
                }

                // ── FN modifier key (held) ──
                if is_fn_key(key) {
                    if down {
                        self.fn_held = true;
                        self.fn_consumed = false;
                    }
                    continue; // ignore repeats; never emits on press
                }

                // ── NumLock wrap suppression ──
                if key == KeyCode::KEY_NUMLOCK {
                    let trailing = self
                        .last_digit_press
                        .map(|t| now.duration_since(t) <= NUMLOCK_WRAP_WINDOW)
                        .unwrap_or(false);
                    if !trailing {
                        self.numlock_pending = Some(now);
                    }
                    continue;
                }

                // Arm wrap suppression on digit presses.
                if is_kp_digit(key) {
                    self.numlock_pending = None;
                    self.last_digit_press = Some(now);
                }

                let Some(raw) = key_to_raw(key) else {
                    tracing::debug!("evdev: unmapped key {:?}", key);
                    continue;
                };

                flush_pending_numlock(
                    &mut self.numlock_pending,
                    &mut out,
                    &self.keymap,
                    mode,
                    display_mode,
                );

                // FN held → resolve via the FN layer; otherwise the context-aware
                // base layer.
                let action = if self.fn_held {
                    self.fn_consumed = true;
                    self.keymap
                        .lookup_fn(raw)
                        .or_else(|| self.keymap.resolve(raw, display_mode, mode))
                } else {
                    self.keymap.resolve(raw, display_mode, mode)
                };
                if let Some(a) = action {
                    out.push(a);
                }
            }
        }

        // Age out a deferred NumLock that never paired with a digit.
        let nl_cutoff = now.checked_sub(NUMLOCK_WRAP_WINDOW);
        if let Some(t) = self.numlock_pending {
            if nl_cutoff.map(|c| t < c).unwrap_or(false) {
                if let Some(a) = self.keymap.resolve("NumLock", display_mode, mode) {
                    out.push(a);
                }
                self.numlock_pending = None;
            }
        }

        out
    }
}

fn flush_pending_numlock(
    pending: &mut Option<Instant>,
    out: &mut Vec<Action>,
    keymap: &Keymap,
    mode: ControlMode,
    display_mode: DisplayMode,
) {
    if pending.take().is_some() {
        if let Some(a) = keymap.resolve("NumLock", display_mode, mode) {
            out.push(a);
        }
    }
}

/// The FN modifier key: numpad `.` (KP_DOT, NumLock on) or Delete (NumLock off).
fn is_fn_key(k: KeyCode) -> bool {
    matches!(k, KeyCode::KEY_KPDOT | KeyCode::KEY_DELETE)
}

fn is_kp_digit(k: KeyCode) -> bool {
    matches!(
        k,
        KeyCode::KEY_KP0
            | KeyCode::KEY_KP1
            | KeyCode::KEY_KP2
            | KeyCode::KEY_KP3
            | KeyCode::KEY_KP4
            | KeyCode::KEY_KP5
            | KeyCode::KEY_KP6
            | KeyCode::KEY_KP7
            | KeyCode::KEY_KP8
            | KeyCode::KEY_KP9
    )
}

/// Physical digit of a KP key (for SlotRelease; apply only uses it to gate the
/// action-gated reload, so the exact value is informational).
fn kp_digit_slot(k: KeyCode) -> Option<u8> {
    Some(match k {
        KeyCode::KEY_KP0 => 0,
        KeyCode::KEY_KP1 => 1,
        KeyCode::KEY_KP2 => 2,
        KeyCode::KEY_KP3 => 3,
        KeyCode::KEY_KP4 => 4,
        KeyCode::KEY_KP5 => 5,
        KeyCode::KEY_KP6 => 6,
        KeyCode::KEY_KP7 => 7,
        KeyCode::KEY_KP8 => 8,
        KeyCode::KEY_KP9 => 9,
        _ => return None,
    })
}

fn key_to_raw(k: KeyCode) -> Option<&'static str> {
    Some(match k {
        KeyCode::KEY_KP0 => "Numpad0",
        KeyCode::KEY_KP1 => "Numpad1",
        KeyCode::KEY_KP2 => "Numpad2",
        KeyCode::KEY_KP3 => "Numpad3",
        KeyCode::KEY_KP4 => "Numpad4",
        KeyCode::KEY_KP5 => "Numpad5",
        KeyCode::KEY_KP6 => "Numpad6",
        KeyCode::KEY_KP7 => "Numpad7",
        KeyCode::KEY_KP8 => "Numpad8",
        KeyCode::KEY_KP9 => "Numpad9",
        KeyCode::KEY_KPPLUS => "NumpadAdd",
        KeyCode::KEY_KPMINUS => "NumpadSubtract",
        KeyCode::KEY_KPASTERISK => "NumpadMultiply",
        KeyCode::KEY_KPSLASH => "NumpadDivide",
        KeyCode::KEY_KPENTER => "NumpadEnter",
        KeyCode::KEY_BACKSPACE => "Backspace",
        KeyCode::KEY_NUMLOCK => "NumLock",
        _ => return None,
    })
}
