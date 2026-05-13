//! Variable-size character grid with attribute byte per cell.
//!
//! Attribute byte:
//!   bit 0  bright
//!   bit 1  dim
//!   bit 2  inverse (swap fg/bg)
//!   bits 3-7 reserved

pub const ATTR_NORMAL: u8 = 0;
pub const ATTR_BRIGHT: u8 = 1 << 0;
pub const ATTR_DIM: u8 = 1 << 1;
pub const ATTR_INVERSE: u8 = 1 << 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub attr: u8,
}

impl Cell {
    pub const BLANK: Cell = Cell {
        ch: ' ',
        attr: ATTR_NORMAL,
    };
    pub fn new(ch: char, attr: u8) -> Self {
        Self { ch, attr }
    }
}

/// Variable-dimension text grid, row-major.
#[derive(Debug, Clone)]
pub struct TextGrid {
    pub cols: usize,
    pub rows: usize,
    cells: Vec<Cell>,
}

impl TextGrid {
    pub fn new(cols: usize, rows: usize) -> Self {
        Self {
            cols,
            rows,
            cells: vec![Cell::BLANK; cols * rows],
        }
    }

    pub fn clear(&mut self) {
        self.cells.fill(Cell::BLANK);
    }

    /// Write text into `row`, starting at column 0. Truncates at col boundary.
    pub fn write_row(&mut self, row: usize, text: &str) {
        if row >= self.rows {
            return;
        }
        let base = row * self.cols;
        for (col, ch) in text.chars().enumerate() {
            if col >= self.cols {
                break;
            }
            self.cells[base + col] = Cell::new(ch, ATTR_NORMAL);
        }
    }

    /// Invert every cell in `row` (flips ATTR_INVERSE bit).
    pub fn invert_row(&mut self, row: usize) {
        if row >= self.rows {
            return;
        }
        let base = row * self.cols;
        for col in 0..self.cols {
            self.cells[base + col].attr ^= ATTR_INVERSE;
        }
    }

    // --- Lower-level helpers used by status compose and tests ---

    pub fn at(&self, row: usize, col: usize) -> Cell {
        debug_assert!(row < self.rows && col < self.cols);
        self.cells[row * self.cols + col]
    }

    pub fn set(&mut self, row: usize, col: usize, cell: Cell) {
        if row < self.rows && col < self.cols {
            self.cells[row * self.cols + col] = cell;
        }
    }

    /// Write string at (row, col) with attr. Truncates at row boundary.
    pub fn write(&mut self, row: usize, col: usize, attr: u8, s: &str) {
        if row >= self.rows {
            return;
        }
        let mut c = col;
        for ch in s.chars() {
            if c >= self.cols {
                break;
            }
            self.set(row, c, Cell::new(ch, attr));
            c += 1;
        }
    }

    /// Fill a horizontal span of cells.
    pub fn fill(&mut self, row: usize, col_lo: usize, len: usize, ch: char, attr: u8) {
        if row >= self.rows {
            return;
        }
        for c in col_lo..(col_lo + len).min(self.cols) {
            self.set(row, c, Cell::new(ch, attr));
        }
    }

    /// All cells as a flat slice (row-major).
    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_grid_all_blank() {
        let g = TextGrid::new(10, 5);
        for row in 0..5 {
            for col in 0..10 {
                assert_eq!(g.at(row, col), Cell::BLANK);
            }
        }
    }

    #[test]
    fn write_row_sets_text() {
        let mut g = TextGrid::new(20, 4);
        g.write_row(1, "HELLO");
        assert_eq!(g.at(1, 0).ch, 'H');
        assert_eq!(g.at(1, 4).ch, 'O');
        assert_eq!(g.at(1, 5), Cell::BLANK);
    }

    #[test]
    fn write_row_truncates() {
        let mut g = TextGrid::new(4, 2);
        g.write_row(0, "ABCDEF");
        assert_eq!(g.at(0, 3).ch, 'D');
        // col 4 doesn't exist
    }

    #[test]
    fn clear_resets_grid() {
        let mut g = TextGrid::new(5, 3);
        g.write_row(0, "HI");
        g.clear();
        assert_eq!(g.at(0, 0), Cell::BLANK);
    }

    #[test]
    fn invert_row_flips_bit() {
        let mut g = TextGrid::new(5, 3);
        g.write_row(1, "AB");
        g.invert_row(1);
        assert!(g.at(1, 0).attr & ATTR_INVERSE != 0);
        assert!(g.at(1, 1).attr & ATTR_INVERSE != 0);
        // row 0 untouched
        assert!(g.at(0, 0).attr & ATTR_INVERSE == 0);
    }

    #[test]
    fn invert_twice_restores_normal() {
        let mut g = TextGrid::new(5, 2);
        g.invert_row(0);
        g.invert_row(0);
        assert_eq!(g.at(0, 0).attr & ATTR_INVERSE, 0);
    }

    #[test]
    fn out_of_bounds_row_ignored() {
        let mut g = TextGrid::new(5, 3);
        g.write_row(99, "CRASH");
        g.invert_row(99);
        // Just checks no panic
    }
}
