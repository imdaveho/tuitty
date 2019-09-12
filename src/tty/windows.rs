//! This module consolidates the parts that make up a TTY into a single struct
//! and exposes the methods that work specifically on Windows systems.
//!
//! Additionally, it provides logic to identify the type of terminal being used
//! on the Windows system (eg. xterm, MinTTY, Cygwin, etc.) as well as check if
//! the terminal has support for ANSI sequences.

use super::screen;
use super::cursor;
use super::output;
use super::input;
use super::shared::{write_ansi, flush_ansi};
use super::{AsyncReader, SyncReader, Termios};

#[cfg(windows)]
use super::{Handle, ConsoleInfo};


pub struct Tty {
    index: usize,
    metas: Vec<Metadata>,
    original_mode: Termios,
    ansi_supported: bool,
    autoflush: bool,
    altscreen: Option<Handle>,
    reset_attrs: u16,
}

pub struct Metadata {
    is_raw_enabled: bool,
    is_mouse_enabled: bool,
    is_cursor_visible: bool,
    saved_position: Option<(i16, i16)>,
}

impl Tty {

    pub fn init() -> Tty {
        Tty {
            index: 0,
            metas: vec![Metadata {
                is_raw_enabled: false,
                is_mouse_enabled: false,
                is_cursor_visible: true,
                saved_position: None,
            }],
            original_mode: {
                if !_is_wincon_supported() {
                    Handle::conout().unwrap()
                        .get_mode().unwrap()
                } else {
                    output::wincon::get_mode().unwrap()
                }
            },
            ansi_supported: _is_ansi_supported(),
            autoflush: false,

            altscreen: None,
            reset_attrs: ConsoleInfo::of(
                &Handle::conout().unwrap()
            ).unwrap().attributes(),
        }
    }

    pub fn terminate(&mut self) {
        let handle = match _is_wincon_supported() {
            true => Handle::stdout().unwrap(),
            false => Handle::conout().unwrap(),
        };

        self.to_main();

        if self.ansi_supported {
            handle.set_mode(&self.original_mode).unwrap();
            write_ansi(&cursor::ansi::show());
            write_ansi("\n\r");
            self.flush();
        } else {
            handle.set_mode(&self.original_mode).unwrap();
            if let Some(handle) = &self.altscreen {
                handle.close().unwrap();
            }
            self.altscreen = None;
            cursor::wincon::show().unwrap();
            self.write("\n\r");
        }
        self.metas.clear();
    }

    pub fn size(&self) -> (i16, i16) {
        // Windows Console API only (no ANSI equivalent).
        screen::wincon::size()
    }

    // "cooked" vs "raw" mode terminology from Wikipedia:
    // https://en.wikipedia.org/wiki/Terminal_mode
    // A terminal mode is one of a set of possible states of a terminal or
    // pseudo terminal character device in Unix-like systems and determines how
    // characters written to the terminal are interpreted. In cooked mode data
    // is preprocessed before being given to a program, while raw mode passes
    // data as-is to the program without interpreting any special characters.
    pub fn raw(&mut self) {
        // Windows Console API only (no ANSI equivalent).
        let mut m = &mut self.metas[self.index];
        output::wincon::enable_raw().unwrap();
        m.is_raw_enabled = true;
    }

    pub fn cook(&mut self) {
        // Windows Console API only (no ANSI equivalent).
        let mut m = &mut self.metas[self.index];
        output::wincon::disable_raw().unwrap();
        m.is_raw_enabled = false;
    }

    // Input module functions are OS-specific.
    // * enable/disable mouse
    // * read_char/sync/async/until_async

    pub fn enable_mouse(&mut self) {
        let mut m = &mut self.metas[self.index];
        input::wincon::enable_mouse_mode().unwrap();
        m.is_mouse_enabled = true;
    }

    pub fn disable_mouse(&mut self) {
        let mut m = &mut self.metas[self.index];
        input::wincon::disable_mouse_mode().unwrap();
        m.is_mouse_enabled = false;
    }

    pub fn read_char(&self) -> char {
        input::wincon::read_char().unwrap()
    }

    pub fn read_sync(&self) -> SyncReader {
        input::wincon::read_sync()
    }

    pub fn read_async(&self) -> AsyncReader {
        input::wincon::read_async()
    }

    pub fn read_until_async(&self, delimiter: u8) -> AsyncReader {
        input::wincon::read_until_async(delimiter)
    }


