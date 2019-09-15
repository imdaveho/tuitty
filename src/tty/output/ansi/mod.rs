// ANSI functions for writing and styling text to be outputted to the terminal.

use super::{csi, Style, Color, Effect, Display};

#[cfg(unix)]
use super::{Error, Result, Termios};
#[cfg(unix)]
mod raw;
#[cfg(unix)]
pub use raw::*;

mod style;
pub use style::*;


pub fn prints<D: Display>(value: D) -> String {
    format!("{}", value)
}
