mod actions;
mod error;
mod plugin;

pub use error::Error;
pub use plugin::{Action, Console, ConsolePlugin};
pub use slowchop_console_derive::Actions;
