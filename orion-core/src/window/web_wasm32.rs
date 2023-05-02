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

pub struct Window {
    pub document: Document,
    pub canvas: HtmlCanvasElement,

    pub mouse_move_listener: Option<Closure<dyn FnMut(web_sys::MouseEvent)>>,

    event_queue: VecDeque<InputEvent>,
}

impl Window {
    pub fn new(_: &str) -> Result<Box<Self>> {
        console_log::init_with_level(Level::Debug)?;
        panic::set_hook(Box::new(console_error_panic_hook::hook));

        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas = canvas.dyn_into::<HtmlCanvasElement>().map_err(|_| ()).unwrap();

        let context = Box::new(Self { document, canvas, mouse_move_listener: None, event_queue: Default::default() });
        Ok(context)
    }

    pub fn init_closures<F>(&mut self, app: Rc<RefCell<ApplicationContext>>, mut event_loop: F)
    where
        F: FnMut() + 'static,
    {
        self.mouse_move_listener = Some(Closure::<dyn FnMut(_)>::new(move |event: MouseEvent| {
            let x = event.offset_x();
            let y = event.offset_y();
            app.borrow_mut().window.event_queue.push_back(InputEvent::MouseMoved(x, y));

            event_loop();
        }));

        self.canvas.add_event_listener_with_callback("mousemove", self.mouse_move_listener.as_ref().unwrap().as_ref().unchecked_ref()).unwrap();
    }

    pub fn poll_event(&mut self) -> Option<InputEvent> {
        self.event_queue.pop_front()
    }
}
