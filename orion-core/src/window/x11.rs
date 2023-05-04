use super::*;
use ::x11::glx;
use ::x11::keysym;
use ::x11::xlib;
use anyhow::bail;
use anyhow::Result;
use log::Level;
use std::collections::VecDeque;
use std::ffi::c_void;
use std::ffi::CString;
use std::mem;
use std::ptr;

pub struct WindowContext {
    pub window: u64,
    pub display: *mut xlib::_XDisplay,
    pub screen: i32,
    pub frame_buffer_config: *mut *mut glx::__GLXFBConfigRec,

    pub size: Coordinates,
    pub cursor_position: Coordinates,
    pub cursor_in_window: bool,

    delete_window_atom: u64,
    mouse_state: Vec<bool>,
    keyboard_state: Vec<bool>,
    event_queue: VecDeque<InputEvent>,
}

impl WindowContext {
    pub fn new(title: &str, style: WindowStyle) -> Result<Box<Self>> {
        simple_logger::init_with_level(Level::Debug)?;

        unsafe {
            let display = xlib::XOpenDisplay(ptr::null());
            if display.is_null() {
                bail!("Error while creating a new display".to_string());
            }

            let screen = xlib::XDefaultScreen(display);
            let attributes = [
                glx::GLX_X_RENDERABLE,
                1,
                glx::GLX_DRAWABLE_TYPE,
                glx::GLX_WINDOW_BIT,
                glx::GLX_RENDER_TYPE,
                glx::GLX_RGBA_BIT,
                glx::GLX_X_VISUAL_TYPE,
                glx::GLX_TRUE_COLOR,
                glx::GLX_RED_SIZE,
                8,
                glx::GLX_GREEN_SIZE,
                8,
                glx::GLX_BLUE_SIZE,
                8,
                glx::GLX_ALPHA_SIZE,
                8,
                glx::GLX_DEPTH_SIZE,
                24,
                glx::GLX_STENCIL_SIZE,
                8,
                glx::GLX_DOUBLEBUFFER,
                1,
                0,
            ];
            let attributes_ptr = attributes.as_ptr() as *const i32;

            let mut frame_buffers_count = 0;
            let frame_buffer_config = glx::glXChooseFBConfig(display, screen, attributes_ptr, &mut frame_buffers_count);
            if frame_buffer_config.is_null() {
                xlib::XCloseDisplay(display);
                bail!("Error while creating a new display".to_string());
            }
            let frame_buffer_config_slice = std::slice::from_raw_parts_mut(frame_buffer_config, frame_buffers_count as usize);

            let mut best_frame_buffer_config_index = -1;
            let mut worst_frame_buffer_config_index = -1;
            let mut best_samples = -1;
            let mut worst_samples = 999;

            for i in 0..frame_buffers_count {
                let config = frame_buffer_config_slice[i as usize];
                let visual_info = glx::glXGetVisualFromFBConfig(display, config);

                if !visual_info.is_null() {
                    let mut samp_buf = 0;
                    let mut samples = 0;

                    glx::glXGetFBConfigAttrib(display, config, glx::GLX_SAMPLE_BUFFERS as i32, &mut samp_buf);
                    glx::glXGetFBConfigAttrib(display, config, glx::GLX_SAMPLES as i32, &mut samples);

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
            xlib::XFree(frame_buffer_config as *mut c_void);

            let visual_info = glx::glXGetVisualFromFBConfig(display, best_frame_buffer_config);
            if visual_info.is_null() {
                xlib::XCloseDisplay(display);
                bail!("Error while creating a new display".to_string());
            }

            if screen != (*visual_info).screen {
                xlib::XCloseDisplay(display);
                bail!("Error while creating a new display".to_string());
            }

            let mut window_attributes = xlib::XSetWindowAttributes {
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
                event_mask: xlib::ExposureMask as i64
                    | xlib::StructureNotifyMask as i64
                    | xlib::ButtonPressMask as i64
                    | xlib::ButtonReleaseMask as i64
                    | xlib::KeyPressMask as i64
                    | xlib::KeyReleaseMask as i64
                    | xlib::PointerMotionMask as i64,
                do_not_propagate_mask: 0,
                override_redirect: 1,
                colormap: xlib::XCreateColormap(display, xlib::XRootWindow(display, screen), (*visual_info).visual, xlib::AllocNone as i32),
                cursor: 0,
            };

            let window_size = if let WindowStyle::Window { size } = style { size } else { Coordinates::new(1, 1) };
            let window = xlib::XCreateWindow(
                display,
                xlib::XRootWindow(display, screen),
                0,
                0,
                1 as u32,
                1 as u32,
                0,
                (*visual_info).depth,
                xlib::InputOutput as u32,
                (*visual_info).visual,
                xlib::CWBackPixel as u64 | xlib::CWColormap as u64 | xlib::CWBorderPixel as u64 | xlib::CWEventMask as u64,
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
                frame_buffer_config,

                size: Coordinates::new(800, 600),
                cursor_position: Default::default(),
                cursor_in_window: false,

                delete_window_atom,
                mouse_state: vec![false; 3],
                keyboard_state: vec![false; Key::Unknown as usize],
                event_queue: Default::default(),
            });
            context.set_style(style);

            Ok(context)
        }
    }

    pub fn set_style(&mut self, style: WindowStyle) -> Result<(), String> {
        unsafe {
            match style {
                WindowStyle::Window { size } => {
                    let screen_width = xlib::XDisplayWidth(self.display, self.screen);
                    let screen_height = xlib::XDisplayHeight(self.display, self.screen);

                    let net_wm_state_cstr = CString::new("_NET_WM_STATE").unwrap();
                    let net_wm_state_fullscreen_cstr = CString::new("_NET_WM_STATE_FULLSCREEN").unwrap();

                    let wm_state = xlib::XInternAtom(self.display, net_wm_state_cstr.as_ptr(), 1);
                    let wm_fullscreen = xlib::XInternAtom(self.display, net_wm_state_fullscreen_cstr.as_ptr(), 1);

                    let mut event: xlib::XEvent = mem::zeroed();
                    event.type_ = xlib::ClientMessage as i32;
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
                        xlib::SubstructureNotifyMask as i64 | xlib::SubstructureRedirectMask as i64,
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

                    let mut event: xlib::XEvent = mem::zeroed();
                    event.type_ = xlib::ClientMessage as i32;
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
                        xlib::SubstructureNotifyMask as i64 | xlib::SubstructureRedirectMask as i64,
                        &mut event,
                    );
                }
            }
        }

        Ok(())
    }

