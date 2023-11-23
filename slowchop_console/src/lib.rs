pub use slowchop_console_derive::Actions;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("No command given")]
    NoCommandGiven,

    #[error("Unknown command: {0}")]
    UnknownCommand(String),

    #[error("Not enough arguments for command: {0}")]
    NotEnoughArguments(String),

    #[error("Parse error: {0} {1}")]
    ParseFloatError(String, #[source] std::num::ParseFloatError),
}
