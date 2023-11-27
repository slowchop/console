use crate::Action;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("No action given")]
    NoCommandGiven,

    #[error("Unknown action: {command} with args: {args:?}")]
    UnknownAction { command: String, args: Vec<String> },

    #[error("Not enough arguments for action: {action} expected {expected} given {given}. Full args: {args:?}")]
    NotEnoughArguments {
        action: String,
        args: Vec<String>,
        expected: usize,
        given: usize,
    },

    #[error("Too many arguments for action: {0}")]
    TooManyArguments(String),

    #[error("Parse float error: {0} {1}")]
    ParseFloatError(String, #[source] std::num::ParseFloatError),

    #[error("Parse int error: {0} {1}")]
    ParseIntError(String, #[source] std::num::ParseIntError),

    #[error("Bad argument at position:{0} {1}")]
    BadArgument(usize, String),
}
