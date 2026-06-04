//! Always-on chrome screen. Draws the bordered 80×26 frame (top bar, transport,
//! NOW divider, right-pane mode menu, hotkey bar) via `layout`, then delegates
//! the left pane to the active mode body.

use crate::action::Action;
use crate::state::{ControlMode, DisplayMode, SharedState};
use crate::status::grid::TextGrid;
use crate::ui::{Screen, ScreenResult};

use super::layout;
use super::{
    browser::BrowserBody, frames::FramesBody, param::ParamBody, sampler::SamplerBody,
    settings::SettingsBody, shaders::ShadersBody, shdr_bnk::ShdrBnkBody,
};

pub struct RootScreen {
    browser: BrowserBody,
    frames: FramesBody,
    sampler: SamplerBody,
    settings: SettingsBody,
    shaders: ShadersBody,
    shdr_bnk: ShdrBnkBody,
    param: ParamBody,
}

impl RootScreen {
    pub fn new() -> Self {
        Self {
            browser: BrowserBody::new(),
            frames: FramesBody::new(),
            sampler: SamplerBody::new(),
            settings: SettingsBody::new(),
            shaders: ShadersBody::new(Vec::new(), 0),
            shdr_bnk: ShdrBnkBody::new(),
            param: ParamBody::new(),
        }
    }

    /// Refresh the SHADERS browser list after a shader library reload.
    /// Called from `main.rs` via Task 16; method is wired here in Task 10.
    pub fn set_shader_names(&mut self, names: Vec<String>, filtered: usize) {
        self.shaders = ShadersBody::new(names, filtered);
    }

    fn render_chrome(&self, state: &SharedState, grid: &mut TextGrid) {
        layout::draw_chrome(state, grid);
    }
}

impl Default for RootScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl Screen for RootScreen {
    fn render(&self, state: &SharedState, grid: &mut TextGrid) {
        self.render_chrome(state, grid);
        match state.display_mode {
            DisplayMode::Browser => self.browser.render(state, grid),
            DisplayMode::Sampler => self.sampler.render(state, grid),
            DisplayMode::Settings => self.settings.render(state, grid),
            DisplayMode::Shaders => self.shaders.render(state, grid),
            DisplayMode::ShdrBnk => self.shdr_bnk.render(state, grid),
            DisplayMode::Frames => self.frames.render(state, grid),
        }
        if state.control_mode == ControlMode::ShaderParam {
            self.param.render(state, grid);
        }
    }

