extern crate tuitty;

use std::thread;
use std::time::Duration;

use tuitty::common::{
    traits::{
        TerminalCursor, TerminalFormatter, TerminalInput,
        TerminalModifier, TerminalSwitcher, TerminalWriter
    }, enums::{ Color, Effect },
    unicode::{ grapheme::*, wcwidth::* }
};

use tuitty::terminal;
use tuitty::interface;

use std::io::{ stdin, stdout, Result, BufRead, Write };

#[cfg(windows)]
use tuitty::terminal::wincon::Handle;

fn main() {
    let content = "👨‍👩‍👧|👨‍🚀|🤦‍♀️|褐色|क्‍ष|👧🏿|☆|\u{200d}\u{fe0f}|寬\u{2060}帶|fa\x00mily|family|";
    let groupe = UnicodeGraphemes::graphemes(content, true).collect::<Vec<&str>>();
    println!("{:?}", groupe);
    let mut t = terminal::Terminal::init();
    t.printf(content);
    let wsize = t.screen_size();
    t.printf(&format!("\n{}, {}\n", wsize.0, wsize.1));
    t.resize(86, 30);
    let wsizea = t.screen_size();
    t.printf(&format!("{}, {}", wsizea.0, wsizea.1));

    t.switch();
    t.raw();
    interface::draw_sides(&mut t);
    // t.cook();
    // t.to_main();

    // let stdout = Handle::stdout().expect("Error with Stdout");
    // let mode = stdout.get_mode().expect("Error getting mode with Stdout");
    // let mask = 0x0002 | 0x0002;
    // thread::sleep(Duration::from_millis(5000));
}