extern crate tuitty;
extern crate winapi;

use winapi::um::wincon::{
    WriteConsoleOutputW,
    WriteConsoleOutputA,
    ReadConsoleOutputW,
    ReadConsoleOutputA,
    CHAR_INFO, COORD, SMALL_RECT
};
use std::mem::zeroed;
use std::thread;
use std::time::Duration;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;


use winapi::um::consoleapi::{
    ReadConsoleW, WriteConsoleW
};

use winapi::ctypes::c_void;

fn main() {
    let handle = tuitty::terminal::actions::win32::Handle::stdout().unwrap();
    let mode = handle.get_mode().unwrap();
    let info = tuitty::terminal::actions::win32::ConsoleInfo::of(&handle).unwrap();
    let attrs = info.attributes();

    // let blue = 0x0001;
    // let green = 0x0002;
    // let red = 0x0004;
    

    let altern = tuitty::terminal::actions::win32::Handle::buffer().unwrap();
    tuitty::terminal::actions::win32::enable_alt(&altern, &mode, false);
    // tuitty::terminal::actions::win32::raw();


    // let output = "qwertyuiopasdfghjklzxcvbnmQWERTYUIOPASDFGHJKLZXCVBNM";
    // let differ = "qwertyuiopasdfghjkl園xcvbnmQWERTYUIOPASDFGHJKLZXCVBNM";
    let output = "qwertyuiopasd⚠️fghjkl😀園v明nmQWE👪RTY👨‍👩‍👧UIOPASDFGHJKLZXCVBNM";
    // let d16 = differ.encode_utf16();
    // let o16 = output.encode_utf16();


    // visualize difference:
    // let mut index = 0;
    // for (o, d) in o16.zip(d16) {
    //     if o != d {
    //         println!("not equal @ i: {}", index);
    //         println!("o: {}", o);
    //         println!("d: {}", d);
    //         println!("-----")
    //     }
    //     index += 1;
    // }


    // Write to buffer
    tuitty::terminal::actions::win32::goto(0, 0, false);
    tuitty::terminal::actions::win32::printf(output, false);

    // tuitty::terminal::actions::win32::goto(0, 20, false);
    // tuitty::terminal::actions::win32::printf(output, false);
    thread::sleep(Duration::from_millis(2000));

    // tuitty::terminal::actions::win32::goto(13, 0, false);
    // let mut ret = String::new();
    // let data: *mut std::ffi::c_void = ret.as_mut_ptr();
    // let read_ch = unsafe {
    //     ReadConsoleW(altern.0, lpBuffer: LPVOID, nNumberOfCharsToRead: DWORD, lpNumberOfCharsRead: LPDWORD, pInputControl: PCONSOLE_READCONSOLE_CONTROL)
    // }

    // // Setup ranges

    // // this is essentially taken from read_buf -- 
    // let _char_info_buf = o16
    //     .map(|ch| unsafe {
    //         let mut char_info: CHAR_INFO = zeroed();
    //         char_info.Attributes = attrs;
    //         *char_info.Char.UnicodeChar_mut() = ch;
    //         char_info
    //     }).collect::<Vec<CHAR_INFO>>();
    
    // let len = char_info_buf.len();
    // let len = 85 * 29;
    let size = info.terminal_size();
    println!("X: {}, Y: {}", size.0, size.1);
    let buf_sizze = COORD {X: 85 , Y: 29};
    let buf_coord = COORD {X: 0, Y: 0};
    let mut dest_rect = SMALL_RECT {
        Top: 0,
        Left: 0,
        Bottom: 29,
        Right: 85,
    };

    // Read contents
    // let mut read_buf: Vec<CHAR_INFO> = Vec::with_capacity(len);
    let mut read_buf: Vec<CHAR_INFO> = unsafe { vec![zeroed(); 85 * 29 * 4] };
    // let mut read_buf = unsafe { vec![zeroed(); 85 * 29 * 4] };
    let buf_read = unsafe {
        ReadConsoleOutputW(
        // ReadConsoleOutputA(
            altern.0, 
            read_buf.as_mut_ptr(), 
            buf_sizze, 
            buf_coord,
            &mut dest_rect)
        };
    if buf_read == 0 { 
        tuitty::terminal::actions::win32::cook();
        tuitty::terminal::actions::win32::disable_alt(false);
        panic!("Something went wrong reading buffer");
    }

    let read_u16: Vec<u16> = read_buf.iter().map(|x| unsafe { *x.Char.UnicodeChar() }).collect();
    let read_string: String = String::from_utf16_lossy(&read_u16);
    tuitty::terminal::actions::win32::goto(0, 0, false);
    let mut written = 0;
    let res = unsafe {
        WriteConsoleW(
            altern.0,
            read_string.as_ptr() as *const c_void,
            85 * 29,
            &mut written, null_mut()
        )
    };
    if res == 0 {
            tuitty::terminal::actions::win32::cook();
            tuitty::terminal::actions::win32::disable_alt(false);
            panic!("Something went wrong pasting buffer");
    }
    


    // let mut index = 0;
    // for u in OsStr::new(&read_string).encode_wide() {
    //     unsafe {
    //         *read_buf[index].Char.UnicodeChar_mut() = u;    
    //     }
    //     index += 1;
    // };

    // // Copy contents over
    // let buf_copy = unsafe {
    //     WriteConsoleOutputW(
    //     // WriteConsoleOutputA(
    //         altern.0, 
    //         read_buf.as_ptr(), 
    //         // COORD {X: len as i16, Y: 1},
    //         COORD {X: 51, Y: 1},
    //         COORD {X: 0, Y: 0},
    //         &mut SMALL_RECT {
    //             Top: 5,
    //             Left: 5,
    //             Bottom: 29,
    //             Right: 85,
    //         })
    // };

    // if buf_copy == 0 { 
    //     tuitty::terminal::actions::win32::cook();
    //     tuitty::terminal::actions::win32::disable_alt(false);
    //     panic!("Something went wrong copying buffer");
    // }

    thread::sleep(Duration::from_millis(2000));


    // // Check read_buf contents
    // // when read_buf was Vec::with_capacity();
    // tuitty::terminal::actions::win32::goto(0, 8, false);
    // // Check dest_rect
    // tuitty::terminal::actions::win32::printf(
    //     &format!("top: {}, left: {}, bot: {}, right: {}",
    //     dest_rect.Top, dest_rect.Left, dest_rect.Bottom, dest_rect.Right
    // ), false);
    // // Check Char
    // // let mut chinfo = *read_buf.get(0).unwrap();
    // let buf = unsafe {
    //     // when read_buf was Vec::with_capacity();
    //     std::slice::from_raw_parts(read_buf.as_ptr(), dest_rect.Right as usize)
    // };
    // tuitty::terminal::actions::win32::printf(
    //     &format!("{:?}", unsafe {
    //         buf[19].Char.UnicodeChar()
    //     }), false
    // );
    // // tuitty::terminal::actions::win32::printf(
    // //     &format!("{:?}", unsafe {
    // //         chinfo.Char.UnicodeChar_mut()
    // //     }), false);

    // // Try modifying read buffer?
    // // let patch = "--".encode_utf16();
    // // let patch = "--".bytes();
    // #[cfg(not(windows))]
    // let patch = "😀".encode_utf16();
    // #[cfg(windows)]
    // let patch: Vec<u16> = OsStr::new("👪").encode_wide().collect();
    // // patch 1
    // // let mut index = 20; // contingent on knowing the correct index...
    // let mut index = 9 * 85 + 12;
    // for i in patch {
    //     unsafe {
    //         *read_buf[index].Char.UnicodeChar_mut() = i;
    //         // *read_buf[index].Char.AsciiChar_mut() = i as i8;
    //     }
    //     index += 1;
    // }
    // // patch 2 (coord: row 8, x: 15)
    // // let patch = "--".encode_utf16();
    // let patch: Vec<u16> = OsStr::new("--").encode_wide().collect();
    // // let patch = "--".bytes();
    // let mut index = 9 * 85 + 15;
    // for i in patch {
    //     // unsafe {
    //     //     let mut ch_info: CHAR_INFO = zeroed();
    //     //     ch_info.Attributes = attrs;
    //     //     *ch_info.Char.UnicodeChar_mut() = i;
    //     //     // *ch_info.Char.AsciiChar_mut() = i as i8;
    //     //     read_buf[index] = ch_info;
    //     // }
    //     unsafe {
    //         *read_buf[index].Char.UnicodeChar_mut() = i;
    //         // *read_buf[index].Char.AsciiChar_mut() = i as i8;
    //     }
    //     index += 1;
    // }


    // let read_u16: Vec<u16> = read_buf.iter().map(|x| unsafe { *x.Char.UnicodeChar() }).collect();
    // let read_string: String = String::from_utf16_lossy(&read_u16);
    // let mut index = 0;
    // for u in OsStr::new(&read_string).encode_wide() {
    //     unsafe {
    //         *read_buf[index].Char.UnicodeChar_mut() = u;    
    //     }
    //     index += 1;
    // };



    let buf_modded = unsafe {
        WriteConsoleOutputW(
        // WriteConsoleOutputA(
            altern.0, 
            read_buf.as_ptr(), 
            COORD {X: 85, Y: 29},
            COORD {X: 0, Y: 0},
            &mut SMALL_RECT {
                Top: 0,
                Left: 0,
                Bottom: 29,
                Right: 85,
            })
    };

    if buf_modded == 0 { 
        tuitty::terminal::actions::win32::cook();
        tuitty::terminal::actions::win32::disable_alt(false);
        panic!("Something went wrong copying buffer");
    }


    // Attempt to fill the screen using iteration
    // let w = "😀".encode_utf16();
    // let w_info_buf = w.map(|ch| unsafe {
    //     let mut char_info: CHAR_INFO = zeroed();
    //     char_info.Attributes = attrs;
    //     *char_info.Char.UnicodeChar_mut() = ch;
    //     char_info
    // }).collect::<Vec<CHAR_INFO>>();
    // for i in 0..30 {
    //     for j in 0..86 {
    //     let buf_iter = unsafe {
    //         WriteConsoleOutputW(
    //             altern.0, 
    //             w_info_buf.as_ptr(), 
    //             COORD {X: 1, Y: 1},
    //             COORD {X: 0, Y: 0},
    //             &mut SMALL_RECT {
    //                 Top: i,
    //                 Left: j,
    //                 Bottom: 29,
    //                 Right: 85,
    //             }
    //         )
    //     };
    //     if buf_iter == 0 { 
    //         tuitty::terminal::actions::win32::cook();
    //         tuitty::terminal::actions::win32::disable_alt(false);
    //         panic!("Something went wrong iterating buffer");
    //     }
    // }}



    // Example full buffer fill
    // let w = "😀".repeat(85*29);
    // let w = w.encode_utf16();
    // let w_info_buf = w.map(|ch| unsafe {
    //     let mut char_info: CHAR_INFO = zeroed();
    //     char_info.Attributes = attrs;
    //     *char_info.Char.UnicodeChar_mut() = ch;
    //     char_info
    // }).collect::<Vec<CHAR_INFO>>();

    // let buf_iter = unsafe {
    //     WriteConsoleOutputW(
    //         altern.0, 
    //         w_info_buf.as_ptr(), 
    //         COORD {X: 86, Y: 30},
    //         COORD {X: 0, Y: 0},
    //         &mut SMALL_RECT {
    //             Top: 0,
    //             Left: 0,
    //             Bottom: 29,
    //             Right: 85,
    //         }
    //     )
    // };

    // if buf_iter == 0 { 
    //     tuitty::terminal::actions::win32::cook();
    //     tuitty::terminal::actions::win32::disable_alt(false);
    //     panic!("Something went wrong iterating buffer");
    // }

    thread::sleep(Duration::from_millis(5000));

    tuitty::terminal::actions::win32::cook();
    tuitty::terminal::actions::win32::disable_alt(false);
}
