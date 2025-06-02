#![cfg_attr(
    all(feature = "gui", not(feature = "debug")),
    windows_subsystem = "windows"
)] /* hides console window */

mod audio;
#[cfg(feature = "cli")]
mod cli;
#[cfg(feature = "gui")]
mod gui;
mod util;
#[cfg(feature = "win_service")]
mod win_service;

fn main() -> Result<(), String> {
    #[cfg(feature = "gui")]
    gui::run_main()?;

    #[cfg(feature = "cli")]
    cli::run_main()?;

    #[cfg(feature = "win_service")]
    win_service::run_main()?;

    Ok(())
}
