//! CPU-side renderer: TextGrid → 480×320 RGB565 framebuffer for SPI panel.

use embedded_graphics::mono_font::ascii::FONT_9X18;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Baseline, Text, TextStyleBuilder};

use crate::status::grid::{TextGrid, ATTR_BRIGHT, ATTR_DIM, ATTR_INVERSE};

pub const PANEL_W: usize = 480;
pub const PANEL_H: usize = 320;
pub const CELL_W: usize = 9;
pub const CELL_H: usize = 18;

const FG_NORMAL: Rgb565 = Rgb565::new(31, 36, 0);
const FG_BRIGHT: Rgb565 = Rgb565::new(31, 48, 0);
const FG_DIM: Rgb565 = Rgb565::new(31, 0, 0);
const BG: Rgb565 = Rgb565::new(0, 0, 0);

/// 480×320 RGB565 in-memory framebuffer.
pub struct Fb {
    pub data: Vec<Rgb565>,
}

impl Fb {
    pub fn new() -> Self {
        Self {
            data: vec![BG; PANEL_W * PANEL_H],
        }
    }

    pub fn pixel_at(&self, x: usize, y: usize) -> Rgb565 {
        self.data[y * PANEL_W + x]
    }

    fn set_pixel(&mut self, x: usize, y: usize, c: Rgb565) {
        if x < PANEL_W && y < PANEL_H {
            self.data[y * PANEL_W + x] = c;
        }
    }

    fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, c: Rgb565) {
        for yy in y..(y + h).min(PANEL_H) {
            for xx in x..(x + w).min(PANEL_W) {
                self.set_pixel(xx, yy, c);
            }
        }
    }
}

impl Default for Fb {
    fn default() -> Self {
        Self::new()
    }
}

impl OriginDimensions for Fb {
    fn size(&self) -> Size {
        Size::new(PANEL_W as u32, PANEL_H as u32)
    }
}

impl DrawTarget for Fb {
    type Color = Rgb565;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(p, c) in pixels {
            if p.x >= 0 && p.y >= 0 {
                self.set_pixel(p.x as usize, p.y as usize, c);
            }
        }
        Ok(())
    }
}

fn resolve(attr: u8) -> (Rgb565, Rgb565) {
    let bright = attr & ATTR_BRIGHT != 0;
    let dim = attr & ATTR_DIM != 0;
    let inverse = attr & ATTR_INVERSE != 0;
    let fg = if dim {
        FG_DIM
    } else if bright {
        FG_BRIGHT
    } else {
        FG_NORMAL
    };
    if inverse {
        (BG, fg)
    } else {
        (fg, BG)
    }
}

fn substitute(ch: char) -> char {
    match ch {
        '─' | '━' => '-',
        '│' | '┃' => '|',
        '┌' | '┐' | '└' | '┘' | '├' | '┤' | '┬' | '┴' | '┼' => '+',
        '═' => '=',
        '█' | '▌' | '▐' | '▀' | '▄' => '#',
        '·' => '.',
        '↺' => '~',
        '→' => '>',
        '←' => '<',
        c if c.is_ascii() => c,
        _ => '?',
    }
}

/// Render the full `TextGrid` into `fb`.
pub fn render(grid: &TextGrid, fb: &mut Fb) {
    fb.data.fill(BG);
    let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();
    for row in 0..grid.rows {
        for col in 0..grid.cols {
            let cell = grid.at(row, col);
            let (fg, bg) = resolve(cell.attr);
            let x = col * CELL_W;
            let y = row * CELL_H;
            fb.fill_rect(x, y, CELL_W, CELL_H, bg);
            let ch = substitute(cell.ch);
            if ch == ' ' {
                continue;
            }
            let style = MonoTextStyle::new(&FONT_9X18, fg);
            let _ = Text::with_text_style(
                &ch.to_string(),
                Point::new(x as i32, y as i32),
                style,
                text_style,
            )
            .draw(fb);
        }
    }
}
