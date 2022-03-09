use crate::strings::{
    FAIL_ANOTHER_CHANNEL, FAIL_AUTHOR_DISCONNECTED, FAIL_AUTHOR_NOT_FOUND,
    FAIL_NO_VOICE_CONNECTION, FAIL_WRONG_CHANNEL, NOTHING_IS_PLAYING, QUEUE_IS_EMPTY,
};
use serenity::{model::misc::Mention, prelude::SerenityError};
use std::fmt::{Debug, Display};
use std::{error::Error, fmt};

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
}

/// `ParrotError` implements the `Debug` and `Display` traits
/// meaning it implements the `Error` trait. This just makes it explicit.
impl Error for ParrotError {}

impl Display for ParrotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Other(msg) => f.write_str(msg),
            Self::QueueEmpty => f.write_str(QUEUE_IS_EMPTY),
            Self::NotInRange(param, value, lower, upper) => f.write_str(&format!(
                "`{param}` should be between {lower} and {upper} but was {value}"
            )),
            Self::NotConnected => f.write_str(FAIL_NO_VOICE_CONNECTION),
            Self::AuthorDisconnected(mention) => {
                f.write_fmt(format_args!("{} {}", FAIL_AUTHOR_DISCONNECTED, mention))
            }
            Self::WrongVoiceChannel => f.write_str(FAIL_WRONG_CHANNEL),
            Self::AuthorNotFound => f.write_str(FAIL_AUTHOR_NOT_FOUND),
            Self::AlreadyConnected(mention) => {
                f.write_fmt(format_args!("{} {}", FAIL_ANOTHER_CHANNEL, mention))
            }
            Self::NothingPlaying => f.write_str(NOTHING_IS_PLAYING),
            Self::Serenity(err) => f.write_str(&format!("{err}")),
        }
    }
}

impl PartialEq for ParrotError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Other(l0), Self::Other(r0)) => l0 == r0,
            (Self::NotInRange(l0, l1, l2, l3), Self::NotInRange(r0, r1, r2, r3)) => {
                l0 == r0 && l1 == r1 && l2 == r2 && l3 == r3
            }
            (Self::AuthorDisconnected(l0), Self::AuthorDisconnected(r0)) => {
                l0.to_string() == r0.to_string()
            }
            (Self::AlreadyConnected(l0), Self::AlreadyConnected(r0)) => {
                l0.to_string() == r0.to_string()
            }
            (Self::Serenity(l0), Self::Serenity(r0)) => format!("{l0:?}") == format!("{r0:?}"),
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl From<SerenityError> for ParrotError {
    fn from(err: SerenityError) -> Self {
        match err {
            SerenityError::NotInRange(param, value, lower, upper) => {
                Self::NotInRange(param, value as isize, lower as isize, upper as isize)
            }
            SerenityError::Other(msg) => Self::Other(msg),
            _ => Self::Serenity(err),
        }
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

pub trait Unpackable<T> {
    fn unpack(self) -> T;
}

impl Unpackable<bool> for bool {
    fn unpack(self) -> bool {
        self
    }
}

impl<T> Unpackable<T> for Option<T> {
    fn unpack(self) -> T {
        self.unwrap()
    }
}

impl<T, E> Unpackable<T> for Result<T, E>
where
    E: Debug,
{
    fn unpack(self) -> T {
        self.unwrap()
    }
}

pub fn verify<K, T: ToBool + Unpackable<K>>(
    condition: T,
    err: ParrotError,
) -> Result<K, ParrotError> {
    if condition.to_bool() {
        Ok(condition.unpack())
    } else {
        Err(err)
    }
}
