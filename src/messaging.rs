use std::fmt::Display;

use serenity::model::misc::Mention;

use crate::strings::*;

const RELEASES_LINK: &str = "https://github.com/aquelemiguel/parrot/releases";

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
    VoteSkip { mention: Mention, missing: usize },
    Seek { timestamp: String },
    Skipped,
    SkippedAll,
    SkippedTo { title: String, url: String },
    Summon { mention: Mention },
    Version { current: String },
    Error,
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
            Self::VoteSkip { mention, missing } => f.write_str(&format!(
                "{}{} {} {} {}",
                SKIP_VOTE_EMOJI, mention, SKIP_VOTE_USER, missing, SKIP_VOTE_MISSING
            )),
            Self::Seek { timestamp } => f.write_str(&format!("{} **{}**!", SEEKED, timestamp)),
            Self::Skipped => f.write_str(SKIPPED),
            Self::SkippedAll => f.write_str(SKIPPED_ALL),
            Self::SkippedTo { title, url } => {
                f.write_str(&format!("{} [**{}**]({})!", SKIPPED_TO, title, url))
            }
            Self::Summon { mention } => f.write_str(&format!("{} **{}**!", JOINING, mention)),
            Self::Version { current } => f.write_str(&format!(
                "{} [{}]({}/tag/v{})\n{}({}/latest)",
                VERSION, current, RELEASES_LINK, current, VERSION_LATEST, RELEASES_LINK
            )),
            Self::Error => f.write_str(ERROR),
        }
    }
}
