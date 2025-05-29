extern crate native_windows_gui as nwg;

use nwg::NativeUi;
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use thread::JoinHandle;
use crate::audio::keep_audio_awake;
use crate::util;

#[derive(Default)]
pub struct App {
    window: nwg::MessageWindow,
    icon: nwg::Icon,
    tray: nwg::TrayNotification,
    tray_menu: nwg::Menu,
    exit_menu_item: nwg::MenuItem,
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
        self.tray.set_visibility(false); /* hide it first because stopping service may take time */

        self.service_running.store(false, Ordering::SeqCst);
        self.service_thread.take().unwrap().join().unwrap().unwrap();

        nwg::stop_thread_dispatch();
    }
    
    fn on_show_menu(&self) {
        let (x, y) = nwg::GlobalCursor::position();
        self.tray_menu.popup(x, y);
    }
}

pub(crate) fn run_main() -> Result<(), String> {
    nwg::init().expect("Failed to init Native Windows GUI");

    util::check_app_running().map_err(|e| {
        nwg::error_message("Error", "Application is already running.");
        e
    })?;
    
    let _ui = App::build_ui(App::default()).expect("Failed to build UI");
    nwg::dispatch_thread_events();

    Ok(())
}

mod app_ui {
    use crate::ui::App;
    use native_windows_gui as nwg;
    use std::cell::RefCell;
    use std::ops::Deref;
    use std::rc::Rc;

    pub struct SystemTrayUi {
        inner: Rc<App>,
        default_handler: RefCell<Vec<nwg::EventHandler>>,
    }

    impl nwg::NativeUi<SystemTrayUi> for App {
        fn build_ui(mut app: App) -> Result<SystemTrayUi, nwg::NwgError> {
            use nwg::Event as E;

            /* Resources */

            let resource = nwg::EmbedResource::load(None).unwrap();
            
            nwg::Icon::builder()
                .source_embed(Some(&resource))
                .source_embed_str(Some("IDI_APP_ICON"))
                .build(&mut app.icon)?;

            /* Controls */

            nwg::MessageWindow::builder().build(&mut app.window)?;

            nwg::TrayNotification::builder()
                .parent(&app.window)
                .icon(Some(&app.icon))
                .tip(Some("Keeping audio device awake"))
                .build(&mut app.tray)?;

            nwg::Menu::builder()
                .popup(true)
                .parent(&app.window)
                .build(&mut app.tray_menu)?;

            nwg::MenuItem::builder()
                .text("Exit")
                .parent(&app.tray_menu)
                .build(&mut app.exit_menu_item)?;

            /* Wrap-up */

            let ui = SystemTrayUi {
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
                .push(nwg::full_bind_event_handler(
                    &ui.window.handle,
                    handle_events,
                ));

            Ok(ui)
        }
    }

    impl Drop for SystemTrayUi {
        fn drop(&mut self) {
            let mut handlers = self.default_handler.borrow_mut();
            for handler in handlers.drain(0..) {
                nwg::unbind_event_handler(&handler);
            }
        }
    }

    impl Deref for SystemTrayUi {
        type Target = App;

        fn deref(&self) -> &App {
            &self.inner
        }
    }
}
