//! End-to-end UI simulation: replays scripted key presses through the exact
//! dispatch path main.rs uses (keymap → ScreenStack::dispatch → apply), with a
//! recording rack, and dumps the rendered LCD grid after key steps.
//!
//! Run with:  cargo test --test sim -- --nocapture

use std::path::{Path, PathBuf};

use recur::action::Action;
use recur::apply::{apply, RackHandle};
use recur::input::keymap::Keymap;
use recur::menu::layout;
use recur::menu::root::RootScreen;
use recur::state::{Bank, DisplayMode, SharedState, Slot};
use recur::status::grid::{TextGrid, ATTR_INVERSE};
use recur::ui::ScreenStack;

/// Records what the rack was asked to do so the test can assert on it.
#[derive(Default)]
struct SimRack {
    triggers: Vec<(u8, u8, String)>,
    toggles: u32,
    seeks: Vec<f64>,
    reloads: u32,
    binding: Option<(u8, u8)>,
    position: Option<f64>,
}

impl RackHandle for SimRack {
    fn reload_all(&mut self) {
        self.reloads += 1;
    }
    fn trigger_slot_with(&mut self, bank: u8, slot_idx: u8, slot: Slot, _b: Bank) {
        self.triggers.push((bank, slot_idx, slot.name));
        self.binding = Some((bank, slot_idx));
    }
    fn current_position(&self) -> Option<f64> {
        self.position
    }
    fn current_binding(&self) -> Option<(u8, u8)> {
        self.binding
    }
    fn toggle_play_pause_now(&mut self) {
        self.toggles += 1;
    }
    fn seek_relative_now(&mut self, s: f64) {
        self.seeks.push(s);
    }
    fn set_rate_now(&mut self, _: f32) {}
    fn trigger_shader(&mut self, _: &str, _: [f32; 8]) {}
    fn clear_shader(&mut self) {}
    fn set_shader_params(&mut self, _: [f32; 8]) {}
    fn detour_scrub_by(&mut self, _: i32) {}
    fn start_recording(
        &mut self,
        _: &str,
        _: &Path,
        _: recur::capture::recording::Target,
    ) -> recur::error::Result<()> {
        Ok(())
    }
    fn stop_recording(&mut self) {}
    fn drain_finalized(&mut self) -> Vec<PathBuf> {
        Vec::new()
    }
}

struct Sim {
    keymap: Keymap,
    state: SharedState,
    stack: ScreenStack,
    rack: SimRack,
    passes: u32,
    fails: u32,
}

impl Sim {
    fn new() -> Self {
        let keymap = Keymap::parse(include_str!("../keymap.toml")).expect("keymap parses");
        let mut stack = ScreenStack::new();
        stack.push(Box::new(RootScreen::new()));
        Sim {
            keymap,
            state: SharedState::new(),
            stack,
            rack: SimRack::default(),
            passes: 0,
            fails: 0,
        }
    }

    /// Press a physical key (base layer) — mirrors the input sources, which now
    /// resolve through the context-aware `resolve()` (detour / shader-bank).
    fn press(&mut self, key: &str) -> Action {
        let action = self
            .keymap
            .resolve(key, self.state.display_mode, self.state.control_mode)
            .unwrap_or_else(|| panic!("key {key:?} not bound"));
        self.dispatch(action.clone());
        action
    }

    /// Press a numpad key on the FN layer (000-held) — mirrors evdev_src.
    fn press_fn(&mut self, key: &str) -> Action {
        let action = self
            .keymap
            .lookup_fn(key)
            .or_else(|| {
                self.keymap
                    .resolve(key, self.state.display_mode, self.state.control_mode)
            })
            .unwrap_or_else(|| panic!("fn key {key:?} not bound"));
        self.dispatch(action.clone());
        action
    }

    fn dispatch(&mut self, action: Action) {
        let synth = self.stack.dispatch(action.clone(), &mut self.state);
        apply(action, &mut self.state, &mut self.rack);
        if let Some(a) = synth {
            apply(a, &mut self.state, &mut self.rack);
        }
    }

    fn render(&self) -> TextGrid {
        let mut g = TextGrid::new(layout::COLS, layout::ROWS);
        if let Some(top) = self.stack.top() {
            top.render(&self.state, &mut g);
        }
        g
    }

