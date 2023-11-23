pub use slowchop_console_derive::Commands;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("No command given")]
    NoCommandGiven,

    #[error("Unknown command: {0}")]
    UnknownCommand(String),
}
