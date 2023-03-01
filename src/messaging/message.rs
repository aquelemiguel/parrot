use serenity::model::mention::Mention;
use std::fmt::Display;

use crate::messaging::messages::*;

const RELEASES_LINK: &str = "https://github.com/aquelemiguel/parrot/releases";

pub enum ParrotMessage {
    AutopauseOff,
    AutopauseOn,
    Clear,
    Error,
    Leaving,
    LoopDisable,
    LoopEnable,
    NowPlaying,
    Pause,
    PlayAllFailed,
    PlayDomainBanned { domain: String },
    PlaylistQueued,
    RemoveMultiple,
    Resume,
    Search,
    Seek { timestamp: String },
    Shuffle,
    Skip,
    SkipAll,
    SkipTo { title: String, url: String },
    Stop,
    Summon { mention: Mention },
    Version { current: String },
    VoteSkip { mention: Mention, missing: usize },
}

impl Display for ParrotMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AutopauseOff => f.write_str(AUTOPAUSE_OFF),
            Self::AutopauseOn => f.write_str(AUTOPAUSE_ON),
            Self::Clear => f.write_str(CLEARED),
            Self::Error => f.write_str(ERROR),
            Self::Leaving => f.write_str(LEAVING),
            Self::LoopDisable => f.write_str(LOOP_DISABLED),
            Self::LoopEnable => f.write_str(LOOP_ENABLED),
            Self::NowPlaying => f.write_str(QUEUE_NOW_PLAYING),
            Self::Pause => f.write_str(PAUSED),
            Self::PlaylistQueued => f.write_str(PLAY_PLAYLIST),
            Self::PlayAllFailed => f.write_str(PLAY_ALL_FAILED),
            Self::PlayDomainBanned { domain } => {
                f.write_str(&format!("⚠️ **{}** {}", domain, PLAY_FAILED_BLOCKED_DOMAIN))
            }
            Self::Search => f.write_str(SEARCHING),
            Self::RemoveMultiple => f.write_str(REMOVED_QUEUE_MULTIPLE),
            Self::Resume => f.write_str(RESUMED),
            Self::Shuffle => f.write_str(SHUFFLED_SUCCESS),
            Self::Stop => f.write_str(STOPPED),
            Self::VoteSkip { mention, missing } => f.write_str(&format!(
                "{}{} {} {} {}",
                SKIP_VOTE_EMOJI, mention, SKIP_VOTE_USER, missing, SKIP_VOTE_MISSING
            )),
            Self::Seek { timestamp } => f.write_str(&format!("{} **{}**!", SEEKED, timestamp)),
            Self::Skip => f.write_str(SKIPPED),
            Self::SkipAll => f.write_str(SKIPPED_ALL),
            Self::SkipTo { title, url } => {
                f.write_str(&format!("{} [**{}**]({})!", SKIPPED_TO, title, url))
            }
            Self::Summon { mention } => f.write_str(&format!("{} **{}**!", JOINING, mention)),
            Self::Version { current } => f.write_str(&format!(
                "{} [{}]({}/tag/v{})\n{}({}/latest)",
                VERSION, current, RELEASES_LINK, current, VERSION_LATEST, RELEASES_LINK
            )),
        }
    }
}
