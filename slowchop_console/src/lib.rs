mod error;
mod plugin;
mod subscriber;

pub use error::Error;
pub use plugin::{Console, ConsolePlugin};
pub use slowchop_console_derive::Actions;

pub trait ActionsImpl {
    fn resolve(text: &str) -> Result<Self, Error>
    where
        Self: Sized;
}