    fn dump(&self, label: &str) {
        let g = self.render();
        println!("\n── {label} ──");
        for r in 0..layout::ROWS {
            println!("  {}", g.row_text(r));
        }
    }

    fn check(&mut self, what: &str, ok: bool) {
        if ok {
            self.passes += 1;
            println!("  ✓ {what}");
        } else {
            self.fails += 1;
            println!("  ✗ {what}");
        }
    }

    /// Grid row of left-pane list row `i`.
    fn list_row(i: usize) -> usize {
        layout::ROW_BODY0 + 1 + i
    }

    /// Is left-pane list row `i` the selected (inverted) one?
    fn row_selected(&self, i: usize) -> bool {
        let g = self.render();
        g.at(Self::list_row(i), layout::PANE_L0).attr & ATTR_INVERSE != 0
    }
}

#[test]
fn simulate_full_ui_walkthrough() {
    let mut sim = Sim::new();

    println!("\n========== r_e_c_u_r UI SIMULATION ==========");
    println!("Driving the real keymap → dispatch → apply path.\n");

    // ---- 1. Sampler navigation ----
    println!("[1] SAMPLER navigation (ArrowDown ×3)");
    sim.state.display_mode = DisplayMode::Sampler;
    sim.check("starts on slot 0", sim.row_selected(0));
    sim.press("ArrowDown");
    sim.press("ArrowDown");
    sim.press("ArrowDown");
    sim.check("selection moved to slot 3", sim.row_selected(3));
    sim.check("slot 0 no longer selected", !sim.row_selected(0));
    sim.press("ArrowUp");
    sim.check("ArrowUp moved back to slot 2", sim.row_selected(2));
    sim.dump("after nav: sampler, slot 2 selected");

    // ---- 2. Mode switching ----
    println!("\n[2] MODE switching");
    sim.press("KeyG");
    sim.check("KeyG → Settings mode", sim.state.display_mode == DisplayMode::Settings);
    sim.press("KeyH");
    sim.check("KeyH → Shaders mode", sim.state.display_mode == DisplayMode::Shaders);
    sim.press("KeyB");
    sim.check("KeyB → Browser mode", sim.state.display_mode == DisplayMode::Browser);
    sim.press("KeyS");
    sim.check("KeyS → Sampler mode", sim.state.display_mode == DisplayMode::Sampler);

    // ---- 3. Bank switching ----
    println!("\n[3] BANK switching");
    sim.check("starts on bank 0", sim.state.bank_number == 0);
    sim.press("Period");
    sim.check("Period (.) → bank 1", sim.state.bank_number == 1);
    let title = sim.render().row_text(layout::ROW_TOP);
    sim.check("title shows BANK B", title.contains("BANK B"));
    sim.press("Comma");
    sim.check("Comma (,) → back to bank 0", sim.state.bank_number == 0);

    // ---- 4. Slot triggering ----
    println!("\n[4] SLOT triggering");
    // Map a clip into slot 4 so a trigger has something to play.
    sim.state.banks[0].slots[4] = Some(Slot {
        source: recur::state::SourceKind::File("/clips/demo.mp4".into()),
        name: "demo.mp4".into(),
        start: -1.0,
        end: -1.0,
        length: 0.0,
        rate: 1.0,
    });
    sim.press("Digit4");
    sim.check(
        "Digit4 triggered slot 4 on the rack",
        sim.rack.triggers.iter().any(|(_, s, n)| *s == 4 && n == "demo.mp4"),
    );
    sim.check("now_playing set to (bank 0, slot 4)", sim.state.now_playing == Some((0, 4)));
    sim.check("trigger flash armed", sim.state.trigger_flash > 0);
    let marker_cell = sim.render().at(Sim::list_row(4), layout::PANE_L0).ch;
    sim.check("playing row shows '>' marker", marker_cell == '>');
    sim.check(
        "loops by default (on_finish=Repeat)",
        matches!(sim.state.sampler.on_finish, recur::state::OnFinish::Repeat),
    );

    // ---- 5. Function (map) toggle ----
    println!("\n[5] FUNCTION key toggle");
    sim.check("function off initially", !sim.state.function_on);
    sim.press("ShiftLeft");
    sim.check("ShiftLeft → function ON", sim.state.function_on);
    let hot = sim.render().row_text(layout::ROW_HOTKEYS);
    sim.check("hotkey bar shows FUNCTION ON", hot.contains("FUNCTION ON"));
    sim.press("ShiftLeft");
    sim.check("ShiftLeft again → function OFF", !sim.state.function_on);

    // ---- 6. Transport: play/pause + seek ----
    println!("\n[6] TRANSPORT play/pause + seek (numpad FN layer)");
    let before = sim.rack.toggles;
    sim.press("Space");
    sim.check("Space toggled play/pause on rack", sim.rack.toggles == before + 1);
    // Seek is on the FN layer of the operator keys: FN+× fwd, FN+/ back.
    sim.press_fn("NumpadMultiply");
    sim.check(
        "FN+× seeks forward by seek_time",
        sim.rack.seeks.last() == Some(&sim.state.sampler.seek_time),
    );
    sim.press_fn("NumpadDivide");
    sim.check(
        "FN+/ seeks backward",
        sim.rack.seeks.last() == Some(&(-sim.state.sampler.seek_time)),
    );

    // ---- 7. Feedback toggle (numpad FN layer) ----
    println!("\n[7] FEEDBACK effect toggle (FN+−)");
    sim.check("feedback off initially", !sim.state.feedback_active);
    sim.press_fn("NumpadSubtract");
    sim.check("FN+− → feedback ON", sim.state.feedback_active);
    sim.press_fn("NumpadSubtract");
    sim.check("FN+− again → feedback OFF", !sim.state.feedback_active);

    // ---- 8. Detour enter / exit ----
    println!("\n[8] DETOUR enter / exit");
    sim.press("KeyD");
    sim.check("KeyD → Frames (detour) display", sim.state.display_mode == DisplayMode::Frames);
    sim.dump("detour screen");
    sim.press("KeyE");
    sim.check(
        "KeyE → exits detour (display restored away from Frames)",
        sim.state.display_mode != DisplayMode::Frames,
    );

    // ---- 9. Settings cycle ----
    println!("\n[9] SETTINGS cycle (Enter on focused setting)");
    sim.press("KeyG");
    let before_loop = format!("{:?}", sim.state.sampler.loop_type);
    sim.press("Enter"); // Enter on first setting (loop_type)
    let after_loop = format!("{:?}", sim.state.sampler.loop_type);
    sim.check(
        &format!("Enter cycles loop_type ({before_loop} → {after_loop})"),
        before_loop != after_loop,
    );
    sim.dump("settings screen after cycle");

    // ---- 10. Browser map a file ----
    println!("\n[10] BROWSER map a file into a slot (Enter)");
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("clip_a.mp4"), b"").unwrap();
    sim.state.paths_to_browser = vec![tmp.path().to_path_buf()];
    sim.state.banks[0].slots[0] = None; // ensure an empty target
    sim.press("KeyB");
    sim.dump("browser listing");
    sim.press("Enter"); // map highlighted file into first empty slot
    sim.check(
        "Enter mapped clip_a.mp4 into a bank slot",
        sim.state.banks[0].slots.iter().flatten().any(|s| s.name == "clip_a.mp4"),
    );

    // ---- 11. Panic recovery ----
    println!("\n[11] PANIC resets to sampler / default");
    sim.state.display_mode = DisplayMode::Shaders;
    sim.state.function_on = true;
    apply(Action::Panic, &mut sim.state, &mut sim.rack);
    sim.check("Panic → display back to Sampler", sim.state.display_mode == DisplayMode::Sampler);
    sim.check("Panic → function cleared", !sim.state.function_on);

    println!("\n========== RESULT: {} passed, {} failed ==========\n", sim.passes, sim.fails);
    assert_eq!(sim.fails, 0, "{} simulated checks failed", sim.fails);
}

