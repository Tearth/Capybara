use super::*;
use crate::app::ApplicationContext;
use anyhow::Error;
use anyhow::Result;
use glow::Context;
use log::Level;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::panic;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::Document;
use web_sys::HtmlCanvasElement;
use web_sys::KeyboardEvent;
use web_sys::MouseEvent;
use web_sys::WebGl2RenderingContext;
use web_sys::WheelEvent;
use web_sys::Window;

pub struct WindowContextWeb {
    pub window: Window,
    pub document: Document,
    pub canvas: HtmlCanvasElement,
    pub webgl_context: Option<WebGl2RenderingContext>,

    pub size: Coordinates,
    pub cursor_visible: bool,
    pub cursor_position: Coordinates,
    pub cursor_in_window: bool,
    pub mouse_state: Vec<bool>,
    pub keyboard_state: Vec<bool>,

    frame_callback: Option<Closure<dyn FnMut()>>,
    resize_callback: Option<Closure<dyn FnMut()>>,
    mousemove_callback: Option<Closure<dyn FnMut(MouseEvent)>>,
    mouseenter_callback: Option<Closure<dyn FnMut(MouseEvent)>>,
    mouseleave_callback: Option<Closure<dyn FnMut(MouseEvent)>>,
    mousedown_callback: Option<Closure<dyn FnMut(MouseEvent)>>,
    mouseup_callback: Option<Closure<dyn FnMut(MouseEvent)>>,
    wheel_callback: Option<Closure<dyn FnMut(WheelEvent)>>,
    keydown_callback: Option<Closure<dyn FnMut(KeyboardEvent)>>,
    keyup_callback: Option<Closure<dyn FnMut(KeyboardEvent)>>,
    keypress_callback: Option<Closure<dyn FnMut(KeyboardEvent)>>,

    last_character: Option<char>,
    event_queue: VecDeque<InputEvent>,
}

impl WindowContextWeb {
    pub fn new(_: &str, _: WindowStyle) -> Result<Box<Self>> {
        console_log::init_with_level(Level::Debug)?;
        panic::set_hook(Box::new(console_error_panic_hook::hook));

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas = canvas.dyn_into::<HtmlCanvasElement>().map_err(|_| ()).unwrap();
        let last_canvas_size = Coordinates::new(canvas.scroll_width(), canvas.scroll_height());

        let mut context = Box::new(Self {
            window,
            document,
            canvas,
            webgl_context: None,

            size: last_canvas_size,
            cursor_visible: true,
            cursor_position: Default::default(),
            cursor_in_window: false,
            mouse_state: vec![false; MouseButton::Unknown as usize],
            keyboard_state: vec![false; Key::Unknown as usize],

            frame_callback: None,
            resize_callback: None,
            mousemove_callback: None,
            mouseenter_callback: None,
            mouseleave_callback: None,
            mousedown_callback: None,
            mouseup_callback: None,
            wheel_callback: None,
            keydown_callback: None,
            keyup_callback: None,
            keypress_callback: None,

            last_character: None,
            event_queue: Default::default(),
        });
        context.init_gl_context()?;

        Ok(context)
    }

    fn init_gl_context(&mut self) -> Result<()> {
        self.webgl_context = Some(
            self.canvas
                .get_context("webgl2")
                .map_err(|e| Error::msg(format!("{:?}", e)).context("Failed to initialize WebGL context"))?
                .ok_or_else(|| Error::msg("Failed to initialize WebGL context"))?
                .dyn_into::<web_sys::WebGl2RenderingContext>()
                .unwrap(),
        );

        Ok(())
    }

    pub fn load_gl_pointers(&self) -> Context {
        Context::from_webgl2_context(self.webgl_context.as_ref().unwrap().clone())
    }

    pub fn set_style(&mut self, _: WindowStyle) {
        // Styles are not supported by browsers
    }

