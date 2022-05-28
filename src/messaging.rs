use std::fmt::Display;

use serenity::model::misc::Mention;

use crate::strings::*;

#[derive(Debug)]
pub enum Response {
    AutopauseOn,
    AutopauseOff,
    LoopEnabled,
    LoopDisabled,
    Paused,
    Cleared,
    Leaving,
    Searching,
    RemoveMultiple,
    Resume,
    Shuffled,
    Stop,
    VoteSkip(Mention, usize),
}

impl Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AutopauseOn => f.write_str(AUTOPAUSE_ON),
            Self::AutopauseOff => f.write_str(AUTOPAUSE_OFF),
            Self::LoopEnabled => f.write_str(LOOP_ENABLED),
            Self::LoopDisabled => f.write_str(LOOP_DISABLED),
            Self::Paused => f.write_str(PAUSED),
            Self::Cleared => f.write_str(CLEARED),
            Self::Leaving => f.write_str(LEAVING),
            Self::Searching => f.write_str(SEARCHING),
            Self::RemoveMultiple => f.write_str(REMOVED_QUEUE_MULTIPLE),
            Self::Resume => f.write_str(RESUMED),
            Self::Shuffled => f.write_str(SHUFFLED_SUCCESS),
            Self::Stop => f.write_str(STOPPED),
            Self::VoteSkip(mention, missing) => f.write_str(&format!(
                "{}{} {} {} {}",
                SKIP_VOTE_EMOJI, mention, SKIP_VOTE_USER, missing, SKIP_VOTE_MISSING
            )),
        }
    }
}
