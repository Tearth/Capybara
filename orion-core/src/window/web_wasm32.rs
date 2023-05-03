use super::*;
use crate::app::ApplicationContext;
use anyhow::Result;
use log::debug;
use log::Level;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::panic;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::Document;
use web_sys::HtmlCanvasElement;
use web_sys::MouseEvent;

pub struct Window {
    pub inner: web_sys::Window,
    pub document: Document,
    pub canvas: HtmlCanvasElement,

    pub resize_listener: Option<Closure<dyn FnMut()>>,
    pub mouse_move_listener: Option<Closure<dyn FnMut(web_sys::MouseEvent)>>,
    pub mouse_enter_listener: Option<Closure<dyn FnMut(web_sys::MouseEvent)>>,
    pub mouse_leave_listener: Option<Closure<dyn FnMut(web_sys::MouseEvent)>>,

    last_canvas_size: Coordinates,
    event_queue: VecDeque<InputEvent>,
}

impl Window {
    pub fn new(_: &str) -> Result<Box<Self>> {
        console_log::init_with_level(Level::Debug)?;
        panic::set_hook(Box::new(console_error_panic_hook::hook));

        let inner = web_sys::window().unwrap();
        let document = inner.document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas = canvas.dyn_into::<HtmlCanvasElement>().map_err(|_| ()).unwrap();
        let last_canvas_size = Coordinates::new(canvas.scroll_width(), canvas.scroll_height());

        let context = Box::new(Self {
            inner,
            document,
            canvas,
            resize_listener: None,
            mouse_move_listener: None,
            mouse_enter_listener: None,
            mouse_leave_listener: None,
            last_canvas_size,
            event_queue: Default::default(),
        });
        Ok(context)
    }

    pub fn init_closures<F>(&mut self, app: Rc<RefCell<ApplicationContext>>, event_loop: F)
    where
        F: FnMut() + Clone + 'static,
    {
        let mut event_loop_copy = event_loop.clone();
        let app_clone = app.clone();

        self.resize_listener = Some(Closure::<dyn FnMut()>::new(move || {
            let mut app = app_clone.borrow_mut();
            let canvas = &app.window.canvas;
            let canvas_size = Coordinates::new(canvas.scroll_width(), canvas.scroll_height());

            if canvas_size != app.window.last_canvas_size {
                app.window.event_queue.push_back(InputEvent::WindowSizeChange(canvas_size));
            }
            drop(app);

            event_loop_copy();
        }));
        self.inner.add_event_listener_with_callback("resize", self.resize_listener.as_ref().unwrap().as_ref().unchecked_ref()).unwrap();

        let mut event_loop_copy = event_loop.clone();
        let app_clone = app.clone();

        self.mouse_move_listener = Some(Closure::<dyn FnMut(_)>::new(move |event: MouseEvent| {
            let x = event.offset_x();
            let y = event.offset_y();
            app_clone.borrow_mut().window.event_queue.push_back(InputEvent::MouseMove(Coordinates::new(x, y)));

            event_loop_copy();
        }));
        self.canvas.add_event_listener_with_callback("mousemove", self.mouse_move_listener.as_ref().unwrap().as_ref().unchecked_ref()).unwrap();

        let mut event_loop_copy = event_loop.clone();
        let app_clone = app.clone();

        self.mouse_enter_listener = Some(Closure::<dyn FnMut(_)>::new(move |event: MouseEvent| {
            let x = event.offset_x();
            let y = event.offset_y();
            app_clone.borrow_mut().window.event_queue.push_back(InputEvent::MouseEnter(Coordinates::new(x, y)));

            event_loop_copy();
        }));
        self.canvas.add_event_listener_with_callback("mouseenter", self.mouse_enter_listener.as_ref().unwrap().as_ref().unchecked_ref()).unwrap();

        let mut event_loop_copy = event_loop.clone();
        let app_clone = app.clone();

        self.mouse_leave_listener = Some(Closure::<dyn FnMut(_)>::new(move |event: MouseEvent| {
            app_clone.borrow_mut().window.event_queue.push_back(InputEvent::MouseLeave);

            event_loop_copy();
        }));
        self.canvas.add_event_listener_with_callback("mouseleave", self.mouse_leave_listener.as_ref().unwrap().as_ref().unchecked_ref()).unwrap();
    }

    pub fn poll_event(&mut self) -> Option<InputEvent> {
        self.event_queue.pop_front()
    }
}
