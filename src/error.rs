use std::fmt;

#[derive(Debug)]
pub enum HerdError {
    SingleMonitor,
    InvalidDisplay(u32, u32),
    NoWindowsToMove,
    NoSnapshot,
}

impl fmt::Display for HerdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HerdError::SingleMonitor => {
                write!(f, "Only one display detected. Nothing to herd.")
            }
            HerdError::InvalidDisplay(requested, available) => {
                write!(
                    f,
                    "Display {} does not exist. Available displays: 1-{}",
                    requested, available
                )
            }
            HerdError::NoWindowsToMove => {
                write!(f, "No windows to move.")
            }
            HerdError::NoSnapshot => {
                write!(f, "No previous herd operation to undo.")
            }
        }
    }
}

impl std::error::Error for HerdError {}
