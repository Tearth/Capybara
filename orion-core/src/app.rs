use crate::window::{Coordinates, InputEvent, WindowStyle};
use log::debug;
use std::cell::RefCell;
use std::rc::Rc;

#[cfg(windows)]
use crate::window::winapi::WindowContext;

#[cfg(unix)]
use crate::window::x11::WindowContext;

#[cfg(web)]
use crate::window::web::WindowContext;

pub struct ApplicationContext {
    pub window: Box<WindowContext>,
}

impl ApplicationContext {
    pub fn new() -> Self {
        Self { window: WindowContext::new("Benchmark", WindowStyle::Window { size: Coordinates::new(800, 600) }).unwrap() }
    }

    pub fn run(self) {
        let app = Rc::new(RefCell::new(self));
        let app_clone = app.clone();

        #[cfg(web)]
        app.borrow_mut().window.init_closures(app.clone(), move || app_clone.borrow_mut().run_internal());

        #[cfg(any(windows, unix))]
        app.borrow_mut().run_internal();
    }

    pub fn run_internal(&mut self) {
        loop {
            while let Some(event) = self.window.poll_event() {
                if event == InputEvent::WindowClose {
                    return;
                }

                debug!("New event: {:?}", event);
            }

            #[cfg(web)]
            break;
        }
    }
}
