use serenity::{model::misc::Mention, prelude::SerenityError};
use std::error::Error;
use std::fmt::Display;

use crate::strings::{
    FAIL_ANOTHER_CHANNEL, FAIL_AUTHOR_DISCONNECTED, FAIL_AUTHOR_NOT_FOUND,
    FAIL_NO_VOICE_CONNECTION, FAIL_WRONG_CHANNEL, NOTHING_IS_PLAYING, QUEUE_IS_EMPTY,
};

#[derive(Debug)]
pub enum ParrotError {
    Other(&'static str),
    QueueEmpty,
    NotConnected,
    AuthorDisconnected(Mention),
    WrongVoiceChannel,
    AuthorNotFound,
    NothingPlaying,
    AlreadyConnected(Mention),
    Serenity(SerenityError),
}

impl Error for ParrotError {}

impl Display for ParrotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParrotError::Other(msg) => f.write_str(msg),
            ParrotError::Serenity(err) => f.write_str(&format!("{err}")),
            ParrotError::QueueEmpty => f.write_str(QUEUE_IS_EMPTY),
            ParrotError::NotConnected => f.write_str(FAIL_NO_VOICE_CONNECTION),
            ParrotError::AuthorDisconnected(mention) => {
                f.write_fmt(format_args!("{} {}", FAIL_AUTHOR_DISCONNECTED, mention))
            }
            ParrotError::WrongVoiceChannel => f.write_str(FAIL_WRONG_CHANNEL),
            ParrotError::AuthorNotFound => f.write_str(FAIL_AUTHOR_NOT_FOUND),
            ParrotError::AlreadyConnected(mention) => {
                f.write_fmt(format_args!("{} {}", FAIL_ANOTHER_CHANNEL, mention))
            }
            ParrotError::NothingPlaying => f.write_str(NOTHING_IS_PLAYING),
        }
    }
}

impl From<SerenityError> for ParrotError {
    fn from(err: SerenityError) -> ParrotError {
        ParrotError::Serenity(err)
    }
}

pub fn verify(condition: bool, err: ParrotError) -> Result<(), ParrotError> {
    if condition {
        Ok(())
    } else {
        Err(err)
    }
}
