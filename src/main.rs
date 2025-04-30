#![cfg_attr(feature = "gui", windows_subsystem = "windows")]  /* hides console window */

mod ka_service;
#[cfg(feature = "console")]
mod console;
#[cfg(feature = "gui")]
mod ui;
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
