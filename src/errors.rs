use rspotify::ClientError;
use serenity::{model::misc::Mention, prelude::SerenityError};
use std::fmt::Display;
use std::{error::Error, fmt};

use crate::strings::{
    FAIL_ANOTHER_CHANNEL, FAIL_AUTHOR_DISCONNECTED, FAIL_AUTHOR_NOT_FOUND,
    FAIL_NO_VOICE_CONNECTION, FAIL_WRONG_CHANNEL, NOTHING_IS_PLAYING, QUEUE_IS_EMPTY,
};

#[derive(Debug)]
pub enum ParrotError {
    Other(&'static str),
    QueueEmpty,
    NotInRange(&'static str, isize, isize, isize),
    NotConnected,
    AuthorDisconnected(Mention),
    WrongVoiceChannel,
    AuthorNotFound,
    NothingPlaying,
    AlreadyConnected(Mention),
    Serenity(SerenityError),
    RSpotify(ClientError),
}

impl Error for ParrotError {}

impl Display for ParrotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParrotError::Other(msg) => f.write_str(msg),
            ParrotError::QueueEmpty => f.write_str(QUEUE_IS_EMPTY),
            ParrotError::NotInRange(param, value, lower, upper) => f.write_str(&format!(
                "`{param}` should be between {lower} and {upper} but was {value}"
            )),
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
            ParrotError::Serenity(err) => f.write_str(&format!("{err}")),
            ParrotError::RSpotify(err) => f.write_str(&format!("{err}")),
        }
    }
}

impl From<SerenityError> for ParrotError {
    fn from(err: SerenityError) -> ParrotError {
        match err {
            SerenityError::NotInRange(param, value, lower, upper) => {
                ParrotError::NotInRange(param, value as isize, lower as isize, upper as isize)
            }
            SerenityError::Other(msg) => ParrotError::Other(msg),
            _ => ParrotError::Serenity(err),
        }
    }
}

impl From<ClientError> for ParrotError {
    fn from(err: ClientError) -> ParrotError {
        ParrotError::RSpotify(err)
    }
}

pub trait ToBool {
    fn to_bool(&self) -> bool;
}

impl ToBool for bool {
    fn to_bool(&self) -> bool {
        *self
    }
}

impl<T> ToBool for Option<T> {
    fn to_bool(&self) -> bool {
        self.is_some()
    }
}

impl<T, E> ToBool for Result<T, E> {
    fn to_bool(&self) -> bool {
        self.is_ok()
    }
}

pub fn verify(condition: impl ToBool, err: ParrotError) -> Result<(), ParrotError> {
    if condition.to_bool() {
        Ok(())
    } else {
        Err(err)
    }
}
