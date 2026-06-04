//! SamplerBody — shows 10 slots of the active bank with name/length/start/end.

use crate::action::Action;
use crate::state::{SharedState, Slot};
use crate::status::grid::TextGrid;
use crate::ui::{Screen, ScreenResult};

pub struct SamplerBody {
    selected: u8,
}

impl SamplerBody {
    pub fn new() -> Self {
        Self { selected: 0 }
    }
}

impl Default for SamplerBody {
    fn default() -> Self {
        Self::new()
    }
}

impl Screen for SamplerBody {
    fn render(&self, state: &SharedState, grid: &mut TextGrid) {
        use crate::menu::layout;
        use crate::status::grid::{ATTR_BRIGHT, ATTR_NORMAL};
        let bank = state.current_bank();
        layout::left_header(
            grid,
            &format!("{:>4} {:<17} {:>5} {:>5} {:<5}", "slot", "name", "len", "in", "out"),
        );
        let rec = state.active_recording.as_ref();
        for (i, opt) in bank.slots.iter().enumerate() {
            let is_playing = state.now_playing == Some((state.bank_number, i as u8));
            // `>` marks the playing slot; the rest of the slot label is 3-wide so
            // the total slot column still lines up under the header's 4-wide field.
            let marker = if is_playing { '>' } else { ' ' };
            let body = match opt {
                None => format!("{:^3} {:<17} {:>5} {:>5} {:<5}", i, "", "", "", ""),
                Some(s) => fmt_slot_row_with_record_state(i, s, rec),
            };
            let attr = if is_playing { ATTR_BRIGHT } else { ATTR_NORMAL };
            layout::left_row(grid, i, &format!("{marker}{body}"), attr);

            // Continuous blink on the playing row: invert on alternating ~0.3 s
            // half-periods for as long as the clip is the active one.
            const BLINK_HALF: u64 = 9;
            let flash_on = is_playing && (state.anim_frame / BLINK_HALF) % 2 == 0;
            if flash_on {
                layout::invert_left_row(grid, i);
            } else if i == self.selected as usize && !is_playing {
                // Cursor highlight (suppressed on the playing row, which already
                // stands out as bright + marked + blinking).
                layout::invert_left_row(grid, i);
            }
        }
    }

    fn handle(&mut self, action: Action, _state: &mut SharedState) -> ScreenResult {
        match action {
            Action::NavUp => {
                self.selected = self.selected.saturating_sub(1);
                ScreenResult::Continue
            }
            Action::NavDown => {
                self.selected = (self.selected + 1).min(9);
                ScreenResult::Continue
            }
            _ => ScreenResult::Continue,
        }
    }
}

#[cfg(test)]
fn fmt_slot_row(idx: usize, s: &Slot) -> String {
    fmt_slot_row_with_record_state(idx, s, None)
}

fn fmt_slot_row_with_record_state(
    idx: usize,
    s: &Slot,
    rec: Option<&crate::capture::recording::ActiveRecording>,
) -> String {
    let truncated: String = match &s.source {
        crate::state::SourceKind::File(_) => {
            let base = s.name.rsplit_once('.').map(|(a, _)| a).unwrap_or(&s.name);
            base.chars().take(17).collect()
        }
        crate::state::SourceKind::Capture(d) => {
            let is_recording_this = rec
                .filter(|r| {
                    r.device_path == d.path
                        && r.state == crate::capture::recording::RecState::Recording
                })
                .is_some();
            let prefix = if is_recording_this {
                "[cap][REC] "
            } else {
                "[cap] "
            };
            format!("{}{}", prefix, s.name).chars().take(17).collect()
        }
    };
    format!(
        "{:^3} {:<17} {:>5} {:>5} {:<5}",
        idx,
        truncated,
        fmt_time(s.length),
        fmt_time(s.start),
        fmt_time(s.end),
    )
}

fn fmt_time(s: f64) -> String {
    if s < 0.0 {
        return String::new();
    }
    let total = s as u64;
    let mm = total / 60;
    let ss = total % 60;
    format!("{:02}:{:02}", mm, ss)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::SourceKind;

    #[test]
    fn nav_clamps() {
        let mut b = SamplerBody::new();
        let mut st = SharedState::new();
        b.handle(Action::NavUp, &mut st);
        assert_eq!(b.selected, 0);
        for _ in 0..20 {
            b.handle(Action::NavDown, &mut st);
        }
        assert_eq!(b.selected, 9);
    }

    #[test]
    fn fmt_time_clamps_negative() {
        assert_eq!(fmt_time(-1.0), "");
        assert_eq!(fmt_time(65.0), "01:05");
    }

    #[test]
    fn fmt_slot_row_handles_non_ascii() {
        let s = Slot {
            source: SourceKind::File("/clips/日本語ビデオ.mp4".into()),
            name: "日本語ビデオ.mp4".into(),
            start: -1.0,
            end: -1.0,
            length: 0.0,
            rate: 1.0,
        };
        // Should not panic.
        let row = fmt_slot_row(7, &s);
        assert!(row.contains("7"));
    }

    #[test]
    fn capture_slot_renders_with_cap_marker_in_name_column() {
        use crate::capture::CaptureDevice;
        let s = Slot {
            source: SourceKind::Capture(CaptureDevice {
                path: "/dev/video0".into(),
                label: "v4l2:video0".into(),
            }),
            name: "v4l2:video0".into(),
            start: -1.0,
            end: -1.0,
            length: 0.0,
            rate: 1.0,
        };
        let row = fmt_slot_row(0, &s);
        assert!(row.contains("[cap]"), "got: {row}");
    }

    #[test]
    fn capture_slot_renders_with_rec_marker_when_recording() {
        use crate::capture::recording::{ActiveRecording, RecState};
        use crate::capture::CaptureDevice;
        use std::time::Instant;
        let s = Slot {
            source: SourceKind::Capture(CaptureDevice {
                path: "/dev/video0".into(),
                label: "v4l2:video0".into(),
            }),
            name: "v4l2:video0".into(),
            start: -1.0,
            end: -1.0,
            length: 0.0,
            rate: 1.0,
        };
        let row = fmt_slot_row_with_record_state(
            0,
            &s,
            Some(&ActiveRecording {
                device_path: "/dev/video0".into(),
                file_path: "/tmp/r.mp4".into(),
                started_at: Instant::now(),
                state: RecState::Recording,
                last_disk_check: Instant::now(),
            }),
        );
        assert!(row.contains("[cap]"), "row: {row}");
        assert!(row.contains("[REC]"), "row: {row}");
    }

    #[test]
    fn capture_slot_with_different_device_does_not_show_rec_marker() {
        use crate::capture::recording::{ActiveRecording, RecState};
        use crate::capture::CaptureDevice;
        use std::time::Instant;
        let s = Slot {
            source: SourceKind::Capture(CaptureDevice {
                path: "/dev/video0".into(),
                label: "v4l2:video0".into(),
            }),
            name: "v4l2:video0".into(),
            start: -1.0,
            end: -1.0,
            length: 0.0,
            rate: 1.0,
        };
        let row = fmt_slot_row_with_record_state(
            0,
            &s,
            Some(&ActiveRecording {
                device_path: "/dev/video1".into(),
                file_path: "/tmp/r.mp4".into(),
                started_at: Instant::now(),
                state: RecState::Recording,
                last_disk_check: Instant::now(),
            }),
        );
        assert!(row.contains("[cap]"), "row: {row}");
        assert!(
            !row.contains("[REC]"),
            "row should NOT contain [REC]: {row}"
        );
    }
}
