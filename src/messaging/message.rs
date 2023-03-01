use serenity::model::mention::Mention;

use crate::messaging::{locale::localize, messages::*};

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

impl ParrotMessage {
    pub fn localize(&self, locale: &str) -> String {
        match self {
            Self::AutopauseOff => localize("AUTOPAUSE_OFF", locale),
            Self::AutopauseOn => localize("AUTOPAUSE_ON", locale),
            Self::Clear => localize("CLEARED", locale),
            Self::Error => localize("ERROR", locale),
            Self::Leaving => localize("LEAVING", locale),
            Self::LoopDisable => localize("LOOP_DISABLED", locale),
            Self::LoopEnable => localize("LOOP_ENABLED", locale),
            Self::NowPlaying => localize("QUEUE_NOW_PLAYING", locale),
            Self::Pause => localize("PAUSED", locale),
            Self::PlaylistQueued => localize("PLAY_PLAYLIST", locale),
            Self::PlayAllFailed => localize("PLAY_ALL_FAILED", locale),
            Self::PlayDomainBanned { domain } => {
                format!(
                    "⚠️ **{}** {}",
                    domain,
                    localize("PLAY_FAILED_BLOCKED_DOMAIN", locale)
                )
            }
            Self::Search => localize("SEARCHING", locale),
            Self::RemoveMultiple => localize("REMOVED_QUEUE_MULTIPLE", locale),
            Self::Resume => localize("RESUMED", locale),
            Self::Shuffle => localize("SHUFFLED_SUCCESS", locale),
            Self::Stop => localize("STOPPED", locale),
            Self::VoteSkip { mention, missing } => format!(
                "{}{} {} {} {}",
                localize(SKIP_VOTE_EMOJI, locale),
                mention,
                localize(SKIP_VOTE_USER, locale),
                missing,
                localize(SKIP_VOTE_MISSING, locale)
            ),
            Self::Seek { timestamp } => format!("{} **{}**!", localize(SEEKED, locale), timestamp),
            Self::Skip => localize("SKIPPED", locale),
            Self::SkipAll => localize("SKIPPED_ALL", locale),
            Self::SkipTo { title, url } => format!(
                "{} [**{}**]({})!",
                localize("SKIPPED_TO", locale),
                title,
                url
            ),
            Self::Summon { mention } => format!("{} **{}**!", localize("JOINING", locale), mention),
            Self::Version { current } => format!(
                "{} [{}]({}/tag/v{})\n{}({}/latest)",
                localize(VERSION, locale),
                current,
                RELEASES_LINK,
                current,
                VERSION_LATEST,
                RELEASES_LINK
            ),
        }
    }
}
