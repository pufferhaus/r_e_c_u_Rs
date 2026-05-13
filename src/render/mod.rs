//! Video render stub.
//!
//! This module provides the `Render` struct with the Phase-1 API surface.
//! The implementation is a no-op stub: all methods log a trace-level message
//! and return immediately. A real GL/DRM backend will be fleshed out in a
//! later task.
//!
//! Required API (from the implementation plan):
//! ```ignore
//! pub fn new(/* window or pi target */) -> anyhow::Result<Self>;
//! pub fn begin_frame(&mut self);
//! pub fn draw_video_layer(&mut self, rgba: &[u8], w: u32, h: u32, alpha: f32);
//! pub fn end_frame(&mut self);
//! ```

/// Opaque render handle. Currently a no-op stub.
pub struct Render {
    _private: (),
}

impl Render {
    /// Construct a new `Render`. Always succeeds in the stub implementation.
    pub fn new() -> anyhow::Result<Self> {
        tracing::warn!("Render::new: using stub implementation — no output will be displayed");
        Ok(Self { _private: () })
    }

    /// Begin a new frame. No-op in stub.
    pub fn begin_frame(&mut self) {
        tracing::trace!("Render::begin_frame (stub)");
    }

    /// Draw a video layer. No-op in stub.
    ///
    /// `rgba` must be `w * h * 4` bytes.
    pub fn draw_video_layer(&mut self, _rgba: &[u8], _w: u32, _h: u32, _alpha: f32) {
        tracing::trace!("Render::draw_video_layer (stub)");
    }

    /// End the current frame and present. No-op in stub.
    pub fn end_frame(&mut self) {
        tracing::trace!("Render::end_frame (stub)");
    }
}

impl Default for Render {
    fn default() -> Self {
        Self { _private: () }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_render_constructs_and_cycles() {
        let mut r = Render::new().unwrap();
        r.begin_frame();
        r.draw_video_layer(&[0u8; 4], 1, 1, 1.0);
        r.end_frame();
    }
}
