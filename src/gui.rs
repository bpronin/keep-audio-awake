use crate::audio::keep_audio_awake;
use crate::gui::res_ids::{IDS_APP_IS_ALREADY_RUNNING, IDS_APP_TITLE};
use crate::{rs, util};
use native_windows_gui::{
    dispatch_thread_events, message, stop_thread_dispatch, GlobalCursor, Menu, MenuItem, MessageButtons,
    MessageIcons, MessageParams, MessageWindow, NativeUi, TrayNotification,
};
use res::RESOURCES;
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use thread::JoinHandle;
use util::check_app_running;

mod res;
mod res_ids;

#[derive(Default)]
pub struct App {
    window: MessageWindow,
    tray: TrayNotification,
    tray_menu: Menu,
    exit_menu_item: MenuItem,
    service_thread: RefCell<Option<JoinHandle<Result<(), String>>>>,
    service_running: Arc<AtomicBool>,
}

impl App {
    fn on_app_init(&self) {
        self.service_running.store(true, Ordering::SeqCst);
        let running = Arc::clone(&self.service_running);
        *self.service_thread.borrow_mut() = Some(thread::spawn(move || {
            keep_audio_awake(running)?;

            Ok(())
        }));
    }

    fn on_app_exit(&self) {
        self.service_running.store(false, Ordering::SeqCst);
        self.service_thread.take().unwrap().join().unwrap().unwrap();

        stop_thread_dispatch();
    }

    fn on_show_menu(&self) {
        let (x, y) = GlobalCursor::position();
        self.tray_menu.popup(x, y);
    }
}

pub(crate) fn run_main() -> Result<(), String> {
    native_windows_gui::init().expect("Failed to init Native Windows GUI");

    check_app_running().map_err(|e| {
        warn_message(rs!(IDS_APP_IS_ALREADY_RUNNING));
        e
    })?;

    /* do not remove `let _ui`! */
    let _ui = App::build_ui(App::default()).expect("Failed to build UI");

    dispatch_thread_events();

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
    use crate::gui::res::RESOURCES;
    use crate::gui::res_ids::IDS_KEEPING_AUDIO_DEVICE_AWAKE;
    use crate::gui::res_ids::{IDI_APP_ICON, IDS_EXIT};
    use crate::gui::App;
    use crate::{r_icon, rs};
    use native_windows_gui::{
        full_bind_event_handler, unbind_event_handler, EventHandler, Menu, MenuItem, MessageWindow, NativeUi,
        NwgError, TrayNotification,
    };
    use std::cell::RefCell;
    use std::ops::Deref;
    use std::rc::Rc;

    pub struct AppUi {
        inner: Rc<App>,
        default_handler: RefCell<Vec<EventHandler>>,
    }

    impl NativeUi<AppUi> for App {
        fn build_ui(mut app: App) -> Result<AppUi, NwgError> {
            use native_windows_gui::Event as E;

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

            let evt_ui = Rc::downgrade(&ui.inner);
            let handle_events = move |evt, _evt_data, handle| {
                if let Some(evt_ui) = evt_ui.upgrade() {
                    match evt {
                        E::OnInit => {
                            if &handle == &evt_ui.window {
                                evt_ui.on_app_init();
                            }
                        }
                        E::OnContextMenu => {
                            if &handle == &evt_ui.tray {
                                evt_ui.on_show_menu();
                            }
                        }
                        E::OnMenuItemSelected => {
                            if &handle == &evt_ui.exit_menu_item {
                                evt_ui.on_app_exit();
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
