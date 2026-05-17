//! Live capture sources (Phase 4 — captur).

pub mod device;
pub use device::{CaptureDevice, enumerate_capture_devices};

pub mod recording;
