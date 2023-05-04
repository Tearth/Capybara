#![allow(non_upper_case_globals)]

use super::*;
use ::x11::glx;
use ::x11::glx::*;
use ::x11::keysym::*;
use ::x11::xlib;
use ::x11::xlib::*;
use anyhow::bail;
use anyhow::Result;
use log::Level;
use std::collections::VecDeque;
use std::ffi::c_void;
use std::ffi::CString;
use std::mem;
use std::ptr;
use std::slice;

pub struct WindowContext {
    pub window: u64,
    pub display: *mut _XDisplay,
    pub screen: i32,
    pub frame_buffer_config: *mut __GLXFBConfigRec,

    pub size: Coordinates,
    pub cursor_position: Coordinates,
    pub cursor_in_window: bool,
    pub mouse_state: Vec<bool>,
    pub keyboard_state: Vec<bool>,

    delete_window_atom: u64,
    event_queue: VecDeque<InputEvent>,
}

impl WindowContext {
    pub fn new(title: &str, style: WindowStyle) -> Result<Box<Self>> {
        simple_logger::init_with_level(Level::Debug)?;

        unsafe {
            let display = xlib::XOpenDisplay(ptr::null());
            if display.is_null() {
                bail!("XOpenDisplay error");
            }

            let screen = xlib::XDefaultScreen(display);
            let attributes = [
                GLX_X_RENDERABLE,
                1,
                GLX_DRAWABLE_TYPE,
                GLX_WINDOW_BIT,
                GLX_RENDER_TYPE,
                GLX_RGBA_BIT,
                GLX_X_VISUAL_TYPE,
                GLX_TRUE_COLOR,
                GLX_RED_SIZE,
                8,
                GLX_GREEN_SIZE,
                8,
                GLX_BLUE_SIZE,
                8,
                GLX_ALPHA_SIZE,
                8,
                GLX_DEPTH_SIZE,
                24,
                GLX_STENCIL_SIZE,
                8,
                GLX_DOUBLEBUFFER,
                1,
                0,
            ];

            let mut frame_buffers_count = 0;
            let attributes_ptr = attributes.as_ptr() as *const i32;
            let frame_buffer_config = glx::glXChooseFBConfig(display, screen, attributes_ptr, &mut frame_buffers_count);

            if frame_buffer_config.is_null() {
                bail!("glXChooseFBConfig error");
            }

            let mut best_frame_buffer_config_index = -1;
            let mut worst_frame_buffer_config_index = -1;
            let mut best_samples = -1;
            let mut worst_samples = 999;
            let frame_buffer_config_slice = slice::from_raw_parts_mut(frame_buffer_config, frame_buffers_count as usize);

            for i in 0..frame_buffers_count {
                let config = frame_buffer_config_slice[i as usize];
                let visual_info = glx::glXGetVisualFromFBConfig(display, config);

                if !visual_info.is_null() {
                    let mut samp_buf = 0;
                    let mut samples = 0;

                    glx::glXGetFBConfigAttrib(display, config, GLX_SAMPLE_BUFFERS as i32, &mut samp_buf);
                    glx::glXGetFBConfigAttrib(display, config, GLX_SAMPLES as i32, &mut samples);

                    if best_frame_buffer_config_index < 0 || (samp_buf != 0 && samples > best_samples) {
                        best_frame_buffer_config_index = i;
                        best_samples = samples;
                    }

                    if worst_frame_buffer_config_index < 0 || samp_buf == 0 || samples < worst_samples {
                        worst_frame_buffer_config_index = i;
                    }

                    worst_samples = samples;
                }

                xlib::XFree(visual_info as *mut c_void);
            }

            let best_frame_buffer_config = frame_buffer_config_slice[best_frame_buffer_config_index as usize];
            let visual_info = glx::glXGetVisualFromFBConfig(display, best_frame_buffer_config);

            if visual_info.is_null() || screen != (*visual_info).screen {
                bail!("glXGetVisualFromFBConfig error");
            }

            xlib::XFree(frame_buffer_config as *mut c_void);

            let event_mask = ExposureMask | StructureNotifyMask | ButtonPressMask | ButtonReleaseMask | KeyPressMask | KeyReleaseMask | PointerMotionMask;
            let colormap = xlib::XCreateColormap(display, xlib::XRootWindow(display, screen), (*visual_info).visual, AllocNone as i32);

            let mut window_attributes = XSetWindowAttributes {
                background_pixmap: 0,
                background_pixel: xlib::XWhitePixel(display, screen),
                border_pixmap: 0,
                border_pixel: xlib::XBlackPixel(display, screen),
                bit_gravity: 0,
                win_gravity: 0,
                backing_store: 0,
                backing_planes: 0,
                backing_pixel: 0,
                save_under: 0,
                event_mask,
                do_not_propagate_mask: 0,
                override_redirect: 1,
                colormap,
                cursor: 0,
            };

            let window = xlib::XCreateWindow(
                display,
                xlib::XRootWindow(display, screen),
                0,
                0,
                1,
                1,
                0,
                (*visual_info).depth,
                InputOutput as u32,
                (*visual_info).visual,
                CWBackPixel | CWColormap | CWBorderPixel | CWEventMask,
                &mut window_attributes,
            );

            let delete_window_cstr = CString::new("WM_DELETE_WINDOW").unwrap();
            let mut delete_window_atom = xlib::XInternAtom(display, delete_window_cstr.as_ptr(), 0);
            xlib::XSetWMProtocols(display, window, &mut delete_window_atom, 1);

            let title_cstr = CString::new(title).unwrap();

            xlib::XStoreName(display, window, title_cstr.as_ptr());
            xlib::XClearWindow(display, window);
            xlib::XMapRaised(display, window);

            let mut context = Box::new(Self {
                window,
                display,
                screen,
                frame_buffer_config: best_frame_buffer_config,

                size: Coordinates::new(1, 1),
                cursor_position: Default::default(),
                cursor_in_window: false,
                mouse_state: vec![false; 3],
                keyboard_state: vec![false; Key::Unknown as usize],

                delete_window_atom,
                event_queue: Default::default(),
            });
            context.set_style(style);

            Ok(context)
        }
    }

