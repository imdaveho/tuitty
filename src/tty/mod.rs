//! The `tty` module wraps the various components that make up a terminal. These
//! are represented by the sub-modules: `cursor`, `screen`, `input`, `output`.
//! The `Tty` struct is meant to be a thin abstraction to standardize between
//! operating systems and APIs (ANSI vs Windows Console).

#[cfg(unix)]
use libc::termios as Termios;

#[cfg(windows)]
pub type Termios = u32;

mod cursor;
mod input;
mod output;
mod screen;
mod shared;

#[cfg(unix)]
use input::ansi::{AsyncReader, SyncReader};

#[cfg(windows)]
use input::wincon::{AsyncReader, SyncReader};

#[cfg(windows)]
use shared::{Handle, ConsoleInfo};


#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use unix::Tty;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::Tty;

