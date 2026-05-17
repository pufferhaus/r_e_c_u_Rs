//! Phase 4b — capture recording state + helpers.

use std::path::PathBuf;
use std::time::Instant;

/// Build target for encoder selection. Inferred at compile time by `Target::current()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Pi3,
    Pi5,
    /// macOS desktop — uses VideoToolbox vtenc_h264.
    MacDesktop,
    /// Linux desktop — uses software x264enc.
    LinuxDesktop,
}

impl Target {
    pub fn current() -> Self {
        #[cfg(feature = "pi3")]
        { return Target::Pi3; }
        #[cfg(feature = "pi5")]
        { return Target::Pi5; }
        #[cfg(all(not(feature = "pi3"), not(feature = "pi5"), target_os = "macos"))]
        { return Target::MacDesktop; }
        #[cfg(all(not(feature = "pi3"), not(feature = "pi5"), target_os = "linux"))]
        { return Target::LinuxDesktop; }
        #[cfg(all(not(feature = "pi3"), not(feature = "pi5"), not(target_os = "macos"), not(target_os = "linux")))]
        { return Target::LinuxDesktop; }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecState {
    Recording,
    Finalizing,
}

#[derive(Debug, Clone)]
pub struct ActiveRecording {
    pub device_path: String,
    pub file_path: PathBuf,
    pub started_at: Instant,
    pub state: RecState,
    pub last_disk_check: Instant,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_current_is_one_of_known() {
        let t = Target::current();
        // Just verify it compiles and resolves to a known variant.
        match t {
            Target::Pi3 | Target::Pi5 | Target::MacDesktop | Target::LinuxDesktop => {}
        }
    }

    #[test]
    fn rec_state_distinct() {
        assert_ne!(RecState::Recording, RecState::Finalizing);
    }

    #[test]
    fn active_recording_round_trip_fields() {
        let now = Instant::now();
        let r = ActiveRecording {
            device_path: "/dev/video0".into(),
            file_path: "/tmp/rec.mp4".into(),
            started_at: now,
            state: RecState::Recording,
            last_disk_check: now,
        };
        assert_eq!(r.device_path, "/dev/video0");
        assert_eq!(r.state, RecState::Recording);
    }
}