    #[allow(clippy::redundant_clone)]
    pub fn init_closures<F>(&mut self, app: Rc<RefCell<ApplicationContext>>, event_loop: F)
    where
        F: FnMut() + Clone + 'static,
    {
        self.init_frame_callback(event_loop.clone());
        self.init_resize_callback(app.clone());
        self.init_mousemove_callback(app.clone());
        self.init_mouseenter_callback(app.clone());
        self.init_mouseleave_callback(app.clone());
        self.init_mousedown_callback(app.clone());
        self.init_mouseup_callback(app.clone());
        self.init_scroll_callback(app.clone());
        self.init_keydown_callback(app.clone());
        self.init_keyup_callback(app.clone());
        self.init_keypress_callback(app.clone());

        self.window.set_timeout_with_callback_and_timeout_and_arguments_0(self.resize_callback.as_ref().unwrap().as_ref().unchecked_ref(), 0).unwrap();
        self.window.request_animation_frame(self.frame_callback.as_ref().unwrap().as_ref().unchecked_ref()).unwrap();
    }

    fn init_frame_callback<F>(&mut self, mut event_loop: F)
    where
        F: FnMut() + Clone + 'static,
    {
        self.frame_callback = Some(Closure::<dyn FnMut()>::new(move || {
            event_loop();
        }));
    }

    fn init_resize_callback(&mut self, app: Rc<RefCell<ApplicationContext>>) {
        self.resize_callback = Some(Closure::<dyn FnMut()>::new(move || {
            let mut app = app.borrow_mut();
            let canvas = &app.window.canvas;
            let size = Coordinates::new(canvas.scroll_width(), canvas.scroll_height());

            canvas.set_width(size.x as u32);
            canvas.set_height(size.y as u32);

            app.window.event_queue.push_back(InputEvent::WindowSizeChange { size });
            app.window.size = size;
        }));
        self.window.add_event_listener_with_callback("resize", self.resize_callback.as_ref().unwrap().as_ref().unchecked_ref()).unwrap();
    }

    fn init_mousemove_callback(&mut self, app: Rc<RefCell<ApplicationContext>>) {
        self.mousemove_callback = Some(Closure::<dyn FnMut(_)>::new(move |event: MouseEvent| {
            let mut app = app.borrow_mut();
            let position = Coordinates::new(event.offset_x(), event.offset_y());
            let modifiers = app.window.get_modifiers();

            app.window.event_queue.push_back(InputEvent::MouseMove { position, modifiers });
            app.window.cursor_position = position;
        }));
        self.canvas.add_event_listener_with_callback("mousemove", self.mousemove_callback.as_ref().unwrap().as_ref().unchecked_ref()).unwrap();
    }

    fn init_mouseenter_callback(&mut self, app: Rc<RefCell<ApplicationContext>>) {
        self.mouseenter_callback = Some(Closure::<dyn FnMut(_)>::new(move |event: MouseEvent| {
            let mut app = app.borrow_mut();
            let position = Coordinates::new(event.offset_x(), event.offset_y());
            let modifiers = app.window.get_modifiers();

            app.window.event_queue.push_back(InputEvent::MouseEnter { position, modifiers });
            app.window.cursor_in_window = true;
        }));
        self.canvas.add_event_listener_with_callback("mouseenter", self.mouseenter_callback.as_ref().unwrap().as_ref().unchecked_ref()).unwrap();
    }

    fn init_mouseleave_callback(&mut self, app: Rc<RefCell<ApplicationContext>>) {
        self.mouseleave_callback = Some(Closure::<dyn FnMut(_)>::new(move |_: MouseEvent| {
            let mut app = app.borrow_mut();

            app.window.event_queue.push_back(InputEvent::MouseLeave);
            app.window.cursor_in_window = false;
        }));
        self.canvas.add_event_listener_with_callback("mouseleave", self.mouseleave_callback.as_ref().unwrap().as_ref().unchecked_ref()).unwrap();
    }

    fn init_mousedown_callback(&mut self, app: Rc<RefCell<ApplicationContext>>) {
        self.mousedown_callback = Some(Closure::<dyn FnMut(_)>::new(move |event: MouseEvent| {
            let mut app = app.borrow_mut();
            let button = match event.button() {
                0 => MouseButton::Left,
                1 => MouseButton::Middle,
                2 => MouseButton::Right,
                _ => MouseButton::Unknown,
            };

            if button != MouseButton::Unknown {
                let position = app.window.cursor_position;
                let modifiers = app.window.get_modifiers();

                app.window.event_queue.push_back(InputEvent::MouseButtonPress { button, position, modifiers });
                app.window.mouse_state[button as usize] = true;
            }
        }));
        self.canvas.add_event_listener_with_callback("mousedown", self.mousedown_callback.as_ref().unwrap().as_ref().unchecked_ref()).unwrap();
    }

