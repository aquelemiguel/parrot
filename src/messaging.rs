use std::fmt::Display;

use crate::strings::{AUTOPAUSE_OFF, AUTOPAUSE_ON, LOOP_DISABLED, LOOP_ENABLED, PAUSED};

#[derive(Debug)]
pub enum Response {
    AutopauseOn,
    AutopauseOff,
    LoopEnabled,
    LoopDisabled,
    Paused,
}

impl Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AutopauseOn => f.write_str(AUTOPAUSE_ON),
            Self::AutopauseOff => f.write_str(AUTOPAUSE_OFF),
            Self::LoopEnabled => f.write_str(LOOP_ENABLED),
            Self::LoopDisabled => f.write_str(LOOP_DISABLED),
            Self::Paused => f.write_str(PAUSED),
        }
    }
}