    pub fn clear(&mut self, method: &str) {
        match method {
            "all" => {
                if self.ansi_supported {
                    write_ansi(&&screen::ansi::clear(screen::Clear::All));
                    self.goto(0, 0);
                } else {
                    screen::wincon::clear(screen::Clear::All).unwrap();
                    self.goto(0, 0);
                }
            }
            "newln" => {
                if self.ansi_supported {
                    write_ansi(&&screen::ansi::clear(screen::Clear::NewLn));
                } else {
                    let (col, row) = cursor::wincon::pos().unwrap();
                    screen::wincon::clear(screen::Clear::NewLn).unwrap();
                    self.goto(col, row);
                }
            }
            "currentln" => {
                if self.ansi_supported {
                    write_ansi(&&screen::ansi::clear(screen::Clear::CurrentLn));
                    let (_, row) = self.pos();
                    self.goto(0, row);
                } else {
                    let (_, row) = cursor::wincon::pos().unwrap();
                    screen::wincon::clear(screen::Clear::CurrentLn).unwrap();
                    self.goto(0, row);
                }
            }
            "cursorup" => {
                if self.ansi_supported {
                    write_ansi(&&screen::ansi::clear(screen::Clear::CursorUp));
                } else {
                    screen::wincon::clear(screen::Clear::CursorUp).unwrap();
                }
            }
            "cursordn" => {
                if self.ansi_supported {
                    write_ansi(&&screen::ansi::clear(screen::Clear::CursorDn));
                } else {
                    screen::wincon::clear(screen::Clear::CursorDn).unwrap();
                }
            }
            _ => ()
        }
        if self.autoflush { self.flush() }
    }

    pub fn resize(&mut self, w: i16, h: i16) {
        if self.ansi_supported {
            write_ansi(&screen::ansi::resize(w, h));
            // (imdaveho) NOTE: In order to resize the terminal, this method
            // must call `flush()` otherwise nothing happens.
            self.flush();
        } else {
            screen::wincon::resize(w, h).unwrap();
        }
    }

    pub fn manual(&mut self) {
        self.autoflush = false;
    }

    pub fn automatic(&mut self) {
        self.autoflush = true;
    }

    pub fn switch(&mut self) {
        // In order to support multiple "screens", this function creates a new
        // Metadata entry which stores any screen specific changes that a User
        // might want to be restored when switching between screens.
        if self.ansi_supported {
            if self.index == 0 {
                // There is no point to switch if you're on another screen
                // since ANSI terminals provide a single "alternate screen".
                write_ansi(&screen::ansi::enable_alt());
            }
            // Add new `Metadata` for the new screen.
            self._add_metadata();
            self.index = self.metas.len() - 1;
            // Prevent multiple `flush()` calls due to `autoflush` setting.
            let autoflush = self.autoflush;
            if self.autoflush { self.manual() }
            // Explicitly set default screen settings. (ANSI-only)
            self.cook();
            self.disable_mouse();
            self.show_cursor();
            self.goto(0, 0);

            if autoflush {
                self.flush();
                // Revert back to previous `autoflush` configuration.
                self.automatic();
            }
        } else {
            if self.altscreen.is_none() {
                self.altscreen = Some(Handle::buffer().unwrap());
            }
            if let Some(handle) = &self.altscreen {
                handle.set_mode(&self.original_mode).unwrap();
                if self.id == 0 {
                    // There is a single handle for the alternate screen buffer;
                    // so only if you're on id == 0 or the main screen, do you
                    // need to enable the alternate.
                    handle.show().unwrap();
                }
                // Add new `Metadata` for the new screen.
                self._add_metadata();
                self.index = self.metas.len() - 1;
                self.show_cursor();
                self.goto(0, 0);
            }
        }
    }

    pub fn to_main(&mut self) {
        // Only execute if the User is not on the main screen buffer.
        if self.index != 0 {
            if self.ansi_supported {
                let metas = &self.metas;
                let rstate = metas[0].is_raw_enabled;
                let mstate = metas[0].is_mouse_enabled;
                let cstate = metas[0].is_cursor_visible;
                self.id = 0;
                write_ansi(&screen::ansi::disable_alt());

                // Prevent multiple `flush()` calls due to `autoflush` setting.
                let autoflush = self.autoflush;
                if self.autoflush { self.manual() }

                if rstate {
                    self.raw();
                } else {
                    self.cook();
                }

                if mstate {
                    self.enable_mouse();
                } else {
                    self.disable_mouse();
                }

                if cstate {
                    self.show_cursor();
                } else {
                    self.hide_cursor();
                }

                if autoflush {
                    self.flush();
                    // Revert back to previous `autoflush` configuration.
                    self.automatic();
                }
            } else {
                let metas = &self.metas;
                let rstate = metas[0].is_raw_enabled;
                let mstate = metas[0].is_mouse_enabled;
                let cstate = metas[0].is_cursor_visible;
                let mode = &self.original_mode;
                let stdout = Handle::stdout().unwrap();
                stdout.set_mode(mode).unwrap();
                self.id = 0;
                screen::wincon::disable_alt().unwrap();

                if rstate {
                    self.raw();
                }

                if mstate {
                    self.enable_mouse();
                }

                if cstate {
                    self.show_cursor();
                } else {
                    self.hide_cursor();
                }
            }
        }
    }

