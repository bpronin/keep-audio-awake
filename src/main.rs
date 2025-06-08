#![cfg_attr(not(feature = "console"), windows_subsystem = "windows")] /* hides console window */

mod audio;
mod gui;
mod util;

fn main() -> Result<(), String> {
    gui::run_main()?;
    Ok(())
}
