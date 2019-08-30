// Shared code and abstractions that is leveraged by the other modules.
// * `ansi`, a simple macro to help with writing escape sequences to stdout.
// * `wincon`, wrappers for pointers to the Handle and ConsoleInfo objects.

mod ansi;
pub use ansi::{write_ansi, flush_ansi};

#[cfg(windows)]
mod wincon;

#[cfg(windows)]
pub use wincon::{Handle, ConsoleInfo};