    pub fn switch_to(&mut self, index: usize) {
        // This only switches the screen buffer and updates the settings.
        // Updating the content that will be passed in and rendered, that is
        // up to the implementation.

        // If the id and the current id are the same, well, there is nothing to
        // do, you're already on the active screen buffer.
        if index != self.index {
            if index == 0 {
                // Switch to the main screen.
                self.to_main();
            } else {
                if self.ansi_supported {
                    let metas = &self.metas;
                    let rstate = metas[index].is_raw_enabled;
                    let mstate = metas[index].is_mouse_enabled;
                    let cstate = metas[index].is_cursor_visible;
                    self.index = index;

                    // Prevent multiple `flush()` calls due to `autoflush`
                    // setting.
                    let autoflush = self.autoflush;
                    if self.autoflush { self.manual() }

                    if rstate {
                        self.raw();
                    } else {
                        self.cook();
                    }

                    if mstate {
                        self.enable_mouse();
                    } else {
                        self.disable_mouse();
                    }

                    if cstate {
                        self.show_cursor();
                    } else {
                        self.hide_cursor();
                    }

                    if autoflush {
                        self.flush();
                        // Revert back to previous `autoflush` configuration.
                        self.automatic();
                    }
                } else {
                    let metas = &self.metas;
                    let rstate = metas[index].is_raw_enabled;
                    let mstate = metas[index].is_mouse_enabled;
                    let cstate = metas[index].is_cursor_visible;
                    let mode = &self.original_mode;
                    if let Some(handle) = &self.altscreen {
                        handle.set_mode(mode).unwrap();
                        // Only show the altscreen handle if there is a switch
                        // from the main screen. Otherwise, the altscreen is
                        // already showing and no need to call `show()`.
                        if self.id == 0 {
                            handle.show().unwrap();
                        }
                        self.index = index;

                        if rstate {
                            self.raw();
                        }

                        if mstate {
                            self.enable_mouse();
                        }

                        if cstate {
                            self.show_cursor();
                        } else {
                            self.hide_cursor();
                        }
                    }
                }
            }
        }
    }

    pub fn goto(&mut self, col: i16, row: i16) {
        if self.ansi_supported {
            write_ansi(&cursor::ansi::goto(col, row));
            if self.autoflush { self.flush() }
        } else {
            cursor::wincon::goto(col, row).unwrap();
        }
    }

    pub fn up(&mut self) {
        if self.ansi_supported {
            write_ansi(&cursor::ansi::move_up(1));
            if self.autoflush { self.flush() }
        } else {
            cursor::wincon::move_up(1).unwrap();
        }
    }

    pub fn dn(&mut self) {
        if self.ansi_supported {
            write_ansi(&cursor::ansi::move_down(1));
            if self.autoflush { self.flush() }
        } else {
            cursor::wincon::move_down(1).unwrap();
        }
    }

    pub fn left(&mut self) {
        if self.ansi_supported {
            write_ansi(&cursor::ansi::move_left(1));
            if self.autoflush { self.flush() }
        } else {
            cursor::wincon::move_left(1).unwrap();
        }
    }

    pub fn right(&mut self) {
        if self.ansi_supported {
            write_ansi(&cursor::ansi::move_right(1));
            if self.autoflush { self.flush() }
        } else {
            cursor::wincon::move_right(1).unwrap();
        }
    }

