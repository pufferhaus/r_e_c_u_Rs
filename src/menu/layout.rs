//! Shared 80×26 bordered layout chrome (mandleROT-style).
//!
//! ```text
//! ┌─ r_e_c_u_r ──────────────────────────── BANK A ─┐  row 0
//! │ 00:00 [=========------------------] 04:20       │  row 1  transport
//! ├─ NOW [0-0] - ───────────┬─ MODE ───────────────┤  row 2  now / menu hdr
//! │ <body header>           │  S sampler            │  row 3
//! │ <body rows ...>         │  B browser            │  rows 4..22
//! │                         │  ...                  │
//! ├─────────────────────────┴───────────────────────┤  row 23 divider
//! │ <hotkey hints>                                   │  row 24 hotkeys
//! └──────────────────────────────────────────────────┘  row 25 bottom
//! ```
//!
//! The left pane (cols `PANE_L0..PANE_L0+PANE_LW`) holds the active mode body.
//! The right pane is a static mode-menu legend highlighting the active mode.

use crate::render::shader_assembly::GlesProfile;
use crate::state::{DisplayMode, SharedState};
use crate::status::grid::{Cell, TextGrid, ATTR_BRIGHT, ATTR_DIM, ATTR_NORMAL};

pub const COLS: usize = 80;
pub const ROWS: usize = 26;

// Vertical structure.
pub const ROW_TOP: usize = 0;
pub const ROW_TRANSPORT: usize = 1;
pub const ROW_DIVIDER_TOP: usize = 2;
pub const ROW_BODY0: usize = 3; // first body row (header)
pub const ROW_BODY_END: usize = 22; // last usable body row (inclusive)
pub const ROW_DIVIDER_BOT: usize = 23;
pub const ROW_HOTKEYS: usize = 24;
pub const ROW_BOTTOM: usize = 25;

// Horizontal structure.
pub const PANE_DIV: usize = 50; // vertical divider column
pub const PANE_L0: usize = 2; // left-pane content start col
pub const PANE_LW: usize = 47; // left-pane content width (cols 2..49)
pub const PANE_R0: usize = 52; // right-pane content start col
pub const PANE_RW: usize = 26; // right-pane content width (cols 52..78)

/// Number of body rows available to a left-pane list (header excluded).
pub const BODY_LIST_ROWS: usize = ROW_BODY_END - (ROW_BODY0 + 1) + 1; // 19

/// Draw the full outer frame, title/bank top bar, transport, divider rows,
/// the right-pane mode menu, and the hotkey bar. The caller then fills the
/// left pane (and any per-mode right-pane overrides) and the transport/now
/// content via the helpers below.
pub fn draw_chrome(state: &SharedState, grid: &mut TextGrid) {
    draw_frame(grid);
    draw_top_bar(state, grid);
    draw_transport(state, grid);
    draw_divider_top(state, grid);
    draw_mode_menu(state, grid);
    draw_divider_bot(grid);
    draw_hotkeys(state, grid);
}

fn dim(ch: char) -> Cell {
    Cell::new(ch, ATTR_DIM)
}

/// Outer rectangle + the vertical pane divider.
fn draw_frame(grid: &mut TextGrid) {
    // Top and bottom borders.
    grid.fill(ROW_TOP, 0, COLS, '─', ATTR_DIM);
    grid.fill(ROW_BOTTOM, 0, COLS, '─', ATTR_DIM);
    grid.set(ROW_TOP, 0, dim('┌'));
    grid.set(ROW_TOP, COLS - 1, dim('┐'));
    grid.set(ROW_BOTTOM, 0, dim('└'));
    grid.set(ROW_BOTTOM, COLS - 1, dim('┘'));
    // Side borders on every interior row.
    for r in 1..ROW_BOTTOM {
        grid.set(r, 0, dim('│'));
        grid.set(r, COLS - 1, dim('│'));
    }
    // Vertical pane divider between the two body dividers.
    for r in (ROW_DIVIDER_TOP + 1)..ROW_DIVIDER_BOT {
        grid.set(r, PANE_DIV, dim('│'));
    }
}

