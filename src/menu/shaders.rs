//! ShadersBody — browse paired `.glsl + .toml` pairs from the shader dir.
//! Enter on a row stashes the shader name into `state.shader_pending_select`;
//! the user then presses Function + SelectShaderSlot(n) to map it.

use crate::action::Action;
use crate::state::SharedState;
use crate::status::grid::TextGrid;
use crate::ui::{Screen, ScreenResult};

pub struct ShadersBody {
    pub names: Vec<String>,
    pub filtered: usize,
    selected: usize,
}

impl ShadersBody {
    pub fn new(names: Vec<String>, filtered: usize) -> Self {
        Self {
            names,
            filtered,
            selected: 0,
        }
    }
}

impl Screen for ShadersBody {
    fn render(&self, _state: &SharedState, grid: &mut TextGrid) {
        use crate::menu::layout;
        layout::left_header(grid, "shader");
        // Reserve the last list row for the count footer.
        let list_rows = layout::BODY_LIST_ROWS - 1;
        for view_i in 0..list_rows {
            match self.names.get(view_i) {
                None => {
                    layout::left_row(grid, view_i, "", crate::status::grid::ATTR_NORMAL);
                }
                Some(name) => {
                    let truncated: String = name.chars().take(layout::PANE_LW).collect();
                    layout::left_row(grid, view_i, &truncated, crate::status::grid::ATTR_NORMAL);
                    if view_i == self.selected {
                        layout::invert_left_row(grid, view_i);
                    }
                }
            }
        }
        let footer = if self.filtered > 0 {
            format!("{} shown, {} hidden (pi5-only)", self.names.len(), self.filtered)
        } else {
            format!("{} shaders", self.names.len())
        };
        layout::left_row(grid, list_rows, &footer, crate::status::grid::ATTR_DIM);
    }

    fn handle(&mut self, action: Action, state: &mut SharedState) -> ScreenResult {
        if self.names.is_empty() {
            return ScreenResult::Continue;
        }
        match action {
            Action::NavUp => self.selected = self.selected.saturating_sub(1),
            Action::NavDown => self.selected = (self.selected + 1).min(self.names.len() - 1),
            Action::Enter => {
                state.shader_pending_select = Some(self.names[self.selected].clone());
            }
            _ => {}
        }
        ScreenResult::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::Action;
    use crate::state::SharedState;

    #[test]
    fn enter_stashes_selected_name_into_state() {
        let names = vec!["color_shift".to_string(), "pixelate".to_string()];
        let mut body = ShadersBody::new(names, 0);
        let mut s = SharedState::new();
        body.handle(Action::NavDown, &mut s);
        body.handle(Action::Enter, &mut s);
        assert_eq!(s.shader_pending_select.as_deref(), Some("pixelate"));
    }

    #[test]
    fn footer_shows_filtered_count_when_nonzero() {
        use crate::menu::layout;
        let body = ShadersBody::new(vec!["a".into()], 3);
        let mut grid = crate::status::grid::TextGrid::new(layout::COLS, layout::ROWS);
        let s = SharedState::new();
        body.render(&s, &mut grid);
        // Footer occupies the last list row.
        let row = grid.row_text(layout::ROW_BODY0 + 1 + (layout::BODY_LIST_ROWS - 1));
        assert!(row.contains("hidden"), "got: {row}");
    }
}
