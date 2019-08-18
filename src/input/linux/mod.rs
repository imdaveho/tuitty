//! This is a *NIX specific implementation for input handling.
use std::fs;
use std::io::{Result, Error, ErrorKind};
use std::char;
use std::{
    io::{Read, Write},
    str::from_utf8
};
use std::thread;
use std::sync::{
    mpsc::{Receiver, Sender, channel},
    Arc,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::os::unix::io::AsRawFd;
use crate::{csi, write_cout};
use super::*;

mod parser;
use parser::parse_event;


fn _get_systty() -> Result<fs::File> {
    fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
}

fn _get_systty_fd() -> Result<i32> {
    let fd = unsafe {
        if libc::isatty(libc::STDIN_FILENO) == 1 {
            libc::STDIN_FILENO
        } else {
            let tty_f = fs::File::open("/dev/tty")?;
            tty_f.as_raw_fd()
        }
    };
    Ok(fd)
}


pub fn _read_char() -> Result<char> {
    let mut buf = [0u8; 20];

    let fd = _get_systty_fd()?;

    let rv = unsafe {
        let read = libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, 20);

        if read < 0 {
            Err(Error::last_os_error())
        } else {
            let mut pressed_char = Ok(' ');

            if let Ok(s) = from_utf8(&buf[..read as usize]) {
                if let Some(c) = s.chars().next() {
                    pressed_char = Ok(c)
                }
            } else {
                pressed_char = Err(Error::new(
                    ErrorKind::Interrupted,
                    "Could not parse char to utf8 char",
                ));
            }

            return pressed_char;
        }
    };
    return rv;
}

pub fn _read_async() -> AsyncReader {
    AsyncReader::new(Box::new(move |event_tx, kill_switch| {
        for i in _get_systty().unwrap().bytes() {
            if event_tx.send(i.unwrap()).is_err() {
                return;
            }
            if kill_switch.load(Ordering::SeqCst) {
                return;
            }
        }
    }))
}

pub fn _read_until_async(delimiter: u8) -> AsyncReader {
    AsyncReader::new(Box::new(move |event_tx, kill_switch| {
        for byte in _get_systty().unwrap().bytes() {
            let b = byte.unwrap();
            let eos = b == delimiter;
            let send_err = event_tx.send(b).is_err();

            if eos || send_err || kill_switch.load(Ordering::SeqCst) {
                return;
            }
        }
    }))
}

pub fn _read_sync() -> SyncReader {
    SyncReader {
        source: Box::from(_get_systty().unwrap()),
        leftover: None,
    }
}

pub fn _enable_mouse_mode() -> TtyResult<()> {
    write_cout!(&format!(
        "{}h{}h{}h{}h",
        csi!("?1000"),
        csi!("?1002"),
        csi!("?1015"),
        csi!("?1006")
    ))?;
    Ok(())
}

pub fn _disable_mouse_mode() -> TtyResult<()> {
    write_cout!(&format!(
        "{}l{}l{}l{}l",
        csi!("?1006"),
        csi!("?1015"),
        csi!("?1002"),
        csi!("?1000")
    ))?;
    Ok(())
}


pub struct AsyncReader {
    event_rx: Receiver<u8>,
    shutdown: Arc<AtomicBool>,
}

impl AsyncReader {
    pub fn new(func: Box<Fn(
        &Sender<u8>,
        &Arc<AtomicBool>
    ) + Send>) -> AsyncReader {
        let shutdown_handle = Arc::new(AtomicBool::new(false));

        let (event_tx, event_rx) = channel();
        let thread_shutdown = shutdown_handle.clone();

        thread::spawn(move || loop {
            func(&event_tx, &thread_shutdown);
        });

        AsyncReader {
            event_rx,
            shutdown: shutdown_handle,
        }
    }

    pub fn stop_reading(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
    }
}

impl Iterator for AsyncReader {
    type Item = InputEvent;

    fn next(&mut self) -> Option<Self::Item> {
        let mut iterator = self.event_rx.try_iter();

        match iterator.next() {
            Some(char_value) => {
                if let Ok(char_value) = parse_event(char_value, &mut iterator) {
                    Some(char_value)
                } else {
                    None
                }
            }
            None => None,
        }
    }
}

impl Drop for AsyncReader {
    fn drop(&mut self) {
        self.stop_reading();
    }
}


pub struct SyncReader {
    source: Box<std::fs::File>,
    leftover: Option<u8>,
}

impl Iterator for SyncReader {
    type Item = InputEvent;
    // Read input from the user.
    //
    // If there are no keys pressed, this will be a blocking call
    // until there is one.
    // This will return `None` in case of a failure and `Some(InputEvent)`
    // in case of an occurred input event.
    fn next(&mut self) -> Option<Self::Item> {
        // TODO: Currently errors are consumed and converted to
        // a `None`. Maybe we shouldn't be doing this?
        let source = &mut self.source;

        if let Some(c) = self.leftover {
            // we have a leftover byte, use it
            self.leftover = None;
            if let Ok(e) = parse_event(c, &mut source.bytes().flatten()) {
                return Some(e);
            } else {
                return None;
            }
        }

        // Here we read two bytes at a time. We need to distinguish
        // between single ESC key presses,
        // and escape sequences (which start with ESC or a x1B byte).
        // The idea is that if this is
        // an escape sequence, we will read multiple bytes (the first
        // byte being ESC) but if this
        // is a single ESC keypress, we will only read a single byte.
        let mut buf = [0u8; 2];

        let res = match source.read(&mut buf) {
            Ok(0) => return None,
            Ok(1) => match buf[0] {
                b'\x1B' => return Some(InputEvent::Keyboard(KeyEvent::Esc)),
                c => {
                    if let Ok(e) = parse_event(
                        c, &mut source
                            .bytes()
                            .flatten()) {
                        return Some(e);
                    } else {
                        return None;
                    }
                }
            },
            Ok(2) => {
                let option_iter = &mut Some(buf[1]).into_iter();
                let iter = option_iter.map(|c| Ok(c)).chain(source.bytes());
                if let Ok(e) = parse_event(buf[0], &mut iter.flatten()) {
                    self.leftover = option_iter.next();
                    Some(e)
                } else {
                    None
                }
            }
            Ok(_) => unreachable!(),
            Err(_) => return None,
            // maybe we should not throw away the error?
        };
        return res;
    }
}