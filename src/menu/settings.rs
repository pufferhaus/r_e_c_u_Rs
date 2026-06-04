//! SettingsBody — flat list of settings, Enter cycles options.

use crate::action::{Action, SettingId};
use crate::state::SharedState;
use crate::status::grid::TextGrid;
use crate::ui::{Screen, ScreenResult};

const ITEMS: &[(SettingId, &str)] = &[
    (SettingId::LoopType, "loop_type"),
    (SettingId::OnFinish, "on_finish"),
    (SettingId::OnStart, "on_start"),
    (SettingId::OnLoad, "on_load"),
    (SettingId::LoadNext, "load_next"),
    (SettingId::RandStartMode, "rand_start_mode"),
    (SettingId::FixedLengthMode, "fixed_length_mode"),
    (SettingId::FixedLengthMultiply, "fixed_length_multiply"),
    (SettingId::ResetPlayers, "reset_players"),
    (SettingId::SeekTime, "seek_time"),
    (SettingId::ActionGated, "action_gated"),
    (SettingId::FuncGated, "func_gated"),
    (SettingId::StrobeAmount, "strobe_amount"),
];

pub struct SettingsBody {
    selected: usize,
}

impl SettingsBody {
    pub fn new() -> Self {
        Self { selected: 0 }
    }
}

impl Default for SettingsBody {
    fn default() -> Self {
        Self::new()
    }
}

impl Screen for SettingsBody {
    fn render(&self, state: &SharedState, grid: &mut TextGrid) {
        use crate::menu::layout;
        layout::left_header(grid, &format!("{:<23} {:<22}", "SETTING", "VALUE"));
        for (i, (id, name)) in ITEMS.iter().enumerate() {
            let value = value_for(state, *id);
            layout::left_row(
                grid,
                i,
                &format!("{:<23} {:<22}", name, value),
                crate::status::grid::ATTR_NORMAL,
            );
            if i == self.selected {
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
                self.selected = (self.selected + 1).min(ITEMS.len() - 1);
                ScreenResult::Continue
            }
            Action::Enter => {
                // Synthesise a CycleSetting action so the central apply.rs path
                // performs the cycle (and enforces invariants). The dispatch
                // loop forwards `ScreenResult::Action` to apply().
                let id = ITEMS[self.selected].0;
                ScreenResult::Action(Action::CycleSetting(id))
            }
            _ => ScreenResult::Continue,
        }
    }
}

fn value_for(state: &SharedState, id: SettingId) -> String {
    let s = &state.sampler;
    match id {
        SettingId::LoopType => format!("{:?}", s.loop_type).to_lowercase(),
        SettingId::OnFinish => format!("{:?}", s.on_finish).to_lowercase(),
        SettingId::OnStart => format!("{:?}", s.on_start).to_lowercase(),
        SettingId::OnLoad => format!("{:?}", s.on_load).to_lowercase(),
        SettingId::LoadNext => format!("{:?}", s.load_next).to_lowercase(),
        SettingId::RandStartMode => s.rand_start_mode.to_string(),
        SettingId::FixedLengthMode => s.fixed_length_mode.to_string(),
        SettingId::FixedLengthMultiply => format!("{:.2}x", s.fixed_length_multiply),
        SettingId::ResetPlayers => s.reset_players.to_string(),
        SettingId::SeekTime => format!("{}s", s.seek_time),
        SettingId::ActionGated => s.action_gated.to_string(),
        SettingId::FuncGated => s.func_gated.to_string(),
        SettingId::StrobeAmount => s.strobe_amount.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nav_clamps_at_bounds() {
        let mut s = SettingsBody::new();
        let mut st = SharedState::new();
        for _ in 0..50 {
            s.handle(Action::NavDown, &mut st);
        }
        assert_eq!(s.selected, ITEMS.len() - 1);
        for _ in 0..50 {
            s.handle(Action::NavUp, &mut st);
        }
        assert_eq!(s.selected, 0);
    }
}
