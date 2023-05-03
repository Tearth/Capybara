use log::debug;
use std::cell::RefCell;
use std::rc::Rc;

#[cfg(target_arch = "x86_64")]
use crate::window::winapi::WindowContext;
use crate::window::InputEvent;

#[cfg(target_arch = "wasm32")]
use crate::window::web::WindowContext;

pub struct ApplicationContext {
    pub window: Box<WindowContext>,
}

impl ApplicationContext {
    pub fn new() -> Self {
        Self { window: WindowContext::new("Benchmark").unwrap() }
    }

    pub fn run(self) {
        let app = Rc::new(RefCell::new(self));
        let app_clone = app.clone();

        #[cfg(target_arch = "wasm32")]
        app.borrow_mut().window.init_closures(app_clone.clone(), move || app_clone.borrow_mut().run_internal());

        #[cfg(target_arch = "x86_64")]
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

            #[cfg(target_arch = "wasm32")]
            break;
        }
    }
}