/// Row 0: `┌─ <title> ──…── BANK X ─┐` (title left, bank right).
fn draw_top_bar(state: &SharedState, grid: &mut TextGrid) {
    let title = match state.display_mode {
        DisplayMode::Shaders | DisplayMode::ShdrBnk => "c_o_n_j_u_r",
        DisplayMode::Frames => "d_e_t_o_u_r",
        _ => "r_e_c_u_r",
    };
    grid.write(ROW_TOP, 2, ATTR_BRIGHT, &format!(" {title} "));
    let bank = format!(" BANK {} ", bank_letter(state.bank_number));
    let bcol = COLS - 2 - bank.chars().count();
    grid.write(ROW_TOP, bcol, ATTR_BRIGHT, &bank);
}

/// Row 1: transport `MM:SS [====----] MM:SS` spanning the full interior.
/// Position/length aren't threaded through SharedState yet, so this renders an
/// empty bar (parity with the previous transport stub).
fn draw_transport(_state: &SharedState, grid: &mut TextGrid) {
    let (pos, len) = (0.0f64, 0.0f64);
    let bar_w = (COLS - 2) - (PANE_L0 + 6 + 1 + 1 + 6); // room for both time stamps
    let filled = if len > 0.0 {
        ((pos / len) * bar_w as f64).round().clamp(0.0, bar_w as f64) as usize
    } else {
        0
    };
    let bar: String = (0..bar_w)
        .map(|i| if i < filled { '=' } else { '-' })
        .collect();
    let line = format!("{} [{}] {}", fmt_clock(pos), bar, fmt_clock(len));
    grid.write(ROW_TRANSPORT, PANE_L0, ATTR_NORMAL, &line);
}

/// Row 2: `├─ NOW [b-s] status ──┬─ MODE ──┤` — left = now player, right header.
fn draw_divider_top(state: &SharedState, grid: &mut TextGrid) {
    grid.fill(ROW_DIVIDER_TOP, 1, COLS - 2, '─', ATTR_DIM);
    grid.set(ROW_DIVIDER_TOP, 0, dim('├'));
    grid.set(ROW_DIVIDER_TOP, COLS - 1, dim('┤'));
    grid.set(ROW_DIVIDER_TOP, PANE_DIV, dim('┬'));
    let now = format!(" NOW {} ", now_status(state));
    grid.write(ROW_DIVIDER_TOP, PANE_L0, ATTR_BRIGHT, &now);
    grid.write(ROW_DIVIDER_TOP, PANE_R0, ATTR_BRIGHT, " MODE ");
}

/// Right pane: vertical mode-menu legend, active mode inverted.
fn draw_mode_menu(state: &SharedState, grid: &mut TextGrid) {
    const ITEMS: &[(DisplayMode, &str, &str)] = &[
        (DisplayMode::Sampler, "S", "sampler"),
        (DisplayMode::Browser, "B", "browser"),
        (DisplayMode::Settings, "G", "settings"),
        (DisplayMode::Shaders, "H", "shaders"),
        (DisplayMode::ShdrBnk, "K", "shdrbank"),
        (DisplayMode::Frames, "D", "detour"),
    ];
    for (i, (mode, key, name)) in ITEMS.iter().enumerate() {
        let row = ROW_BODY0 + 1 + i;
        grid.write(row, PANE_R0, ATTR_NORMAL, &format!(" {key} {name}"));
        if *mode == state.display_mode {
            grid.invert_span(row, PANE_DIV + 1, COLS - 1);
        }
    }
    // Capture hint below the mode list.
    grid.write(
        ROW_BODY0 + 1 + ITEMS.len() + 1,
        PANE_R0,
        ATTR_DIM,
        " C add capture",
    );
}