    fn init_mouseup_callback(&mut self, app: Rc<RefCell<ApplicationContext>>) {
        self.mouseup_callback = Some(Closure::<dyn FnMut(_)>::new(move |event: MouseEvent| {
            let mut app = app.borrow_mut();
            let button = match event.button() {
                0 => MouseButton::Left,
                1 => MouseButton::Middle,
                2 => MouseButton::Right,
                _ => MouseButton::Unknown,
            };

            if button != MouseButton::Unknown {
                let position = app.window.cursor_position;
                let modifiers = app.window.get_modifiers();

                app.window.event_queue.push_back(InputEvent::MouseButtonRelease { button, position, modifiers });
                app.window.mouse_state[button as usize] = false;
            }
        }));
        self.canvas.add_event_listener_with_callback("mouseup", self.mouseup_callback.as_ref().unwrap().as_ref().unchecked_ref()).unwrap();
    }

    fn init_scroll_callback(&mut self, app: Rc<RefCell<ApplicationContext>>) {
        self.wheel_callback = Some(Closure::<dyn FnMut(_)>::new(move |event: WheelEvent| {
            let mut app = app.borrow_mut();
            let direction = if event.delta_y() < 0.0 { MouseWheelDirection::Up } else { MouseWheelDirection::Down };
            let modifiers = app.window.get_modifiers();

            app.window.event_queue.push_back(InputEvent::MouseWheelRotated { direction, modifiers });
        }));
        self.canvas.add_event_listener_with_callback("wheel", self.wheel_callback.as_ref().unwrap().as_ref().unchecked_ref()).unwrap();
    }

    fn init_keydown_callback(&mut self, app: Rc<RefCell<ApplicationContext>>) {
        self.keydown_callback = Some(Closure::<dyn FnMut(_)>::new(move |event: KeyboardEvent| {
            let mut app = app.borrow_mut();
            let key = map_key(event.code());

            if key != Key::Unknown {
                let repeat = app.window.keyboard_state[key as usize];
                let modifiers = app.window.get_modifiers();

                app.window.event_queue.push_back(InputEvent::KeyPress { key, repeat, modifiers });
                app.window.keyboard_state[key as usize] = true;
            }
        }));
        self.canvas.add_event_listener_with_callback("keydown", self.keydown_callback.as_ref().unwrap().as_ref().unchecked_ref()).unwrap();
    }

    fn init_keyup_callback(&mut self, app: Rc<RefCell<ApplicationContext>>) {
        self.keyup_callback = Some(Closure::<dyn FnMut(_)>::new(move |event: KeyboardEvent| {
            let mut app = app.borrow_mut();
            let key = map_key(event.code());

            if key != Key::Unknown {
                let modifiers = app.window.get_modifiers();

                app.window.event_queue.push_back(InputEvent::KeyRelease { key, modifiers });
                app.window.keyboard_state[key as usize] = false;
                app.window.last_character = None;
            }
        }));
        self.canvas.add_event_listener_with_callback("keyup", self.keyup_callback.as_ref().unwrap().as_ref().unchecked_ref()).unwrap();
    }

    fn init_keypress_callback(&mut self, app: Rc<RefCell<ApplicationContext>>) {
        self.keypress_callback = Some(Closure::<dyn FnMut(_)>::new(move |event: KeyboardEvent| {
            let mut app = app.borrow_mut();
            let mut character = event.key();
            let modifiers = app.window.get_modifiers();

            if character == "Enter" {
                character = "\r".to_string();
            }

            if character.len() == 1 {
                let character = character.chars().next().unwrap();
                let repeat = match app.window.last_character {
                    Some(c) => c == character,
                    None => false,
                };

                app.window.event_queue.push_back(InputEvent::CharPress { character, repeat, modifiers });
                app.window.last_character = Some(character);
            }
        }));
        self.canvas.add_event_listener_with_callback("keypress", self.keypress_callback.as_ref().unwrap().as_ref().unchecked_ref()).unwrap();
    }

