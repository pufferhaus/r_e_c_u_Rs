//! Always-on chrome screen. Renders the title, transport banner, status row,
//! mode tabs, column headers, and footer. Delegates the body region to one
//! of BrowserBody / SamplerBody / SettingsBody.

use crate::action::Action;
use crate::state::{DisplayMode, SharedState};
use crate::status::grid::TextGrid;
use crate::ui::{Screen, ScreenResult};

use super::{browser::BrowserBody, sampler::SamplerBody, settings::SettingsBody};

pub struct RootScreen;

impl RootScreen {
    pub fn new() -> Self {
        Self
    }

    fn render_chrome(&self, state: &SharedState, grid: &mut TextGrid) {
        // Row 1 — title
        let title = match state.display_mode {
            DisplayMode::Shaders | DisplayMode::ShdrBnk => "============== c_o_n_j_u_r =============",
            DisplayMode::Frames => "============== d_e_t_o_u_r =============",
            _ => "============== r_e_c_u_r ===============",
        };
        grid.write_row(0, title);

        // Row 2 — transport banner (stub when no player loaded)
        grid.write_row(1, " 00:00 [-----------------------] 00:00");

        // Row 3 — status: NOW [b-s] STATUS    NEXT [b-s] STATUS
        grid.write_row(2, "NOW [0-0] -                NEXT [0-0] -");

        // Row 4 — mode tabs
        grid.write_row(3, &body_title(state));

        // Row 16 — footer (status / message)
        let footer = if state.function_on {
            "               < FUNCTION KEY ON >".to_string()
        } else {
            format!("CONTROL: {:?}", state.control_mode)
        };
        grid.write_row(15, &footer);
    }
}

impl Screen for RootScreen {
    fn render(&self, state: &SharedState, grid: &mut TextGrid) {
        self.render_chrome(state, grid);
        let body: Box<dyn Screen> = match state.display_mode {
            DisplayMode::Browser => Box::new(BrowserBody::new()),
            DisplayMode::Sampler => Box::new(SamplerBody::new()),
            DisplayMode::Settings => Box::new(SettingsBody::new()),
            _ => Box::new(StubBody),
        };
        body.render(state, grid);
    }

    fn handle(&mut self, _action: Action, _state: &mut SharedState) -> ScreenResult {
        // RootScreen never consumes — body screens do.
        ScreenResult::Continue
    }
}

struct StubBody;
impl Screen for StubBody {
    fn render(&self, _state: &SharedState, grid: &mut TextGrid) {
        grid.write_row(10, "      (not yet implemented in Phase 1)");
    }
    fn handle(&mut self, _: Action, _: &mut SharedState) -> ScreenResult {
        ScreenResult::Continue
    }
}

fn body_title(state: &SharedState) -> String {
    let abbrev = |m: DisplayMode| match m {
        DisplayMode::Browser => "br",
        DisplayMode::Sampler => "sa",
        DisplayMode::Settings => "se",
        DisplayMode::Shaders => "sh",
        DisplayMode::ShdrBnk => "sb",
        DisplayMode::Frames => "fr",
    };
    let all = [
        DisplayMode::Browser,
        DisplayMode::Settings,
        DisplayMode::Sampler,
        DisplayMode::Shaders,
        DisplayMode::ShdrBnk,
        DisplayMode::Frames,
    ];
    let mut parts = Vec::new();
    for m in all {
        if m == state.display_mode {
            parts.push(format!("[{:_<8}]", format!("{:?}", m).to_lowercase()));
        } else {
            parts.push(format!("<{}>", abbrev(m)));
        }
    }
    let s = parts.join("");
    format!("---{}---", &s[..s.len().min(42)])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_title_brackets_current_mode() {
        let mut st = SharedState::new();
        st.display_mode = DisplayMode::Sampler;
        let t = body_title(&st);
        assert!(t.contains("[sampler"));
    }
}