    pub fn set_style(&mut self, style: WindowStyle) {
        unsafe {
            match style {
                WindowStyle::Window { size } => {
                    let screen_width = xlib::XDisplayWidth(self.display, self.screen);
                    let screen_height = xlib::XDisplayHeight(self.display, self.screen);

                    let net_wm_state_cstr = CString::new("_NET_WM_STATE").unwrap();
                    let net_wm_state_fullscreen_cstr = CString::new("_NET_WM_STATE_FULLSCREEN").unwrap();

                    let wm_state = xlib::XInternAtom(self.display, net_wm_state_cstr.as_ptr(), 1);
                    let wm_fullscreen = xlib::XInternAtom(self.display, net_wm_state_fullscreen_cstr.as_ptr(), 1);

                    let mut event: XEvent = mem::zeroed();
                    event.type_ = ClientMessage as i32;
                    event.client_message.window = self.window;
                    event.client_message.format = 32;
                    event.client_message.message_type = wm_state;
                    event.client_message.data.set_long(0, 0);
                    event.client_message.data.set_long(1, wm_fullscreen as i64);
                    event.client_message.data.set_long(2, 0);
                    event.client_message.data.set_long(3, 1);

                    xlib::XSendEvent(
                        self.display,
                        xlib::XDefaultRootWindow(self.display),
                        0,
                        SubstructureNotifyMask as i64 | SubstructureRedirectMask as i64,
                        &mut event,
                    );

                    xlib::XMoveWindow(self.display, self.window, screen_width / 2 - size.x / 2, screen_height / 2 - size.y / 2);
                    xlib::XResizeWindow(self.display, self.window, size.x as u32, size.y as u32);
                }
                WindowStyle::Borderless | WindowStyle::Fullscreen => {
                    let net_wm_state_cstr = CString::new("_NET_WM_STATE").unwrap();
                    let net_wm_state_fullscreen_cstr = CString::new("_NET_WM_STATE_FULLSCREEN").unwrap();

                    let wm_state = xlib::XInternAtom(self.display, net_wm_state_cstr.as_ptr(), 1);
                    let wm_fullscreen = xlib::XInternAtom(self.display, net_wm_state_fullscreen_cstr.as_ptr(), 1);

                    let mut event: XEvent = mem::zeroed();
                    event.type_ = ClientMessage as i32;
                    event.client_message.window = self.window;
                    event.client_message.format = 32;
                    event.client_message.message_type = wm_state;
                    event.client_message.data.set_long(0, 1);
                    event.client_message.data.set_long(1, wm_fullscreen as i64);
                    event.client_message.data.set_long(2, 0);
                    event.client_message.data.set_long(3, 1);

                    xlib::XSendEvent(
                        self.display,
                        xlib::XDefaultRootWindow(self.display),
                        0,
                        SubstructureNotifyMask as i64 | SubstructureRedirectMask as i64,
                        &mut event,
                    );
                }
            }
        }
    }