/// Row 23: `├──…──┴──…──┤` closing the pane divider.
fn draw_divider_bot(grid: &mut TextGrid) {
    grid.fill(ROW_DIVIDER_BOT, 1, COLS - 2, '─', ATTR_DIM);
    grid.set(ROW_DIVIDER_BOT, 0, dim('├'));
    grid.set(ROW_DIVIDER_BOT, COLS - 1, dim('┤'));
    grid.set(ROW_DIVIDER_BOT, PANE_DIV, dim('┴'));
}

/// Row 24: context hotkey hints + recording / function indicators.
fn draw_hotkeys(state: &SharedState, grid: &mut TextGrid) {
    let mut hints = if state.function_on {
        "  < FUNCTION ON >  [0-9] map slot   Shift: gate".to_string()
    } else {
        "[0-9] play  Shift map  SPACE play  ,/. bank  \\ loop".to_string()
    };

    if let Some(rec) = state.active_recording.as_ref() {
        use crate::capture::recording::RecState;
        let suffix = match rec.state {
            RecState::Recording => {
                let secs = rec.started_at.elapsed().as_secs();
                format!("  <REC> {:02}:{:02}", secs / 60, secs % 60)
            }
            RecState::Finalizing => "  <SAV>".to_string(),
        };
        let max_w = (COLS - 4).saturating_sub(suffix.chars().count());
        hints = hints.chars().take(max_w).collect::<String>() + &suffix;
    } else if let Some(err) = state.last_error.as_deref() {
        hints = format!("ERR: {}", err);
    } else if state.gles_profile == GlesProfile::V100 {
        hints.push_str("  [pi3]");
    }

    let hints: String = hints.chars().take(COLS - 4).collect();
    grid.write(ROW_HOTKEYS, PANE_L0, ATTR_NORMAL, &hints);
}

// ── helpers ──────────────────────────────────────────────────────────────

fn bank_letter(n: u8) -> char {
    (b'A' + (n % 26)) as char
}

fn now_status(state: &SharedState) -> String {
    // Minimal NOW indicator until richer player state is threaded through.
    format!("[{}-0] -", bank_letter(state.bank_number))
}

fn fmt_clock(s: f64) -> String {
    if s < 0.0 {
        return "00:00".to_string();
    }
    let t = s as u64;
    format!("{:02}:{:02}", t / 60, t % 60)
}

/// Clear the left-pane body region (rows ROW_BODY0..=ROW_BODY_END, interior cols).
pub fn clear_left_pane(grid: &mut TextGrid) {
    for r in ROW_BODY0..=ROW_BODY_END {
        grid.fill(r, PANE_L0, PANE_LW, ' ', ATTR_NORMAL);
    }
}

/// Write a header label on the body header row of the left pane.
pub fn left_header(grid: &mut TextGrid, text: &str) {
    let t: String = text.chars().take(PANE_LW).collect();
    grid.write(ROW_BODY0, PANE_L0, ATTR_DIM, &t);
}

/// Write list row `i` (0-based) of the left pane, truncated to pane width.
/// Returns the grid row index used (or None if `i` is out of the pane).
pub fn left_row(grid: &mut TextGrid, i: usize, text: &str, attr: u8) -> Option<usize> {
    if i >= BODY_LIST_ROWS {
        return None;
    }
    let row = ROW_BODY0 + 1 + i;
    let t: String = text.chars().take(PANE_LW).collect();
    grid.write(row, PANE_L0, attr, &t);
    Some(row)
}

/// Invert a left-pane list row `i` for selection (interior cols only).
pub fn invert_left_row(grid: &mut TextGrid, i: usize) {
    let row = ROW_BODY0 + 1 + i;
    grid.invert_span(row, PANE_L0, PANE_DIV);
}

/// Dim a left-pane list row `i` (interior cols only).
pub fn dim_left_row(grid: &mut TextGrid, i: usize) {
    let row = ROW_BODY0 + 1 + i;
    grid.dim_span(row, PANE_L0, PANE_DIV);
}
