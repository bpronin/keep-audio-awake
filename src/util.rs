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

pub fn from_utf16(s: &[u16]) -> String {
    use std::os::windows::ffi::OsStringExt;
    use std::ffi::OsString;

    let null_index = s.iter().position(|&i| i == 0).unwrap_or(s.len());
    let os_string = OsString::from_wide(&s[0..null_index]);

    os_string
        .into_string()
        .unwrap_or("Decoding error".to_string())
}