    pub fn poll_event(&mut self) -> Option<InputEvent> {
        unsafe {
            while xlib::XPending(self.display) > 0 {
                let mut event = mem::zeroed();
                xlib::XNextEvent(self.display, &mut event);

                match event.type_ {
                    ConfigureNotify => {
                        if event.configure.width != (self.size.x as i32) || event.configure.height != (self.size.y as i32) {
                            let size = Coordinates::new(event.configure.width, event.configure.height);
                            self.event_queue.push_back(InputEvent::WindowSizeChange { size });
                            self.size = size;
                        }
                    }
                    KeyPress => {
                        let mut buffer = vec![0; 1];
                        let buffer_ptr = buffer.as_mut_ptr() as *mut i8;
                        xlib::XLookupString(&mut event.key, buffer_ptr, 1, ptr::null_mut(), ptr::null_mut());

                        let keysym = xlib::XLookupKeysym(&event.key as *const _ as *mut XKeyEvent, 0);
                        let key = map_key(keysym as u32);

                        if key != Key::Unknown {
                            let character = char::from_u32(buffer[0] as u32).unwrap();
                            let repeat = self.keyboard_state[key as usize];
                            let modifiers = self.get_modifiers();

                            self.event_queue.push_back(InputEvent::KeyPress { key, repeat, modifiers });
                            self.event_queue.push_back(InputEvent::CharPress { character, repeat, modifiers });
                            self.keyboard_state[key as usize] = true;
                        }
                    }
                    KeyRelease => {
                        let keysym = xlib::XLookupKeysym(&event.key as *const _ as *mut XKeyEvent, 0);
                        let key = map_key(keysym as u32);
                        let modifiers = self.get_modifiers();

                        if xlib::XEventsQueued(self.display, 1) > 0 {
                            let mut next_event = mem::zeroed();
                            xlib::XPeekEvent(self.display, &mut next_event);

                            if next_event.type_ == KeyPress && next_event.key.keycode == event.key.keycode {
                                continue;
                            }
                        }

                        if key != Key::Unknown {
                            self.event_queue.push_back(InputEvent::KeyRelease { key, modifiers });
                            self.keyboard_state[key as usize] = false;
                        }
                    }
                    ButtonPress => {
                        let position = self.cursor_position;
                        let button = match event.button.button {
                            Button1 => MouseButton::Left,
                            Button2 => MouseButton::Middle,
                            Button3 => MouseButton::Right,
                            _ => MouseButton::Unknown,
                        };
                        let modifiers = self.get_modifiers();

                        if button != MouseButton::Unknown {
                            self.event_queue.push_back(InputEvent::MouseButtonPress { button, position, modifiers });
                            self.mouse_state[(event.button.button as usize) - 1] = true;
                        }
                    }
                    ButtonRelease => {
                        let position = self.cursor_position;
                        let button = match event.button.button {
                            Button1 => MouseButton::Left,
                            Button2 => MouseButton::Middle,
                            Button3 => MouseButton::Right,
                            _ => MouseButton::Unknown,
                        };
                        let modifiers = self.get_modifiers();

                        if button != MouseButton::Unknown {
                            self.event_queue.push_back(InputEvent::MouseButtonRelease { button, position, modifiers });
                            self.mouse_state[(event.button.button as usize) - 1] = false;
                        } else {
                            let direction = match event.button.button {
                                Button4 => MouseWheelDirection::Up,
                                Button5 => MouseWheelDirection::Down,
                                _ => MouseWheelDirection::Unknown,
                            };

                            if direction != MouseWheelDirection::Unknown {
                                self.event_queue.push_back(InputEvent::MouseWheelRotated { direction, modifiers });
                            }
                        }
                    }
                    MotionNotify => {
                        let position = Coordinates::new(event.motion.x, self.size.y - event.motion.y);
                        let modifiers = self.get_modifiers();

                        self.event_queue.push_back(InputEvent::MouseMove { position, modifiers });
                        self.cursor_position = position;
                    }
                    ClientMessage => {
                        if event.client_message.data.get_long(0) == self.delete_window_atom as i64 {
                            self.event_queue.push_back(InputEvent::WindowClose);
                        }
                    }
                    _ => {}
                }
            }

            self.event_queue.pop_front()
        }
    }

