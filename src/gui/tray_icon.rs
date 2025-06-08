use crate::gui::res_ids::{IDI_APP_ICON, IDI_APP_ICON_GRAY};
use crate::gui::RESOURCES;
use crate::r_icon;
use crate::util::hwnd;
use native_windows_gui::{MessageWindow, TrayNotification};
use windows::Win32::UI::WindowsAndMessaging::{KillTimer, SetTimer};

pub const TIMER_ICON_BLINK: usize = 411;
const TIMER_ICON_BLINK_PERIOD_MS: u32 = 500;

pub fn start_blink_icon(window: &MessageWindow, tray: &TrayNotification) {
    set_busy_icon(tray, true);

    if unsafe {
        SetTimer(
            hwnd(window.handle),
            TIMER_ICON_BLINK,
            TIMER_ICON_BLINK_PERIOD_MS,
            None,
        )
    } == 0
    {
        set_busy_icon(tray, false);
        panic!("Failed to start icon blink timer");
    }
}

pub fn stop_blink_icon(window: &MessageWindow, tray: &TrayNotification) {
    set_busy_icon(tray, false);
    unsafe {
        KillTimer(hwnd(window.handle), TIMER_ICON_BLINK).unwrap_or_else(|e| {
            eprintln!("Failed to stop icon blink timer: {}", e);
        })
    }
}

fn set_busy_icon(tray: &TrayNotification, busy: bool) {
    let icon_res = if busy {
        IDI_APP_ICON_GRAY
    } else {
        IDI_APP_ICON
    };
    tray.set_icon(&r_icon!(icon_res));
}
