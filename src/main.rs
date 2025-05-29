#![cfg_attr(
    all(feature = "gui", not(feature = "debug")),
    windows_subsystem = "windows"
)] /* hides console window */
mod audio;
#[cfg(feature = "console")]
mod console;
#[cfg(feature = "gui")]
mod ui;
mod util;
#[cfg(feature = "win_service")]
mod win_service;
mod res;
mod res_ids;

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
