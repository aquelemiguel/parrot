use serenity::{framework::standard::CommandError, prelude::SerenityError};
use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum ParrotError {
    Serenity(SerenityError),
    Command(CommandError),
}

unsafe impl Send for ParrotError {}

unsafe impl Sync for ParrotError {}

impl Display for ParrotError {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        write!(formatter, "TEST")
    }
}

impl std::error::Error for ParrotError {}

impl From<SerenityError> for ParrotError {
    fn from(err: SerenityError) -> ParrotError {
        ParrotError::Serenity(err)
    }
}