    pub fn poll_event(&mut self) -> Option<InputEvent> {
        unsafe {
            while xlib::XPending(self.display) > 0 {
                let mut event = mem::zeroed();
                xlib::XNextEvent(self.display, &mut event);

                match event.type_ {
                    xlib::ConfigureNotify => {
                        if event.configure.width != (self.size.x as i32) || event.configure.height != (self.size.y as i32) {
                            let size = Coordinates::new(event.configure.width, event.configure.height);
                            self.event_queue.push_back(InputEvent::WindowSizeChange { size });
                            self.size = size;
                        }
                    }
                    xlib::KeyPress => {
                        let mut buffer = vec![0; 1];
                        let buffer_ptr = buffer.as_mut_ptr() as *mut i8;
                        xlib::XLookupString(&mut event.key, buffer_ptr, 1, ptr::null_mut(), ptr::null_mut());

                        let keysym = xlib::XLookupKeysym(&event.key as *const _ as *mut xlib::XKeyEvent, 0);
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
                    xlib::KeyRelease => {
                        let keysym = xlib::XLookupKeysym(&event.key as *const _ as *mut xlib::XKeyEvent, 0);
                        let key = map_key(keysym as u32);
                        let modifiers = self.get_modifiers();

                        if xlib::XEventsQueued(self.display, 1) > 0 {
                            let mut next_event = mem::zeroed();
                            xlib::XPeekEvent(self.display, &mut next_event);

                            if next_event.type_ == xlib::KeyPress && next_event.key.keycode == event.key.keycode {
                                continue;
                            }
                        }

                        if key != Key::Unknown {
                            self.event_queue.push_back(InputEvent::KeyRelease { key, modifiers });
                            self.keyboard_state[key as usize] = false;
                        }
                    }
                    xlib::ButtonPress => {
                        let position = self.cursor_position;
                        let button = match event.button.button {
                            xlib::Button1 => MouseButton::Left,
                            xlib::Button2 => MouseButton::Middle,
                            xlib::Button3 => MouseButton::Right,
                            _ => MouseButton::Unknown,
                        };
                        let modifiers = self.get_modifiers();

                        if button != MouseButton::Unknown {
                            self.event_queue.push_back(InputEvent::MouseButtonPress { button, position, modifiers });
                            self.mouse_state[(event.button.button as usize) - 1] = true;
                        }
                    }
                    xlib::ButtonRelease => {
                        let position = self.cursor_position;
                        let button = match event.button.button {
                            xlib::Button1 => MouseButton::Left,
                            xlib::Button2 => MouseButton::Middle,
                            xlib::Button3 => MouseButton::Right,
                            _ => MouseButton::Unknown,
                        };
                        let modifiers = self.get_modifiers();

                        if button != MouseButton::Unknown {
                            self.event_queue.push_back(InputEvent::MouseButtonRelease { button, position, modifiers });
                            self.mouse_state[(event.button.button as usize) - 1] = false;
                        } else {
                            let direction = match event.button.button {
                                xlib::Button4 => MouseWheelDirection::Up,
                                xlib::Button5 => MouseWheelDirection::Down,
                                _ => MouseWheelDirection::Unknown,
                            };

                            if direction != MouseWheelDirection::Unknown {
                                self.event_queue.push_back(InputEvent::MouseWheelRotated { direction, modifiers });
                            }
                        }
                    }
                    xlib::MotionNotify => {
                        let position = Coordinates::new(event.motion.x, self.size.y - event.motion.y);
                        let modifiers = self.get_modifiers();

                        self.event_queue.push_back(InputEvent::MouseMove { position, modifiers });
                        self.cursor_position = position;
                    }
                    xlib::ClientMessage => {
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
        keysym::XK_Return => Key::Enter,
        keysym::XK_Escape => Key::Escape,
        keysym::XK_BackSpace => Key::Backspace,
        keysym::XK_space => Key::Space,
        keysym::XK_Control_L | keysym::XK_Control_R => Key::Control,
        keysym::XK_Shift_L | keysym::XK_Shift_R | keysym::XK_ISO_Level3_Shift => Key::Shift,
        keysym::XK_Alt_L | keysym::XK_Alt_R => Key::Alt,

        keysym::XK_Left => Key::ArrowLeft,
        keysym::XK_Up => Key::ArrowUp,
        keysym::XK_Right => Key::ArrowRight,
        keysym::XK_Down => Key::ArrowDown,

        keysym::XK_0 => Key::Key0,
        keysym::XK_1 => Key::Key1,
        keysym::XK_2 => Key::Key2,
        keysym::XK_3 => Key::Key3,
        keysym::XK_4 => Key::Key4,
        keysym::XK_5 => Key::Key5,
        keysym::XK_6 => Key::Key6,
        keysym::XK_7 => Key::Key7,
        keysym::XK_8 => Key::Key8,
        keysym::XK_9 => Key::Key9,

        keysym::XK_F1 => Key::F1,
        keysym::XK_F2 => Key::F2,
        keysym::XK_F3 => Key::F3,
        keysym::XK_F4 => Key::F4,
        keysym::XK_F5 => Key::F5,
        keysym::XK_F6 => Key::F6,
        keysym::XK_F7 => Key::F7,
        keysym::XK_F8 => Key::F8,
        keysym::XK_F9 => Key::F9,
        keysym::XK_F10 => Key::F10,
        keysym::XK_F11 => Key::F11,
        keysym::XK_F12 => Key::F12,

        keysym::XK_A | keysym::XK_a => Key::KeyA,
        keysym::XK_B | keysym::XK_b => Key::KeyB,
        keysym::XK_C | keysym::XK_c => Key::KeyC,
        keysym::XK_D | keysym::XK_d => Key::KeyD,
        keysym::XK_E | keysym::XK_e => Key::KeyE,
        keysym::XK_F | keysym::XK_f => Key::KeyF,
        keysym::XK_G | keysym::XK_g => Key::KeyG,
        keysym::XK_H | keysym::XK_h => Key::KeyH,
        keysym::XK_I | keysym::XK_i => Key::KeyI,
        keysym::XK_J | keysym::XK_j => Key::KeyJ,
        keysym::XK_K | keysym::XK_k => Key::KeyK,
        keysym::XK_L | keysym::XK_l => Key::KeyL,
        keysym::XK_M | keysym::XK_m => Key::KeyM,
        keysym::XK_N | keysym::XK_n => Key::KeyN,
        keysym::XK_O | keysym::XK_o => Key::KeyO,
        keysym::XK_P | keysym::XK_p => Key::KeyP,
        keysym::XK_Q | keysym::XK_q => Key::KeyQ,
        keysym::XK_R | keysym::XK_r => Key::KeyR,
        keysym::XK_S | keysym::XK_s => Key::KeyS,
        keysym::XK_T | keysym::XK_t => Key::KeyT,
        keysym::XK_U | keysym::XK_u => Key::KeyU,
        keysym::XK_V | keysym::XK_v => Key::KeyV,
        keysym::XK_W | keysym::XK_w => Key::KeyW,
        keysym::XK_X | keysym::XK_x => Key::KeyX,
        keysym::XK_Y | keysym::XK_y => Key::KeyY,
        keysym::XK_Z | keysym::XK_z => Key::KeyZ,

        keysym::XK_KP_0 | keysym::XK_KP_Insert => Key::Num0,
        keysym::XK_KP_1 | keysym::XK_KP_End => Key::Num1,
        keysym::XK_KP_2 | keysym::XK_KP_Down => Key::Num2,
        keysym::XK_KP_3 | keysym::XK_KP_Page_Down => Key::Num3,
        keysym::XK_KP_4 | keysym::XK_KP_Left => Key::Num4,
        keysym::XK_KP_5 | keysym::XK_KP_Begin => Key::Num5,
        keysym::XK_KP_6 | keysym::XK_KP_Right => Key::Num6,
        keysym::XK_KP_7 | keysym::XK_KP_Home => Key::Num7,
        keysym::XK_KP_8 | keysym::XK_KP_Up => Key::Num8,
        keysym::XK_KP_9 | keysym::XK_KP_Page_Up => Key::Num9,

        _ => Key::Unknown,
    }
}
