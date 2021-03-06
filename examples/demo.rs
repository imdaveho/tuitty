use tuitty_core::terminal::Term;
use tuitty_core::common::enums::{ Color, Effect };
use std::{thread, time::Duration };


fn main() {
    let mut term = Term::new().expect("Error creating terminal");
    #[cfg(windows)]
    let (_, _, ansi) = term.init_data();
    #[cfg(windows)]
    println!("is_ansi: {}", ansi);
    term.printf("hello\n").unwrap();
    thread::sleep(Duration::from_millis(1500));

    term.enable_alt().unwrap();
    term.raw().unwrap();

    term.goto(15, 15).unwrap();
    term.hide_cursor().unwrap();
    term.printf("hello alternate").unwrap();
    thread::sleep(Duration::from_millis(1500));


    term.show_cursor().unwrap();

    term.goto(12, 12).unwrap();
    term.flush().unwrap();
    #[cfg(unix)]
    let (col, row) = term.raw_pos().unwrap();
    #[cfg(windows)]
    let (col, row) = term.pos().unwrap();
    term.goto(0, 0).unwrap();
    term.set_fg(Color::Red).unwrap();
    term.prints(&format!("col: {}, row: {}", col, row)).unwrap();
    term.goto(col, row).unwrap();
    term.flush().unwrap();
    term.reset_styles().unwrap();
    thread::sleep(Duration::from_millis(800));

    term.up(2).unwrap();
    term.flush().unwrap();
    #[cfg(unix)]
    let (col, row) = term.raw_pos().unwrap();
    #[cfg(windows)]
    let (col, row) = term.pos().unwrap();
    term.goto(0, 0).unwrap();
    term.set_fg(Color::Black).unwrap();
    term.set_bg(Color::DarkMagenta).unwrap();
    term.prints(&format!("col: {}, row: {}", col, row)).unwrap();
    term.goto(col, row).unwrap();
    term.flush().unwrap();
    term.reset_styles().unwrap();
    thread::sleep(Duration::from_millis(800));

    term.right(2).unwrap();
    term.flush().unwrap();
    #[cfg(unix)]
    let (col, row) = term.raw_pos().unwrap();
    #[cfg(windows)]
    let (col, row) = term.pos().unwrap();
    term.goto(0, 0).unwrap();
    term.set_fg(Color::Green).unwrap();
    term.set_fx(Effect::Underline as u32).unwrap();
    term.prints(&format!("col: {}, row: {}", col, row)).unwrap();
    term.goto(col, row).unwrap();
    term.flush().unwrap();
    term.reset_styles().unwrap();
    thread::sleep(Duration::from_millis(800));

    term.down(2).unwrap();
    term.flush().unwrap();
    #[cfg(unix)]
    let (col, row) = term.raw_pos().unwrap();
    #[cfg(windows)]
    let (col, row) = term.pos().unwrap();
    term.goto(0, 0).unwrap();
    term.set_styles(Color::DarkBlue, Color::Yellow, Effect::Bold as u32).unwrap();
    term.prints(&format!("col: {}, row: {}", col, row)).unwrap();
    term.goto(col, row).unwrap();
    term.flush().unwrap();
    term.reset_styles().unwrap();
    thread::sleep(Duration::from_millis(800));

    term.left(2).unwrap();
    term.flush().unwrap();
    #[cfg(unix)]
    let (col, row) = term.raw_pos().unwrap();
    #[cfg(windows)]
    let (col, row) = term.pos().unwrap();
    term.goto(0, 0).unwrap();
    term.prints(&format!("col: {}, row: {}", col, row)).unwrap();
    term.goto(col, row).unwrap();
    term.flush().unwrap();
    thread::sleep(Duration::from_millis(800));


    term.cook().unwrap();
    term.disable_alt().unwrap();

    term.printf("\nhello again").unwrap();
    thread::sleep(Duration::from_millis(1500));
    term.printf("\n\r").unwrap();
}