    fn handle(&mut self, action: Action, state: &mut SharedState) -> ScreenResult {
        if state.control_mode == ControlMode::ShaderParam {
            return self.param.handle(action, state);
        }
        match state.display_mode {
            DisplayMode::Browser => self.browser.handle(action, state),
            DisplayMode::Sampler => self.sampler.handle(action, state),
            DisplayMode::Settings => self.settings.handle(action, state),
            DisplayMode::Shaders => self.shaders.handle(action, state),
            DisplayMode::ShdrBnk => self.shdr_bnk.handle(action, state),
            DisplayMode::Frames => self.frames.handle(action, state),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu::layout;
    use crate::status::grid::ATTR_INVERSE;

    fn new_grid() -> TextGrid {
        TextGrid::new(layout::COLS, layout::ROWS)
    }

    /// Grid row index of left-pane list row `i`.
    fn list_row(i: usize) -> usize {
        layout::ROW_BODY0 + 1 + i
    }

    #[test]
    fn hotkeys_show_gles_profile_indicator_when_v100() {
        use crate::render::shader_assembly::GlesProfile;
        let mut st = SharedState::new();
        st.gles_profile = GlesProfile::V100;
        st.display_mode = DisplayMode::Sampler;
        let root = RootScreen::new();
        let mut grid = new_grid();
        root.render(&st, &mut grid);
        let row = grid.row_text(layout::ROW_HOTKEYS);
        assert!(row.contains("pi3"), "hotkey bar should call out pi3, got: {row}");
    }

    #[test]
    fn shdr_bnk_mode_renders_shdr_bnk_body() {
        use crate::shader::ShaderSlot;
        let mut st = SharedState::new();
        st.display_mode = DisplayMode::ShdrBnk;
        st.current_shader_bank_mut().slots[2] = Some(ShaderSlot {
            shader: "kaleidoscope".into(),
            params: [0.0; 8],
        });
        let root = RootScreen::new();
        let mut grid = new_grid();
        root.render(&st, &mut grid);
        // Slot 2 → left-pane list row 2.
        let row = grid.row_text(list_row(2));
        assert!(row.contains("kaleidoscope"), "got: {row}");
    }

    #[test]
    fn hotkeys_show_rec_when_active_recording() {
        use crate::capture::recording::{ActiveRecording, RecState};
        use std::time::Instant;
        let mut s = SharedState::new();
        s.active_recording = Some(ActiveRecording {
            device_path: "/dev/video0".into(),
            file_path: "/tmp/rec.mp4".into(),
            started_at: Instant::now(),
            state: RecState::Recording,
            last_disk_check: Instant::now(),
        });
        let r = RootScreen::new();
        let mut grid = new_grid();
        r.render_chrome(&s, &mut grid);
        let bar = grid.row_text(layout::ROW_HOTKEYS);
        assert!(bar.contains("<REC>"), "hotkey bar: {bar:?}");
    }

    #[test]
    fn hotkeys_show_sav_when_finalizing() {
        use crate::capture::recording::{ActiveRecording, RecState};
        use std::time::Instant;
        let mut s = SharedState::new();
        s.active_recording = Some(ActiveRecording {
            device_path: "/dev/video0".into(),
            file_path: "/tmp/rec.mp4".into(),
            started_at: Instant::now(),
            state: RecState::Finalizing,
            last_disk_check: Instant::now(),
        });
        let r = RootScreen::new();
        let mut grid = new_grid();
        r.render_chrome(&s, &mut grid);
        let bar = grid.row_text(layout::ROW_HOTKEYS);
        assert!(bar.contains("<SAV>"), "hotkey bar: {bar:?}");
    }

    #[test]
    fn frame_has_corners_and_borders() {
        let st = SharedState::new();
        let root = RootScreen::new();
        let mut grid = new_grid();
        root.render(&st, &mut grid);
        assert_eq!(grid.at(0, 0).ch, '┌');
        assert_eq!(grid.at(0, layout::COLS - 1).ch, '┐');
        assert_eq!(grid.at(layout::ROW_BOTTOM, 0).ch, '└');
        assert_eq!(grid.at(layout::ROW_BOTTOM, layout::COLS - 1).ch, '┘');
        // Side borders present on a mid row.
        assert_eq!(grid.at(5, 0).ch, '│');
        assert_eq!(grid.at(5, layout::COLS - 1).ch, '│');
    }

    #[test]
    fn nav_routes_to_active_body() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("a.mp4"), b"").unwrap();
        std::fs::write(tmp.path().join("b.mp4"), b"").unwrap();
        let mut st = SharedState::new();
        st.display_mode = DisplayMode::Browser;
        st.paths_to_browser = vec![tmp.path().to_path_buf()];
        let mut root = RootScreen::new();

        // Selection highlight lands on interior cols (PANE_L0), not the border.
        let col = layout::PANE_L0;

        // Before NavDown: selected 0 → list row 0 inverted.
        let mut grid = new_grid();
        root.render(&st, &mut grid);
        assert!(
            grid.at(list_row(0), col).attr & ATTR_INVERSE != 0,
            "list row 0 should be inverted before nav"
        );
        assert!(
            grid.at(list_row(1), col).attr & ATTR_INVERSE == 0,
            "list row 1 should not be inverted before nav"
        );

        // NavDown → selected 1 → list row 1 inverted.
        root.handle(Action::NavDown, &mut st);
        let mut grid2 = new_grid();
        root.render(&st, &mut grid2);
        assert!(
            grid2.at(list_row(0), col).attr & ATTR_INVERSE == 0,
            "list row 0 should not be inverted after nav"
        );
        assert!(
            grid2.at(list_row(1), col).attr & ATTR_INVERSE != 0,
            "list row 1 should be inverted after nav"
        );
    }

    /// Visual check: prints the sampler screen so the layout can be eyeballed
    /// with `cargo test dump_sampler_layout -- --nocapture`.
    #[test]
    fn dump_sampler_layout() {
        use crate::state::{Slot, SourceKind};
        let mut st = SharedState::new();
        st.display_mode = DisplayMode::Sampler;
        st.banks[0].slots[0] = Some(Slot {
            source: SourceKind::File("/clips/cityscape.mp4".into()),
            name: "cityscape.mp4".into(),
            start: 0.0,
            end: 4.2,
            length: 4.2,
            rate: 1.0,
        });
        let root = RootScreen::new();
        let mut grid = new_grid();
        root.render(&st, &mut grid);
        println!("\n+{}+", "-".repeat(layout::COLS));
        for r in 0..layout::ROWS {
            println!("|{}|", grid.row_text(r));
        }
        println!("+{}+", "-".repeat(layout::COLS));
    }
}