    pub fn poll_event(&mut self) -> Option<InputEvent> {
        self.event_queue.pop_front()
    }

    pub fn get_modifiers(&self) -> Modifiers {
        Modifiers::new(self.keyboard_state[Key::Control as usize], self.keyboard_state[Key::Alt as usize], self.keyboard_state[Key::Shift as usize])
    }

    pub fn set_cursor_visibility(&mut self, visible: bool) {
        match visible {
            true => self.canvas.style().set_property("cursor", "default").unwrap(),
            false => self.canvas.style().set_property("cursor", "none").unwrap(),
        };

        self.cursor_visible = visible;
    }

    pub fn set_swap_interval(&self, _: u32) {
        // Swap interval is not supported by browsers
    }

    pub fn swap_buffers(&self) {
        self.window.request_animation_frame(self.frame_callback.as_ref().unwrap().as_ref().unchecked_ref()).unwrap();
    }

    pub fn close(&self) {
        // Window closing is not supported by browsers
    }
}

fn map_key(key: String) -> Key {
    match key.as_str() {
        "Enter" | "NumpadEnter" => Key::Enter,
        "Escape" => Key::Escape,
        "Backspace" => Key::Backspace,
        "Space" => Key::Space,
        "ControlLeft" | "ControlRight" => Key::Control,
        "ShiftLeft" | "ShiftRight" => Key::Shift,
        "AltLeft" | "AltRight" => Key::Alt,

        "ArrowLeft" => Key::ArrowLeft,
        "ArrowUp" => Key::ArrowUp,
        "ArrowRight" => Key::ArrowRight,
        "ArrowDown" => Key::ArrowDown,

        "Digit0" => Key::Key0,
        "Digit1" => Key::Key1,
        "Digit2" => Key::Key2,
        "Digit3" => Key::Key3,
        "Digit4" => Key::Key4,
        "Digit5" => Key::Key5,
        "Digit6" => Key::Key6,
        "Digit7" => Key::Key7,
        "Digit8" => Key::Key8,
        "Digit9" => Key::Key9,

        "F1" => Key::F1,
        "F2" => Key::F2,
        "F3" => Key::F3,
        "F4" => Key::F4,
        "F5" => Key::F5,
        "F6" => Key::F6,
        "F7" => Key::F7,
        "F8" => Key::F8,
        "F9" => Key::F9,
        "F10" => Key::F10,
        "F11" => Key::F11,
        "F12" => Key::F12,

        "KeyA" => Key::KeyA,
        "KeyB" => Key::KeyB,
        "KeyC" => Key::KeyC,
        "KeyD" => Key::KeyD,
        "KeyE" => Key::KeyE,
        "KeyF" => Key::KeyF,
        "KeyG" => Key::KeyG,
        "KeyH" => Key::KeyH,
        "KeyI" => Key::KeyI,
        "KeyJ" => Key::KeyJ,
        "KeyK" => Key::KeyK,
        "KeyL" => Key::KeyL,
        "KeyM" => Key::KeyM,
        "KeyN" => Key::KeyN,
        "KeyO" => Key::KeyO,
        "KeyP" => Key::KeyP,
        "KeyQ" => Key::KeyQ,
        "KeyR" => Key::KeyR,
        "KeyS" => Key::KeyS,
        "KeyT" => Key::KeyT,
        "KeyU" => Key::KeyU,
        "KeyV" => Key::KeyV,
        "KeyW" => Key::KeyW,
        "KeyX" => Key::KeyX,
        "KeyY" => Key::KeyY,
        "KeyZ" => Key::KeyZ,

        "Numpad0" => Key::Num0,
        "Numpad1" => Key::Num1,
        "Numpad2" => Key::Num2,
        "Numpad3" => Key::Num3,
        "Numpad4" => Key::Num4,
        "Numpad5" => Key::Num5,
        "Numpad6" => Key::Num6,
        "Numpad7" => Key::Num7,
        "Numpad8" => Key::Num8,
        "Numpad9" => Key::Num9,

        _ => Key::Unknown,
    }
}