#[test]
fn simulate_numpad_mapping() {
    let mut sim = Sim::new();
    println!("\n========== USB NUMPAD MAPPING ==========\n");

    // ---- Reading-order slots: physical key 9 = slot 1, 0 = slot 0 ----
    println!("[A] Reading-order slot triggers (normal layer)");
    sim.state.display_mode = DisplayMode::Sampler;
    for s in 0..10 {
        sim.state.banks[0].slots[s] = Some(Slot {
            source: recur::state::SourceKind::File(format!("/clips/s{s}.mp4").into()),
            name: format!("s{s}.mp4"),
            start: -1.0,
            end: -1.0,
            length: 0.0,
            rate: 1.0,
        });
    }
    // top-left physical key (Numpad9) → slot 1; lone key (Numpad0) → slot 0.
    sim.press("Numpad9");
    sim.check("physical 9 → slot 1", sim.rack.triggers.last().map(|t| t.1) == Some(1));
    sim.press("Numpad1");
    sim.check("physical 1 → slot 9", sim.rack.triggers.last().map(|t| t.1) == Some(9));
    sim.press("Numpad0");
    sim.check("physical 0 → slot 0", sim.rack.triggers.last().map(|t| t.1) == Some(0));
    sim.press("Numpad5");
    sim.check("physical 5 → slot 5 (centre)", sim.rack.triggers.last().map(|t| t.1) == Some(5));

    // ---- Operator keys ----
    println!("\n[B] Operator keys (normal layer)");
    let a = sim.press("NumpadSubtract");
    sim.check("− prev bank", matches!(a, Action::PrevBank));
    let a = sim.press("NumpadAdd");
    sim.check("+ next bank", matches!(a, Action::NextBank));
    let a = sim.press("NumpadMultiply");
    sim.check("× nav up", matches!(a, Action::NavUp));
    let a = sim.press("NumpadDivide");
    sim.check("/ nav down", matches!(a, Action::NavDown));
    let a = sim.press("NumpadDecimal");
    sim.check(". play/pause", matches!(a, Action::TogglePlayPause));

    // ---- FN layer (000 then key) ----
    println!("\n[C] FN layer (hold 000)");
    let a = sim.press_fn("Numpad9");
    sim.check("FN+9 → Sampler", matches!(a, Action::EnterMode(DisplayMode::Sampler)));
    let a = sim.press_fn("Numpad6");
    sim.check("FN+6 → Browser", matches!(a, Action::EnterMode(DisplayMode::Browser)));
    let a = sim.press_fn("Numpad5");
    sim.check("FN+5 → ShdrBnk", matches!(a, Action::EnterMode(DisplayMode::ShdrBnk)));
    let a = sim.press_fn("Numpad7");
    sim.check("FN+7 → loop in", matches!(a, Action::SetLoopIn));
    let a = sim.press_fn("NumpadSubtract");
    sim.check("FN+− → feedback", matches!(a, Action::ToggleFeedback));
    let a = sim.press_fn("Backspace");
    sim.check("FN+Bksp → panic", matches!(a, Action::Panic));

    // ---- Auto-context: DETOUR ----
    println!("\n[D] Auto-context — Detour scrub (digits become scrub controls)");
    sim.state.display_mode = DisplayMode::Frames;
    sim.state.control_mode = recur::state::ControlMode::DetourScrub;
    let a = sim.press("Numpad8");
    sim.check("in detour, physical 8 → scrub -1", matches!(a, Action::DetourScrubBy(-1)));
    let a = sim.press("Numpad2");
    sim.check("in detour, physical 2 → scrub +1", matches!(a, Action::DetourScrubBy(1)));
    let a = sim.press("Numpad5");
    sim.check("in detour, physical 5 → clear markers", matches!(a, Action::DetourClearMarkers));
    let a = sim.press("Numpad9");
    sim.check("in detour, physical 9 → cycle mix", matches!(a, Action::DetourCycleMix));
    sim.state.control_mode = recur::state::ControlMode::Default;

    // ---- Auto-context: SHADER BANK ----
    println!("\n[E] Auto-context — Shader bank (digits trigger shader slots)");
    sim.state.display_mode = DisplayMode::ShdrBnk;
    let a = sim.press("Numpad9");
    sim.check("in shdrbnk, physical 9 → shader slot 1", matches!(a, Action::TriggerShaderSlot(1)));
    let a = sim.press("Numpad0");
    sim.check("in shdrbnk, physical 0 → shader slot 0", matches!(a, Action::TriggerShaderSlot(0)));
    let a = sim.press("NumpadMultiply");
    sim.check("in shdrbnk, × still nav up", matches!(a, Action::NavUp));

    println!("\n========== RESULT: {} passed, {} failed ==========\n", sim.passes, sim.fails);
    assert_eq!(sim.fails, 0, "{} numpad checks failed", sim.fails);
}