    pub fn dpad(&mut self, dir: &str, n: i16) {
        // Case-insensitive.
        let d = dir.to_lowercase();
        if n > 0 {
            match d.as_str() {
                "up" => {
                    if self.ansi_supported {
                        write_ansi(&cursor::ansi::move_up(n));
                    } else {
                        cursor::wincon::move_up(n).unwrap();
                    }
                },
                "dn" => {
                    if self.ansi_supported {
                        write_ansi(&cursor::ansi::move_down(n));
                    } else {
                        cursor::wincon::move_down(n).unwrap();
                    }
                },
                "left" => {
                    if self.ansi_supported {
                        write_ansi(&cursor::ansi::move_left(n));
                    } else {
                        cursor::wincon::move_left(n).unwrap();
                    }
                },
                "right" => {
                    if self.ansi_supported {
                        write_ansi(&cursor::ansi::move_right(n));
                    } else {
                        cursor::wincon::move_right(n).unwrap();
                    }
                },
                _ => ()
            };
        }
        if self.autoflush { self.flush() }
    }

    pub fn pos(&mut self) -> (i16, i16) {
        if self.ansi_supported {
            if self.metas[self.index].is_raw_enabled {
                cursor::ansi::pos_raw().unwrap()
            } else {
                self.raw();
                let (col, row) = cursor::ansi::pos_raw().unwrap();
                self.cook();
                return (col, row);
            }
        } else {
            cursor::wincon::pos().unwrap()
        }
    }

    pub fn mark(&mut self) {
        if self.ansi_supported {
            write_ansi(&cursor::ansi::save_pos());
            if self.autoflush { self.flush() }
        } else {
            self.metas[self.index].saved_position = Some(
                cursor::wincon::pos().unwrap()
            );
        }
    }

    pub fn load(&mut self) {
        if self.ansi_supported {
            write_ansi(&cursor::ansi::load_pos());
            if self.autoflush { self.flush() }
        } else {
            match self.metas[self.index].saved_position {
                Some(pos) => {
                    self.goto(pos.0, pos.1);
                }
                None => ()
            }
        }
    }

    pub fn hide_cursor(&mut self) {
        if self.ansi_supported {
            write_ansi(&cursor::ansi::hide());
            if self.autoflush { self.flush() }
        } else {
            cursor::wincon::hide().unwrap();
        }
        let mut m = &mut self.metas[self.index];
        m.is_cursor_visible = false;
    }

    pub fn show_cursor(&mut self) {
        if self.ansi_supported {
            write_ansi(&cursor::ansi::show());
            if self.autoflush { self.flush() }
        } else {
            cursor::wincon::show().unwrap();
        }
        let mut m = &mut self.metas[self.index];
        m.is_cursor_visible = true;
    }

    pub fn set_fg(&mut self, color: &str) {
        let fg_col = output::Color::from(color);
        if self.ansi_supported {
            write_ansi(&output::ansi::set_fg(fg_col));
            if self.autoflush { self.flush() }
        } else {
            output::wincon::set_fg(fg_col, self.reset_attrs).unwrap();
        }
    }

    pub fn set_bg(&mut self, color: &str) {
        let bg_col = output::Color::from(color);
        if self.ansi_supported {
            write_ansi(&output::ansi::set_bg(bg_col));
            if self.autoflush { self.flush() }
        } else {
            output::wincon::set_bg(bg_col, self.reset_attrs).unwrap();
        }
    }

    pub fn set_tx(&mut self, style: &str) {
        // NOTE: `style` will be `reset` if the passed in
        // `&str` contains multiple values (eg. "bold, underline").
        let style = output::TextStyle::from(style);
        if self.ansi_supported {
            write_ansi(&output::ansi::set_tx(style));
            if self.autoflush { self.flush() }
        } else {
            output::wincon::set_tx(style).unwrap();
        }
    }

    pub fn set_fg_rgb(&mut self, r: u8, g:u8, b: u8) {
        let fg_col = output::Color::Rgb{
            r: r,
            g: g,
            b: b,
        };
        if self.ansi_supported {
            write_ansi(&output::ansi::set_fg(fg_col));
            if self.autoflush { self.flush() }
        } else {
            output::wincon::set_fg(fg_col, self.reset_attrs).unwrap();
        }
    }

    pub fn set_bg_rgb(&mut self, r: u8, g:u8, b: u8) {
        let bg_col = output::Color::Rgb{
            r: r,
            g: g,
            b: b,
        };
        if self.ansi_supported {
            write_ansi(&output::ansi::set_bg(bg_col));
            if self.autoflush { self.flush() }
        } else {
            output::wincon::set_bg(bg_col, self.reset_attrs).unwrap();
        }
    }

    pub fn set_fg_ansi(&mut self, v: u8) {
        let fg_col = output::Color::AnsiValue(v);
        if self.ansi_supported {
            write_ansi(&output::ansi::set_fg(fg_col));
            if self.autoflush { self.flush() }
        } else {
            output::wincon::set_fg(fg_col, self.reset_attrs).unwrap();
        }
    }

