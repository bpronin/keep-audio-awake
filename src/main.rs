#![cfg_attr(feature = "gui", windows_subsystem = "windows")] /* hides console window */
use windows::core::PCSTR;
use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
use windows::Win32::Storage::FileSystem::SYNCHRONIZE;
use windows::Win32::System::Threading::CreateMutexExA;

#[cfg(feature = "console")]
mod console;
mod ka_service;
#[cfg(feature = "gui")]
mod ui;
#[cfg(feature = "win_service")]
mod win_service;

fn already_running() -> bool {
    let name = b"Global\\8e22f9ab-0f7f-4f01-8dc2-6047b74a2a99\0";

    unsafe {
        let handle = CreateMutexExA(
            Some(std::ptr::null_mut()),
            PCSTR(name.as_ptr()),
            0,
            SYNCHRONIZE.0,
        );

        handle.unwrap().is_invalid() || GetLastError() == ERROR_ALREADY_EXISTS
    }
}

fn main() -> Result<(), String> {
    if already_running() {
        return Err("Already running.".to_string());
    }

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
