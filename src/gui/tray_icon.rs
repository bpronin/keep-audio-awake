use crate::gui::res_ids::{IDI_APP_ICON, IDI_APP_ICON_GRAY};
use crate::gui::RESOURCES;
use crate::r_icon;
use crate::util::{hwnd, start_timer, stop_timer};
use log::{trace};
use native_windows_gui::{MessageWindow, TrayNotification};

pub const TIMER_ICON_BLINK: usize = 411;
const TIMER_ICON_BLINK_PERIOD_MS: u32 = 500;

pub fn start_blink_icon(window: &MessageWindow, tray: &TrayNotification) {
    set_busy_icon(tray, true);

    trace!("Starting icon blink");
    
    if start_timer(
        hwnd(window.handle),
        TIMER_ICON_BLINK,
        TIMER_ICON_BLINK_PERIOD_MS,
    )
    .is_err()
    {
        set_busy_icon(tray, false);
    }
}

pub fn stop_blink_icon(window: &MessageWindow, tray: &TrayNotification) {
    trace!("Stopping icon blink ");
    
    set_busy_icon(tray, false);
    stop_timer(hwnd(window.handle), TIMER_ICON_BLINK);
}

fn set_busy_icon(tray: &TrayNotification, busy: bool) {
    let icon_res = if busy {
        IDI_APP_ICON_GRAY
    } else {
        IDI_APP_ICON
    };
    tray.set_icon(&r_icon!(icon_res));
}
