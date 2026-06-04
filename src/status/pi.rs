//! SPI LCD backend for the 3.5" ILI9486 panel (fb_ili9486 / fbtft driver).
//!
//! Boot config required in `/boot/firmware/config.txt`:
//! ```
//! dtparam=spi=on
//! dtoverlay=fbtft,spi0-0,ili9486,reset_pin=25,dc_pin=24,led_pin=18,\
//!     width=320,height=480,rotate=0,bgr=1,speed=32000000,fps=30
//! ```
//! And in `/boot/firmware/cmdline.txt` (append, no newline):
//! ```
//! spidev.bufsiz=524288
//! ```
//!
//! `rotate=0` is intentional: the fbtft driver's address-window math is
//! buggy on this clone panel for landscape rotations. We keep the
//! framebuffer in native portrait (320×480) and rotate 90° CW in software.

use std::fs::{File, OpenOptions};
use std::path::PathBuf;

use embedded_graphics::pixelcolor::IntoStorage;
use memmap2::{MmapMut, MmapOptions};

use crate::error::{Error, Result};
use crate::status::grid::TextGrid;
use crate::status::render::{Fb, PANEL_H, PANEL_W};

const PANEL_NAME: &str = "fb_ili9486";
const NATIVE_W: usize = PANEL_H; // 320
const NATIVE_H: usize = PANEL_W; // 480
const BYTES_PER_PIXEL: usize = 2;

pub struct PiPanelBackend {
    _file: File,
    map: MmapMut,
    fb: Fb,
    /// Staging buffer in native portrait layout (RGB565 LE). Full frame is built
    /// here, then committed to mmap in one copy_from_slice to minimise the window
    /// in which fbtft DMA can read a partially-updated framebuffer.
    staging: Vec<u8>,
    /// Cell snapshot from the last flush. We skip re-rendering and re-committing
    /// when the grid hasn't changed — eliminates mmap writes on static frames
    /// and removes tearing entirely for idle/static displays.
    prev_cells: Vec<crate::status::grid::Cell>,
}

impl PiPanelBackend {
    pub fn open() -> Result<Self> {
        let path = find_panel_fb()?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .map_err(|e| Error::Other(format!("open {}: {e}", path.display())))?;
        let map = unsafe {
            MmapOptions::new()
                .len(NATIVE_W * NATIVE_H * BYTES_PER_PIXEL)
                .map_mut(&file)
                .map_err(|e| Error::Other(format!("mmap {}: {e}", path.display())))?
        };
        Ok(Self {
            _file: file,
            map,
            fb: Fb::new(),
            staging: vec![0u8; NATIVE_W * NATIVE_H * BYTES_PER_PIXEL],
            prev_cells: Vec::new(),
        })
    }

    /// Render `grid` to the SPI panel — skips the mmap write if the grid
    /// is unchanged since the last flush (eliminates tearing on static frames).
    ///
    /// When the grid does change: builds the full rotated frame in `self.staging`,
    /// then copies it to the mmap in one `copy_from_slice` call. The bulk copy is
    /// ~1 ms vs ~30 ms for pixel-by-pixel writes, closing the window in which the
    /// fbtft driver can read a partially-updated framebuffer and produce
    /// column tearing.
    pub fn flush(&mut self, grid: &TextGrid) {
        // Skip if grid content is unchanged — no mmap write, no DMA race.
        if self.prev_cells == grid.cells() {
            return;
        }
        self.prev_cells.clear();
        self.prev_cells.extend_from_slice(grid.cells());

        crate::status::render::render(grid, &mut self.fb);
        // Build rotated frame in staging buffer.
        // Inverse of CW rotation: native(dx, dy) → landscape(sx=dy, sy=NATIVE_W-1-dx)
        for dy in 0..NATIVE_H {
            for dx in 0..NATIVE_W {
                let sx = dy;
                let sy = NATIVE_W - 1 - dx;
                let raw: u16 = swap_rb565(self.fb.pixel_at(sx, sy).into_storage());
                let bytes = raw.to_le_bytes();
                let off = (dy * NATIVE_W + dx) * BYTES_PER_PIXEL;
                self.staging[off] = bytes[0];
                self.staging[off + 1] = bytes[1];
            }
        }
        // Single bulk transfer to mmap — minimises tearing window.
        self.map.copy_from_slice(&self.staging);
    }
}

fn find_panel_fb() -> Result<PathBuf> {
    for entry in std::fs::read_dir("/sys/class/graphics")
        .map_err(|e| Error::Other(format!("read /sys/class/graphics: {e}")))?
    {
        let entry = entry.map_err(|e| Error::Other(format!("dir entry: {e}")))?;
        let name_path = entry.path().join("name");
        let Ok(name) = std::fs::read_to_string(&name_path) else {
            continue;
        };
        if name.trim() == PANEL_NAME {
            let dev_name = entry.file_name();
            return Ok(PathBuf::from("/dev").join(&dev_name));
        }
    }
    Err(Error::Other(format!(
        "no framebuffer with name '{PANEL_NAME}' found; is fbtft overlay loaded?"
    )))
}

/// Swap R and B 5-bit fields of an Rgb565 word. The MPI3501 ILI9486 clone
/// reads incoming top-5 bits as blue, bottom-5 as red — opposite of the
/// embedded-graphics convention.
#[inline]
fn swap_rb565(raw: u16) -> u16 {
    ((raw & 0x001F) << 11) | (raw & 0x07E0) | ((raw & 0xF800) >> 11)
}

/// Write one landscape source pixel into the rotated native portrait
/// framebuffer. 90° CW: dst_x = NATIVE_W - 1 - sy, dst_y = sx.
#[inline]
fn write_rotated(map: &mut [u8], sx: usize, sy: usize, pixel_le: [u8; 2]) {
    let dx = NATIVE_W - 1 - sy;
    let dy = sx;
    let off = (dy * NATIVE_W + dx) * BYTES_PER_PIXEL;
    map[off] = pixel_le[0];
    map[off + 1] = pixel_le[1];
}
