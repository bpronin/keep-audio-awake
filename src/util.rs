use std::ptr;
use windows::core::PCSTR;
use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
use windows::Win32::Storage::FileSystem::SYNCHRONIZE;
use windows::Win32::System::Threading::CreateMutexExA;

pub fn check_app_running() -> Result<(), String> {
    let name = b"Global\\8e22f9ab-0f7f-4f01-8dc2-6047b74a2a99\0";

    unsafe {
        let handle = CreateMutexExA(
            Some(ptr::null_mut()),
            PCSTR(name.as_ptr()),
            0,
            SYNCHRONIZE.0,
        )
        .map_err(|e| e.message().to_owned())?;

        if handle.is_invalid() || GetLastError() == ERROR_ALREADY_EXISTS {
            Err("Already running.".to_string())
        } else {
            Ok(())
        }
    }
}
