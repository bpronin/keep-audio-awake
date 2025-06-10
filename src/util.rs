use log::warn;
use native_windows_gui::{ControlHandle};
use std::time::Duration;
use std::{ptr, thread};
use windows::core::{HRESULT, PCSTR};
use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS, HWND};
use windows::Win32::Storage::FileSystem::SYNCHRONIZE;
use windows::Win32::System::Threading::CreateMutexExA;
use windows::Win32::UI::WindowsAndMessaging::{KillTimer, SetTimer};

pub fn hwnd(handle: ControlHandle) -> Option<HWND> {
    Some(HWND(handle.hwnd().unwrap() as _))
}

pub fn check_app_running() -> Result<(), String> {
    let mutex_id = b"Global\\8e22f9ab-0f7f-4f01-8dc2-6047b74a2a99\0";

    unsafe {
        let handle = CreateMutexExA(
            Some(ptr::null_mut()),
            PCSTR(mutex_id.as_ptr()),
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
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    let null_index = s.iter().position(|&i| i == 0).unwrap_or(s.len());
    let os_string = OsString::from_wide(&s[0..null_index]);

    os_string
        .into_string()
        .unwrap_or("Decoding error".to_string())
}

pub fn sleep_cancelable<F>(duration: Duration, should_cancel: F)
where
    F: Fn() -> bool,
{
    let short = Duration::from_millis(10);
    let steps = duration.as_millis() / short.as_millis();
    let remainder = duration - (short * steps as u32);

    for _ in 0..steps {
        if should_cancel() {
            return;
        }
        thread::sleep(short);
    }

    if !should_cancel() && remainder > Duration::ZERO {
        thread::sleep(remainder);
    }
}

pub fn start_timer(hwnd: Option<HWND>, timer_id: usize, period: u32) -> Result<(), String> {
    unsafe {
        if SetTimer(hwnd, timer_id, period, None) == 0 {
            Err(format!("Failed to set timer ID:{}", timer_id))?
        }
    };
    Ok(())
}

pub fn stop_timer(hwnd: Option<HWND>, timer_id: usize) {
    unsafe {
        KillTimer(hwnd, timer_id).unwrap_or_else(|e| {
            if e.code() != HRESULT(0) {
                warn!("Failed to stop timer ID:{}. {:?}", timer_id, e);
            }
        })
    }
}