    pub fn get_modifiers(&self) -> Modifiers {
        Modifiers::new(self.keyboard_state[Key::Control as usize], self.keyboard_state[Key::Alt as usize], self.keyboard_state[Key::Shift as usize])
    }
}

pub fn map_key(key: u32) -> Key {
    match key {
        XK_Return => Key::Enter,
        XK_Escape => Key::Escape,
        XK_BackSpace => Key::Backspace,
        XK_space => Key::Space,
        XK_Control_L | XK_Control_R => Key::Control,
        XK_Shift_L | XK_Shift_R | XK_ISO_Level3_Shift => Key::Shift,
        XK_Alt_L | XK_Alt_R => Key::Alt,

        XK_Left => Key::ArrowLeft,
        XK_Up => Key::ArrowUp,
        XK_Right => Key::ArrowRight,
        XK_Down => Key::ArrowDown,

        XK_0 => Key::Key0,
        XK_1 => Key::Key1,
        XK_2 => Key::Key2,
        XK_3 => Key::Key3,
        XK_4 => Key::Key4,
        XK_5 => Key::Key5,
        XK_6 => Key::Key6,
        XK_7 => Key::Key7,
        XK_8 => Key::Key8,
        XK_9 => Key::Key9,

        XK_F1 => Key::F1,
        XK_F2 => Key::F2,
        XK_F3 => Key::F3,
        XK_F4 => Key::F4,
        XK_F5 => Key::F5,
        XK_F6 => Key::F6,
        XK_F7 => Key::F7,
        XK_F8 => Key::F8,
        XK_F9 => Key::F9,
        XK_F10 => Key::F10,
        XK_F11 => Key::F11,
        XK_F12 => Key::F12,

        XK_A | XK_a => Key::KeyA,
        XK_B | XK_b => Key::KeyB,
        XK_C | XK_c => Key::KeyC,
        XK_D | XK_d => Key::KeyD,
        XK_E | XK_e => Key::KeyE,
        XK_F | XK_f => Key::KeyF,
        XK_G | XK_g => Key::KeyG,
        XK_H | XK_h => Key::KeyH,
        XK_I | XK_i => Key::KeyI,
        XK_J | XK_j => Key::KeyJ,
        XK_K | XK_k => Key::KeyK,
        XK_L | XK_l => Key::KeyL,
        XK_M | XK_m => Key::KeyM,
        XK_N | XK_n => Key::KeyN,
        XK_O | XK_o => Key::KeyO,
        XK_P | XK_p => Key::KeyP,
        XK_Q | XK_q => Key::KeyQ,
        XK_R | XK_r => Key::KeyR,
        XK_S | XK_s => Key::KeyS,
        XK_T | XK_t => Key::KeyT,
        XK_U | XK_u => Key::KeyU,
        XK_V | XK_v => Key::KeyV,
        XK_W | XK_w => Key::KeyW,
        XK_X | XK_x => Key::KeyX,
        XK_Y | XK_y => Key::KeyY,
        XK_Z | XK_z => Key::KeyZ,

        XK_KP_0 | XK_KP_Insert => Key::Num0,
        XK_KP_1 | XK_KP_End => Key::Num1,
        XK_KP_2 | XK_KP_Down => Key::Num2,
        XK_KP_3 | XK_KP_Page_Down => Key::Num3,
        XK_KP_4 | XK_KP_Left => Key::Num4,
        XK_KP_5 | XK_KP_Begin => Key::Num5,
        XK_KP_6 | XK_KP_Right => Key::Num6,
        XK_KP_7 | XK_KP_Home => Key::Num7,
        XK_KP_8 | XK_KP_Up => Key::Num8,
        XK_KP_9 | XK_KP_Page_Up => Key::Num9,

        _ => Key::Unknown,
    }
}