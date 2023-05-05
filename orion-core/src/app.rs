use crate::window::{Coordinates, InputEvent, Key, WindowStyle};
use chrono::{DateTime, Utc};
use glow::HasContext;
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

    fps_timestamp: DateTime<Utc>,
    fps_count: u32,
}

impl ApplicationContext {
    pub fn new() -> Self {
        Self { window: WindowContext::new("Benchmark", WindowStyle::Window { size: Coordinates::new(800, 600) }).unwrap(), fps_timestamp: Utc::now(), fps_count: 0 }
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
        self.window.set_swap_interval(1);

        loop {
            while let Some(event) = self.window.poll_event() {
                match event {
                    InputEvent::KeyPress { key, repeat: _, modifiers: _ } => {
                        if key == Key::Escape {
                            self.window.close();
                        } else if key == Key::Space {
                            let context = self.window.load_gl_pointers();
                            debug!("GL version: {:?}", context.version());
                        }
                    }
                    InputEvent::WindowClose => {
                        return;
                    }
                    _ => {}
                }

                debug!("New event: {:?}", event);
            }

            self.fps_count += 1;

            if (Utc::now() - self.fps_timestamp).num_seconds() >= 1 {
                debug!("FPS: {}", self.fps_count);
                self.fps_timestamp = Utc::now();
                self.fps_count = 0;
            }

            self.window.swap_buffers();

            #[cfg(web)]
            break;
        }
    }
}
