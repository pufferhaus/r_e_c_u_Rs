//! Phase 4b — capture recording state + helpers.

use std::path::{Path, PathBuf};
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

/// Returns the first non-colliding path `<dir>/rec-YYYY-MM-DD-N.mp4`.
/// `N` starts at 0 and increments until the candidate does not exist.
/// Pure of side-effects except for `Path::exists()` checks.
///
/// `date_yyyymmdd` is injected so tests don't depend on system clock.
pub fn generate_recording_path(dir: &Path, date_yyyymmdd: &str) -> PathBuf {
    let mut n = 0u32;
    loop {
        let candidate = dir.join(format!("rec-{date_yyyymmdd}-{n}.mp4"));
        if !candidate.exists() {
            return candidate;
        }
        n += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn generate_recording_path_starts_at_zero() {
        let td = TempDir::new().unwrap();
        let p = generate_recording_path(td.path(), "2026-05-17");
        assert_eq!(p.file_name().unwrap().to_str(), Some("rec-2026-05-17-0.mp4"));
    }

    #[test]
    fn generate_recording_path_increments_on_collision() {
        let td = TempDir::new().unwrap();
        fs::write(td.path().join("rec-2026-05-17-0.mp4"), b"").unwrap();
        fs::write(td.path().join("rec-2026-05-17-1.mp4"), b"").unwrap();
        let p = generate_recording_path(td.path(), "2026-05-17");
        assert_eq!(p.file_name().unwrap().to_str(), Some("rec-2026-05-17-2.mp4"));
    }

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
