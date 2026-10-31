//! Keeping the screen awake while a score is open (READ-9).
//!
//! Windows: `SetThreadExecutionState` with `ES_DISPLAY_REQUIRED`, held by the calling thread
//! (the UI thread) until released. Other platforms report [`PowerError::Unsupported`] until
//! their backends land (macOS IOPMAssertion, Linux idle inhibit, Android FLAG_KEEP_SCREEN_ON,
//! iOS idleTimerDisabled, the web Screen Wake Lock API).

use std::fmt;

/// Why the screen can't be kept awake.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PowerError {
    /// No backend on this platform yet.
    Unsupported,
    /// The platform refused.
    Failed(String),
}

impl fmt::Display for PowerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PowerError::Unsupported => {
                f.write_str("keeping the screen awake isn't supported on this platform yet")
            }
            PowerError::Failed(m) => write!(f, "couldn't keep the screen awake: {m}"),
        }
    }
}

impl std::error::Error for PowerError {}

/// Holds the screen awake while `on`. Dropping it lets the screen sleep again.
#[derive(Debug, Default)]
pub struct KeepAwake {
    on: bool,
}

impl KeepAwake {
    pub fn new() -> KeepAwake {
        KeepAwake { on: false }
    }

    pub fn is_on(&self) -> bool {
        self.on
    }

    /// Turns keep-awake on or off. Call from the thread that will keep calling it (the UI
    /// thread): Windows ties the request to the thread.
    pub fn set(&mut self, on: bool) -> Result<(), PowerError> {
        if on == self.on {
            return Ok(());
        }
        backend::set(on)?;
        self.on = on;
        Ok(())
    }
}

impl Drop for KeepAwake {
    fn drop(&mut self) {
        if self.on {
            let _ = backend::set(false);
        }
    }
}

#[cfg(all(windows, feature = "power"))]
mod backend {
    use super::PowerError;
    use windows_sys::Win32::System::Power::{
        ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED, SetThreadExecutionState,
    };

    pub fn set(on: bool) -> Result<(), PowerError> {
        let flags = if on {
            ES_CONTINUOUS | ES_DISPLAY_REQUIRED | ES_SYSTEM_REQUIRED
        } else {
            ES_CONTINUOUS
        };
        // SAFETY: SetThreadExecutionState takes a flags value and touches no memory of ours; any
        // combination of ES_* flags is valid. It returns 0 on failure.
        #[allow(unsafe_code)]
        let previous = unsafe { SetThreadExecutionState(flags) };
        if previous == 0 {
            Err(PowerError::Failed(
                "SetThreadExecutionState returned 0".into(),
            ))
        } else {
            Ok(())
        }
    }
}

#[cfg(not(all(windows, feature = "power")))]
mod backend {
    use super::PowerError;

    pub fn set(on: bool) -> Result<(), PowerError> {
        if on {
            Err(PowerError::Unsupported)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turning_off_when_off_is_fine() {
        let mut k = KeepAwake::new();
        assert!(k.set(false).is_ok());
        assert!(!k.is_on());
    }

    #[cfg(not(windows))]
    #[test]
    fn unsupported_elsewhere_and_stays_off() {
        let mut k = KeepAwake::new();
        assert_eq!(k.set(true), Err(PowerError::Unsupported));
        assert!(!k.is_on());
    }
}