    pub fn set_bg_ansi(&mut self, v: u8) {
        let bg_col = output::Color::AnsiValue(v);
        if self.ansi_supported {
            write_ansi(&output::ansi::set_bg(bg_col));
            if self.autoflush { self.flush() }
        } else {
            output::wincon::set_bg(bg_col, self.reset_attrs).unwrap();
        }
    }

    pub fn set_style(&mut self, fg: &str, bg: &str, style: &str) {
        // The params fg is a single word, bg is also a single word, however
        // the tx param can be treated as a comma-separated list of words that
        // match the various text styles that are supported: "bold", "dim",
        // "underline", "reverse", "hide", and "reset".
        if self.ansi_supported {
            write_ansi(&output::ansi::set_all(fg, bg, style));
            if self.autoflush { self.flush() }
        } else {
            output::wincon::set_all(fg, bg, style, self.reset_attrs).unwrap();
        }
    }

    pub fn reset(&mut self) {
        if self.ansi_supported {
            write_ansi(&output::ansi::reset());
            if self.autoflush { self.flush() }
        } else {
            output::wincon::reset(self.reset_attrs).unwrap();
        }
    }

    pub fn prints(&mut self, s: &str) {
        if self.ansi_supported {
            write_ansi(&output::ansi::prints(s));
            if self.autoflush { self.flush() }
        } else {
            output::wincon::prints(s).unwrap();
        }
    }

    pub fn flush(&mut self) {
        // ANSI-only
        if self.ansi_supported {
            flush_ansi();
        }
    }

    pub fn printf(&mut self, s: &str) {
        self.prints(s);
        self.flush();
    }

    // pub fn paint() {
    //     // write with colors and styles
    // }

    // pub fn render() {
    //     // write from a template
    // }

    // pub fn intellisense() {
    //     // write from a set of rules
    //     // eg. syntax highlighting
    // }



    fn _add_metadata(&mut self) {
        let metas = &mut self.metas;
        let rstate = metas[self.index].is_raw_enabled;
        let mstate = metas[self.index].is_mouse_enabled;
        let cstate = metas[self.index].is_cursor_visible;
        metas.push(Metadata{
            is_raw_enabled: rstate,
            is_mouse_enabled: mstate,
            is_cursor_visible: cstate,
            saved_position: None,
        });
    }
}


impl Drop for Tty {
    fn drop(&mut self) {
        self.terminate()
    }
}


fn _is_ansi_supported() -> bool {
    const TERMS: [&'static str; 15] = [
        "xterm",  // xterm, PuTTY, Mintty
        "rxvt",   // RXVT
        "eterm",  // Eterm
        "screen", // GNU screen, tmux
        "tmux",   // tmux
        "vt100", "vt102", "vt220", "vt320",   // DEC VT series
        "ansi",    // ANSI
        "scoansi", // SCO ANSI
        "cygwin",  // Cygwin, MinGW
        "linux",   // Linux console
        "konsole", // Konsole
        "bvterm",  // Bitvise SSH Client
    ];

    let matched_terms = match std::env::var("TERM") {
        Ok(val) => val != "dumb" || TERMS.contains(&val.as_str()),
        Err(_) => false,
    };

    if matched_terms {
        return true
    } else {
        let enable_vt = 0x0004;
        let handle = match Handle::stdout() {
            Ok(h) => h,
            Err(_) => return false,
        };
        let mode = match handle.get_mode() {
            Ok(m) => m,
            Err(_) => return false,
        };
        match handle.set_mode(&(mode | enable_vt)) {
            Ok(_) => return true,
            Err(_) => return false,
        }
    }
}


fn _is_wincon_supported() -> bool {
    // MinTTY (and alledgedly ConPTY) do not have common support for the native
    // Console API. The MinTTY instance used by `git-bash` emulates over MSYS2,
    // which supports ANSI sequences, but throws an error when tryiing to fetch
    // the default terminal mode from `Termios` (does not exist on Windows) or
    // from the `Handle` (Console API not supported).
    //
    // MSYSTEM environment variable: (stackoverflow)
    // questions/37460073/msys-vs-mingw-internal-environment-variables
    //
    // MinTTY github issue: https://github.com/mintty/mintty/issues/56
    match std::env::var("MSYSTEM") {
        Ok(_) => false, // MSYS, MINGW64, MINGW32
        Err(_) => true, //
    }
}