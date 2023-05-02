use crate::window::InputEvent;
use std::cell::RefCell;
use std::rc::Rc;

#[cfg(target_arch = "x86_64")]
use crate::window::windows_winapi::Window;

#[cfg(target_arch = "wasm32")]
use crate::window::web_wasm32::Window;

pub struct ApplicationContext {
    pub window: Box<Window>,
}

impl ApplicationContext {
    pub fn new() -> Self {
        Self { window: Window::new().unwrap() }
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
                if let InputEvent::MouseMoved(x, y) = event {
                    unsafe { web_sys::console::log_1(&format!("{} {}", x, y).into()) };
                    //println!("{} {}", x, y);
                }
            }

            #[cfg(target_arch = "wasm32")]
            break;
        }
    }
}
