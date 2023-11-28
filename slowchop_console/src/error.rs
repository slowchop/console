#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("No action given")]
    NoCommandGiven,

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
}
