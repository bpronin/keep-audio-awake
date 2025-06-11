use crate::audio::AudioControl;
use crate::gui::res_ids::{IDS_APP_IS_ALREADY_RUNNING, IDS_APP_TITLE};
use crate::gui::tray_icon::start_blink_icon;
use crate::util::hwnd;
use crate::{rs, util};
use log::{debug, trace};
use native_windows_gui::{
    dispatch_thread_events, message, stop_thread_dispatch, GlobalCursor, Menu, MenuItem, MessageButtons,
    MessageIcons, MessageParams, MessageWindow, NativeUi, TrayNotification,
};
use res::RESOURCES;
use std::cell::RefCell;
use tray_icon::stop_blink_icon;
use util::check_app_running;

mod res;
mod res_ids;
mod tray_icon;

#[derive(Default)]
pub struct App {
    window: MessageWindow,
    tray: TrayNotification,
    tray_menu: Menu,
    exit_menu_item: MenuItem,
    audio: RefCell<AudioControl>,
}

impl App {
    fn on_app_exit(&self) {
        debug!("Exiting application");

        stop_blink_icon(&self.window, &self.tray);
        self.audio.borrow_mut().stop();
        stop_thread_dispatch();
    }

    fn on_timer(&self) {
        start_blink_icon(&self.window, &self.tray);
        self.audio
            .borrow_mut()
            .play()
            .expect("Failed to play audio");
    }

    fn on_show_menu(&self) {
        let (x, y) = GlobalCursor::position();
        self.tray_menu.popup(x, y);
    }

    pub fn run(&self) {
        self.audio
            .borrow_mut()
            .start(hwnd(self.window.handle))
            .expect("Failed to start audio controller");

        debug!("Application started");

        dispatch_thread_events();
    }
}

pub(crate) fn run_main() -> Result<(), String> {
    native_windows_gui::init().expect("Failed to init Native Windows GUI");

    check_app_running().map_err(|e| {
        warn_message(rs!(IDS_APP_IS_ALREADY_RUNNING));
        e
    })?;

    /* do not remove `let ui`! */
    let ui = App::build_ui(App::default()).expect("Failed to build UI");
    ui.run();

    Ok(())
}

fn warn_message(text: &str) {
    message(&MessageParams {
        title: rs!(IDS_APP_TITLE),
        content: text,
        buttons: MessageButtons::Ok,
        icons: MessageIcons::Warning,
    });
}

mod app_ui {
    use crate::audio;
    use crate::gui::res::RESOURCES;
    use crate::gui::res_ids::IDS_KEEPING_AUDIO_DEVICE_AWAKE;
    use crate::gui::res_ids::{IDI_APP_ICON, IDS_EXIT};
    use crate::gui::tray_icon::{stop_blink_icon, TIMER_ICON_BLINK};
    use crate::gui::App;
    use crate::{r_icon, rs};
    use audio::TIMER_AUDIO;
    use native_windows_gui::{
        full_bind_event_handler, unbind_event_handler, ControlHandle, Event, EventHandler, Menu, MenuItem, MessageWindow,
        NativeUi, NwgError, TrayNotification,
    };
    use std::cell::RefCell;
    use std::ops::Deref;
    use std::rc::Rc;
    use ControlHandle::Timer;

    pub struct AppUi {
        inner: Rc<App>,
        default_handler: RefCell<Vec<EventHandler>>,
    }

    impl NativeUi<AppUi> for App {
        fn build_ui(mut app: App) -> Result<AppUi, NwgError> {
            /* Controls */

            MessageWindow::builder().build(&mut app.window)?;

            TrayNotification::builder()
                .parent(&app.window)
                .icon(Some(&r_icon!(IDI_APP_ICON)))
                .tip(Some(rs!(IDS_KEEPING_AUDIO_DEVICE_AWAKE)))
                .build(&mut app.tray)?;

            Menu::builder()
                .popup(true)
                .parent(&app.window)
                .build(&mut app.tray_menu)?;

            MenuItem::builder()
                .text(rs!(IDS_EXIT))
                .parent(&app.tray_menu)
                .build(&mut app.exit_menu_item)?;

            /* Wrap-up */

            let ui = AppUi {
                inner: Rc::new(app),
                default_handler: Default::default(),
            };

            /* Events */

            let app_weak = Rc::downgrade(&ui.inner);
            let handle_events = move |evt, _data, handle| {
                if let Some(app) = app_weak.upgrade() {
                    match evt {
                        Event::OnTimerTick => {
                            if let Timer(_hwnd, timer_id) = handle {
                                if timer_id as usize == TIMER_AUDIO {
                                    app.on_timer()
                                } else if timer_id as usize == TIMER_ICON_BLINK {
                                    stop_blink_icon(&app.window, &app.tray);
                                }
                            }
                        }
                        Event::OnContextMenu => {
                            if &handle == &app.tray {
                                app.on_show_menu();
                            }
                        }
                        Event::OnMenuItemSelected => {
                            if &handle == &app.exit_menu_item {
                                app.on_app_exit();
                            }
                        }
                        _ => {}
                    }
                }
            };

            ui.default_handler
                .borrow_mut()
                .push(full_bind_event_handler(&ui.window.handle, handle_events));

            Ok(ui)
        }
    }

    impl Drop for AppUi {
        fn drop(&mut self) {
            let mut handlers = self.default_handler.borrow_mut();
            for handler in handlers.drain(0..) {
                unbind_event_handler(&handler);
            }
        }
    }

    impl Deref for AppUi {
        type Target = App;

        fn deref(&self) -> &App {
            &self.inner
        }
    }
}
