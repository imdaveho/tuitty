// Windows Console API specific functions that enable/disable mouse mode.

use std::io::Result;
use super::handle::Handle;


const MOUSE_MODE: u32 = 0x0010 | 0x0080 | 0x0008;


pub fn enable_mouse_mode(conin: &Handle) -> Result<()> {
    let mode = conin.get_mode()?;
    let mouse_mode = (mode | MOUSE_MODE) & !0x0040;
    conin.set_mode(&mouse_mode)?;
    Ok(())
}

pub fn disable_mouse_mode(conin: &Handle) -> Result<()> {
    let mode = conin.get_mode()?;
    let mouse_mode = (mode & !MOUSE_MODE) | 0x0040;
    conin.set_mode(&mouse_mode)?;
    Ok(())
}
