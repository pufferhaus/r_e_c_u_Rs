//! Pi USB-HID numpad input via `/dev/input/event*` using the `evdev` crate.
//!
//! Designed for a cheap 19-key USB numpad mounted rotated 90° CCW. Key mapping
//! is driven by `keymap.toml` `[bindings]` (normal layer) and `[fn_bindings]`
//! (FN layer). The FN layer is toggled by a triple-tap of `KP0` within 120 ms.

use std::time::{Duration, Instant};

use evdev::{Device, EventType, KeyCode};

use crate::action::Action;
use crate::input::keymap::Keymap;
use crate::state::{ControlMode, DisplayMode};

const KP0_BURST_WINDOW: Duration = Duration::from_millis(120);

/// After a `000` burst the FN modifier stays armed for this long so the
/// follow-up key press lands as an FN action even with human latency.
const FN_HOLD: Duration = Duration::from_millis(600);

/// NumLock auto-wrap suppression window — some firmware issues a NumLock
/// press/release around certain digit keys. We suppress NumLock events that
/// arrive within this window of a digit press.
const NUMLOCK_WRAP_WINDOW: Duration = Duration::from_millis(250);

pub struct EvdevSource {
    devices: Vec<Device>,
    keymap: Keymap,
    /// Triple-tap KP0 burst timestamps.
    kp0_pending: Vec<Instant>,
    /// FN modifier deadline. `Some(t)` while active, consumed on first use.
    fn_until: Option<Instant>,
    /// Timestamp of last digit/kp0 press (for NumLock wrap detection).
    last_digit_press: Option<Instant>,
    /// Deferred NumLock press waiting for wrap-pair classification.
    numlock_pending: Option<Instant>,
}

impl EvdevSource {
    pub fn empty() -> Self {
        Self {
            devices: Vec::new(),
            keymap: Keymap::default(),
            kp0_pending: Vec::new(),
            fn_until: None,
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
            kp0_pending: Vec::new(),
            fn_until: None,
            last_digit_press: None,
            numlock_pending: None,
        })
    }

    /// Non-blocking poll — returns translated `Action` values, same contract as `WinitSource`.
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

                // Handle key-up for slot-release and FN-release events.
                if !down && !repeat {
                    match key {
                        KeyCode::KEY_KP1
                        | KeyCode::KEY_KP2
                        | KeyCode::KEY_KP3
                        | KeyCode::KEY_KP4
                        | KeyCode::KEY_KP5
                        | KeyCode::KEY_KP6
                        | KeyCode::KEY_KP7
                        | KeyCode::KEY_KP8
                        | KeyCode::KEY_KP9 => {
                            if let Some(slot) = kp_digit(key) {
                                out.push(Action::SlotRelease(slot));
                            }
                        }
                        KeyCode::KEY_KP0 => out.push(Action::SlotRelease(0)),
                        _ => {}
                    }
                    continue;
                }

                // Press or repeat.
                match key {
                    KeyCode::KEY_NUMLOCK => {
                        let trailing = self
                            .last_digit_press
                            .map(|t| now.duration_since(t) <= NUMLOCK_WRAP_WINDOW)
                            .unwrap_or(false);
                        if !trailing {
                            self.numlock_pending = Some(now);
                        }
                        continue;
                    }
                    KeyCode::KEY_KP0 => {
                        let cutoff = now.checked_sub(KP0_BURST_WINDOW);
                        self.kp0_pending
                            .retain(|t| cutoff.map(|c| *t >= c).unwrap_or(true));
                        self.kp0_pending.push(now);
                        self.numlock_pending = None;
                        self.last_digit_press = Some(now);
                        if self.kp0_pending.len() >= 3 {
                            // `000` burst → arm the FN layer for the next key.
                            // One-shot (consumed by the next key or the timeout).
                            // We deliberately do NOT emit ToggleFunction: the FN
                            // layer is resolved via `fn_bindings`, whereas
                            // `function_on` drives the (desktop Shift) map path.
                            self.fn_until = Some(now + FN_HOLD);
                            self.kp0_pending.clear();
                        }
                        continue;
                    }
                    KeyCode::KEY_KP1
                    | KeyCode::KEY_KP2
                    | KeyCode::KEY_KP3
                    | KeyCode::KEY_KP4
                    | KeyCode::KEY_KP5
                    | KeyCode::KEY_KP6
                    | KeyCode::KEY_KP7
                    | KeyCode::KEY_KP8
                    | KeyCode::KEY_KP9
                    | KeyCode::KEY_KPDOT => {
                        self.numlock_pending = None;
                        self.last_digit_press = Some(now);
                    }
                    _ => {}
                }

                let Some(raw) = key_to_raw(key) else {
                    tracing::debug!("evdev: unmapped key {:?}", key);
                    continue;
                };

                // Flush stale KP0 presses as SelectSlot(0) before processing this key.
                flush_pending_kp0(&mut self.kp0_pending, &mut self.fn_until, &mut out);
                flush_pending_numlock(
                    &mut self.numlock_pending,
                    &mut out,
                    &self.keymap,
                    mode,
                    display_mode,
                );

                // FN layer applies to ANY key (digit or operator) while armed.
                let use_fn = self.fn_until.map(|t| t > now).unwrap_or(false);
                let action = if use_fn {
                    self.fn_until = None; // one-shot: consume the arm
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

        // Age out stale KP0 presses → SelectSlot(0).
        let flush_cutoff = now.checked_sub(KP0_BURST_WINDOW);
        let aged_count = self
            .kp0_pending
            .iter()
            .take_while(|t| flush_cutoff.map(|c| **t < c).unwrap_or(false))
            .count();
        for _ in 0..aged_count {
            self.fn_until = None;
            out.push(Action::SelectSlot(0));
        }
        self.kp0_pending.drain(..aged_count);

        // Age out stale NumLock → look up binding if any.
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

fn flush_pending_kp0(
    pending: &mut Vec<Instant>,
    fn_until: &mut Option<Instant>,
    out: &mut Vec<Action>,
) {
    for _ in 0..pending.len() {
        out.push(Action::SelectSlot(0));
    }
    pending.clear();
    *fn_until = None;
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

fn kp_digit(k: KeyCode) -> Option<u8> {
    match k {
        KeyCode::KEY_KP1 => Some(1),
        KeyCode::KEY_KP2 => Some(2),
        KeyCode::KEY_KP3 => Some(3),
        KeyCode::KEY_KP4 => Some(4),
        KeyCode::KEY_KP5 => Some(5),
        KeyCode::KEY_KP6 => Some(6),
        KeyCode::KEY_KP7 => Some(7),
        KeyCode::KEY_KP8 => Some(8),
        KeyCode::KEY_KP9 => Some(9),
        _ => None,
    }
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
        KeyCode::KEY_KPDOT => "NumpadDecimal",
        KeyCode::KEY_KPENTER => "NumpadEnter",
        KeyCode::KEY_BACKSPACE => "Backspace",
        KeyCode::KEY_NUMLOCK => "NumLock",
        _ => return None,
    })
}
