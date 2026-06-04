//! FRAMES body — ring stats + scrub readouts. Reads everything from
//! `SharedState.detour` + `SharedState.frames_stats_*`.

use crate::action::Action;
use crate::state::SharedState;
use crate::status::grid::TextGrid;
use crate::ui::{Screen, ScreenResult};

pub struct FramesBody;

impl FramesBody {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FramesBody {
    fn default() -> Self {
        Self::new()
    }
}

impl Screen for FramesBody {
    fn render(&self, state: &SharedState, grid: &mut TextGrid) {
        let d = &state.detour;
        let fps = state.frames_stats_fps.max(1);
        let scrub_age_s = (state
            .frames_stats_count
            .saturating_sub(1)
            .saturating_sub(d.read_position) as f32)
            / fps as f32;
        let marker_str = match (d.start_marker, d.end_marker) {
            (Some(a), Some(b)) => format!("[{a}..{b}]"),
            (Some(a), None) => format!("[{a}..]"),
            (None, Some(b)) => format!("[..{b}]"),
            _ => "[none]".to_string(),
        };
        use crate::menu::layout;
        layout::left_header(grid, "DETOUR RING");
        let n = crate::status::grid::ATTR_NORMAL;
        layout::left_row(
            grid,
            0,
            &format!(
                "ring: {}/{} frames ({}/{} MB)",
                state.frames_stats_count,
                state.frames_stats_capacity,
                state.frames_stats_used_mb,
                state.frames_stats_budget_mb
            ),
            n,
        );
        layout::left_row(
            grid,
            1,
            &format!(
                "scrub: frame {}/{} ({:.2}s ago)",
                d.read_position,
                state.frames_stats_count.saturating_sub(1),
                scrub_age_s
            ),
            n,
        );
        layout::left_row(
            grid,
            2,
            &format!(
                "speed: {:.2}x  dir: {}  mix: {:.0}%",
                d.speed,
                if d.forward { "fwd" } else { "rev" },
                d.mix * 100.0
            ),
            n,
        );
        layout::left_row(grid, 3, &format!("markers: {}", marker_str), n);
        layout::left_row(
            grid,
            4,
            &format!("auto-play: {}", if d.auto_play { "ON" } else { "off" }),
            n,
        );
    }

    fn handle(&mut self, _action: Action, _state: &mut SharedState) -> ScreenResult {
        // All scrub keys go through apply.rs (via keymap → Action::Detour*).
        ScreenResult::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_shows_ring_count_and_capacity() {
        use crate::menu::layout;
        let body = FramesBody::new();
        let mut st = SharedState::new();
        st.frames_stats_count = 34;
        st.frames_stats_capacity = 100;
        st.frames_stats_used_mb = 50;
        st.frames_stats_budget_mb = 128;
        let mut grid = TextGrid::new(layout::COLS, layout::ROWS);
        body.render(&st, &mut grid);
        let row = grid.row_text(layout::ROW_BODY0 + 1); // ring = list row 0
        assert!(row.contains("34/100"), "got: {row}");
        assert!(row.contains("50/128"), "got: {row}");
    }

    #[test]
    fn render_shows_auto_play_state() {
        use crate::menu::layout;
        let body = FramesBody::new();
        let mut st = SharedState::new();
        st.detour.auto_play = true;
        let mut grid = TextGrid::new(layout::COLS, layout::ROWS);
        body.render(&st, &mut grid);
        let row = grid.row_text(layout::ROW_BODY0 + 1 + 4); // auto-play = list row 4
        assert!(row.contains("ON"), "got: {row}");
    }

    #[test]
    fn render_shows_speed_and_direction() {
        use crate::menu::layout;
        let body = FramesBody::new();
        let mut st = SharedState::new();
        st.detour.speed = 2.0;
        st.detour.forward = false;
        st.detour.mix = 0.75;
        let mut grid = TextGrid::new(layout::COLS, layout::ROWS);
        body.render(&st, &mut grid);
        let row = grid.row_text(layout::ROW_BODY0 + 1 + 2); // speed = list row 2
        assert!(row.contains("2.00x"), "got: {row}");
        assert!(row.contains("rev"), "got: {row}");
        assert!(row.contains("75%"), "got: {row}");
    }
}
