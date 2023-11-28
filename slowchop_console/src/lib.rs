#![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(future_incompatible)]
#![warn(missing_debug_implementations)]

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

pub fn parse_bool(s: &str) -> Option<bool> {
    match s.to_lowercase().as_str() {
        "1" | "t" | "true" | "y" | "yes" | "yeah" => Some(true),
        "0" | "f" | "false" | "n" | "no" | "nah" => Some(false),
        _ => None,
    }
}
