use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("No action given")]
    NoActionGiven,

    #[error("Unknown action: {action}")]
    UnknownAction { action: String },

    #[error("Not enough arguments for action: {action}")]
    NotEnoughArguments { action: String },

    #[error("Too many arguments for action: {0}")]
    TooManyArguments(String),

    #[error("Parse float error: {0} {1}")]
    ParseFloatError(String, #[source] std::num::ParseFloatError),

    #[error("Parse int error: {0} {1}")]
    ParseIntError(String, #[source] std::num::ParseIntError),

    #[error("Parse bool error: {0}")]
    ParseBoolError(String),

    #[error("Bad argument at position:{0} {1}")]
    BadArgument(usize, String),

    #[error("Parse error: {0}")]
    ParseError(winnow::error::ErrMode<winnow::error::ContextError>),
}

impl From<winnow::error::ErrMode<winnow::error::ContextError>> for Error {
    fn from(e: winnow::error::ErrMode<winnow::error::ContextError>) -> Self {
        Error::ParseError(e)
    }
}
