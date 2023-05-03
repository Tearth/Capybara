use super::*;
use crate::app::ApplicationContext;
use anyhow::Result;
use log::Level;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::panic;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::Document;
use web_sys::HtmlCanvasElement;
use web_sys::MouseEvent;
use web_sys::Window;

pub struct WindowContext {
    pub window: Window,
    pub document: Document,
    pub canvas: HtmlCanvasElement,

    resize_callback: Option<Closure<dyn FnMut()>>,
    mousemove_callback: Option<Closure<dyn FnMut(web_sys::MouseEvent)>>,
    mouseenter_callback: Option<Closure<dyn FnMut(web_sys::MouseEvent)>>,
    mouseleave_callback: Option<Closure<dyn FnMut(web_sys::MouseEvent)>>,

    last_canvas_size: Coordinates,
    event_queue: VecDeque<InputEvent>,
}

impl WindowContext {
    pub fn new(_: &str) -> Result<Box<Self>> {
        console_log::init_with_level(Level::Debug)?;
        panic::set_hook(Box::new(console_error_panic_hook::hook));

        let inner = web_sys::window().unwrap();
        let document = inner.document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas = canvas.dyn_into::<HtmlCanvasElement>().map_err(|_| ()).unwrap();
        let last_canvas_size = Coordinates::new(canvas.scroll_width(), canvas.scroll_height());

        let context = Box::new(Self {
            window: inner,
            document,
            canvas,
            resize_callback: None,
            mousemove_callback: None,
            mouseenter_callback: None,
            mouseleave_callback: None,
            last_canvas_size,
            event_queue: Default::default(),
        });
        Ok(context)
    }

    #[allow(clippy::redundant_clone)]
    pub fn init_closures<F>(&mut self, app: Rc<RefCell<ApplicationContext>>, event_loop: F)
    where
        F: FnMut() + Clone + 'static,
    {
        self.init_resize_callback(app.clone(), event_loop.clone());
        self.init_mousemove_callback(app.clone(), event_loop.clone());
        self.init_mouseenter_callback(app.clone(), event_loop.clone());
        self.init_mouseleave_callback(app.clone(), event_loop.clone());
    }

    fn init_resize_callback<F>(&mut self, app: Rc<RefCell<ApplicationContext>>, mut event_loop: F)
    where
        F: FnMut() + Clone + 'static,
    {
        self.resize_callback = Some(Closure::<dyn FnMut()>::new(move || {
            let mut app = app.borrow_mut();
            let canvas = &app.window.canvas;
            let canvas_size = Coordinates::new(canvas.scroll_width(), canvas.scroll_height());

            if canvas_size != app.window.last_canvas_size {
                app.window.event_queue.push_back(InputEvent::WindowSizeChange(canvas_size));
            }
            drop(app);

            event_loop();
        }));
        self.window.add_event_listener_with_callback("resize", self.resize_callback.as_ref().unwrap().as_ref().unchecked_ref()).unwrap();
    }

    fn init_mousemove_callback<F>(&mut self, app: Rc<RefCell<ApplicationContext>>, mut event_loop: F)
    where
        F: FnMut() + Clone + 'static,
    {
        self.mousemove_callback = Some(Closure::<dyn FnMut(_)>::new(move |event: MouseEvent| {
            app.borrow_mut().window.event_queue.push_back(InputEvent::MouseMove(Coordinates::new(event.offset_x(), event.offset_y())));
            event_loop();
        }));
        self.canvas.add_event_listener_with_callback("mousemove", self.mousemove_callback.as_ref().unwrap().as_ref().unchecked_ref()).unwrap();
    }

    fn init_mouseenter_callback<F>(&mut self, app: Rc<RefCell<ApplicationContext>>, mut event_loop: F)
    where
        F: FnMut() + Clone + 'static,
    {
        self.mouseenter_callback = Some(Closure::<dyn FnMut(_)>::new(move |event: MouseEvent| {
            app.borrow_mut().window.event_queue.push_back(InputEvent::MouseEnter(Coordinates::new(event.offset_x(), event.offset_y())));
            event_loop();
        }));
        self.canvas.add_event_listener_with_callback("mouseenter", self.mouseenter_callback.as_ref().unwrap().as_ref().unchecked_ref()).unwrap();
    }

    fn init_mouseleave_callback<F>(&mut self, app: Rc<RefCell<ApplicationContext>>, mut event_loop: F)
    where
        F: FnMut() + Clone + 'static,
    {
        self.mouseleave_callback = Some(Closure::<dyn FnMut(_)>::new(move |_: MouseEvent| {
            app.borrow_mut().window.event_queue.push_back(InputEvent::MouseLeave);
            event_loop();
        }));
        self.canvas.add_event_listener_with_callback("mouseleave", self.mouseleave_callback.as_ref().unwrap().as_ref().unchecked_ref()).unwrap();
    }

    pub fn poll_event(&mut self) -> Option<InputEvent> {
        self.event_queue.pop_front()
    }
}
