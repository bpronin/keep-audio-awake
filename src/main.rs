#![cfg_attr(all(feature = "gui", not(feature = "debug")), windows_subsystem = "windows")] /* hides console window */
#[cfg(feature = "console")]
mod console;
mod audio;
#[cfg(feature = "gui")]
mod ui;
mod util;
#[cfg(feature = "win_service")]
mod win_service;

fn main() -> Result<(), String> {
    #[cfg(feature = "gui")]
    {
        ui::run_main()?;
    }

    #[cfg(feature = "console")]
    {
        console::run_main()?;
    }

    #[cfg(feature = "win_service")]
    {
        win_service::run_main()?;
    }

    Ok(())
}
