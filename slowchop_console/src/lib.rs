pub use slowchop_console_derive::Actions;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("No action given")]
    NoCommandGiven,

    #[error("Unknown action: {0}")]
    UnknownCommand(String),

    #[error("Not enough arguments for action: {0}")]
    NotEnoughArguments(String),

    #[error("Too many arguments for action: {0}")]
    TooManyArguments(String),

    #[error("Parse float error: {0} {1}")]
    ParseFloatError(String, #[source] std::num::ParseFloatError),

    #[error("Parse int error: {0} {1}")]
    ParseIntError(String, #[source] std::num::ParseIntError),
}
