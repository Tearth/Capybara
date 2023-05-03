use super::*;
use ::winapi::shared::basetsd;
use ::winapi::shared::minwindef;
use ::winapi::shared::windef;
use ::winapi::um::errhandlingapi;
use ::winapi::um::libloaderapi;
use ::winapi::um::winuser;
use ::winapi::um::winuser::WNDCLASSA;
use anyhow::bail;
use anyhow::Result;
use log::Level;
use std::collections::VecDeque;
use std::ffi::CString;
use std::mem;
use std::ptr;

pub struct WindowContext {
    pub hwnd: windef::HWND,
    pub hdc: windef::HDC,
    pub initialized: bool,

    cursor_in_window: bool,
    event_queue: VecDeque<InputEvent>,
}

impl WindowContext {
    pub fn new(title: &str) -> Result<Box<Self>> {
        simple_logger::init_with_level(Level::Debug)?;

        unsafe {
            let class_cstr = CString::new("OrionWindow").unwrap();
            let app_icon_cstr = CString::new("APP_ICON").unwrap();
            let cursor_icon_cstr = CString::new("CURSOR_ICON").unwrap();
            let module_handle = libloaderapi::GetModuleHandleA(ptr::null_mut());

            let window_class = WNDCLASSA {
                lpfnWndProc: Some(wnd_proc),
                hInstance: module_handle,
                hbrBackground: winuser::COLOR_BACKGROUND as windef::HBRUSH,
                lpszClassName: class_cstr.as_ptr(),
                style: winuser::CS_OWNDC,
                cbClsExtra: 0,
                cbWndExtra: 0,
                hIcon: winuser::LoadImageA(module_handle, app_icon_cstr.as_ptr(), winuser::IMAGE_ICON, 0, 0, winuser::LR_DEFAULTSIZE) as windef::HICON,
                hCursor: winuser::LoadImageA(module_handle, cursor_icon_cstr.as_ptr(), winuser::IMAGE_ICON, 0, 0, winuser::LR_DEFAULTSIZE) as windef::HICON,
                lpszMenuName: ptr::null_mut(),
            };

            if winuser::RegisterClassA(&window_class) == 0 {
                bail!("Error while initializing a new window class, GetLastError()={}", errhandlingapi::GetLastError());
            }

            let mut context =
                Box::new(Self { hwnd: ptr::null_mut(), hdc: ptr::null_mut(), initialized: false, cursor_in_window: false, event_queue: Default::default() });
            let title_cstr = CString::new(title).unwrap();

            let mut size = windef::RECT { left: 0, top: 0, right: 800, bottom: 600 };
            winuser::AdjustWindowRect(&mut size, winuser::WS_OVERLAPPEDWINDOW | winuser::WS_VISIBLE, 0);

            let hwnd = winuser::CreateWindowExA(
                0,
                window_class.lpszClassName,
                title_cstr.as_ptr(),
                winuser::WS_OVERLAPPEDWINDOW | winuser::WS_VISIBLE,
                0,
                0,
                size.right - size.left,
                size.bottom - size.top,
                ptr::null_mut(),
                ptr::null_mut(),
                module_handle,
                context.as_mut() as *mut _ as minwindef::LPVOID,
            );

            if hwnd.is_null() {
                bail!("Error while initializing a new window instance, GetLastError()={}", errhandlingapi::GetLastError());
            }

            // Wait for WM_CREATE, where the context is initialized
            while !context.initialized {}

            Ok(context)
        }
    }

    pub fn poll_event(&mut self) -> Option<InputEvent> {
        unsafe {
            let mut event: winuser::MSG = mem::zeroed();

            while winuser::PeekMessageA(&mut event, ptr::null_mut(), 0, 0, winuser::PM_REMOVE) > 0 {
                winuser::TranslateMessage(&event);
                winuser::DispatchMessageA(&event);

                match event.message {
                    winuser::WM_KEYDOWN => {
                        let key = map_key(event.wParam);
                        let repeat = (event.lParam & (1 << 30)) != 0;
                        let modifiers = self.get_modifiers();

                        self.event_queue.push_back(InputEvent::KeyPress { key, repeat, modifiers });
                    }
                    winuser::WM_KEYUP => {
                        let key = map_key(event.wParam);
                        let modifiers = self.get_modifiers();

                        self.event_queue.push_back(InputEvent::KeyRelease { key, modifiers })
                    }
                    winuser::WM_CHAR => {
                        let character = char::from_u32(event.wParam as u32).unwrap();
                        let repeat = (event.lParam & (1 << 30)) != 0;
                        let modifiers = self.get_modifiers();

                        self.event_queue.push_back(InputEvent::CharPress { character, repeat, modifiers })
                    }
                    winuser::WM_MOUSEMOVE => {
                        let x = (event.lParam as i32) & 0xffff;
                        let y = (event.lParam as i32) >> 16;

                        if !self.cursor_in_window {
                            winuser::TrackMouseEvent(&mut winuser::TRACKMOUSEEVENT {
                                cbSize: mem::size_of::<winuser::TRACKMOUSEEVENT>() as u32,
                                dwFlags: winuser::TME_LEAVE,
                                hwndTrack: self.hwnd,
                                dwHoverTime: 0,
                            });

                            let coordinates = Coordinates::new(x, y);
                            let modifiers = self.get_modifiers();
                            self.event_queue.push_back(InputEvent::MouseEnter { coordinates, modifiers });

                            self.cursor_in_window = true;
                        }

                        let coordinates = Coordinates::new(x, y);
                        let modifiers = self.get_modifiers();
                        self.event_queue.push_back(InputEvent::MouseMove { coordinates, modifiers });
                    }
                    winuser::WM_QUIT => self.event_queue.push_back(InputEvent::WindowClose),
                    _ => {}
                }
            }

            self.event_queue.pop_front()
        }
    }

    pub fn get_modifiers(&self) -> Modifiers {
        unsafe {
            Modifiers::new(
                (winuser::GetKeyState(winuser::VK_CONTROL) as u16 & 0x8000) != 0,
                (winuser::GetKeyState(winuser::VK_MENU) as u16 & 0x8000) != 0,
                (winuser::GetKeyState(winuser::VK_SHIFT) as u16 & 0x8000) != 0,
            )
        }
    }
}

extern "system" fn wnd_proc(hwnd: windef::HWND, message: u32, w_param: usize, l_param: isize) -> isize {
    unsafe {
        match message {
            winuser::WM_CREATE => {
                let create_struct = &mut *(l_param as *mut winuser::CREATESTRUCTA);
                let window = &mut *(create_struct.lpCreateParams as *mut WindowContext);
                let hdc: windef::HDC = winuser::GetDC(hwnd);

                // Save pointer to the window context, so it can be used in all future events
                winuser::SetWindowLongPtrA(hwnd, winuser::GWLP_USERDATA, window as *mut _ as basetsd::LONG_PTR);

                window.hwnd = hwnd;
                window.hdc = hdc;
                window.initialized = true;
            }
            winuser::WM_SIZE => {
                let window_ptr = winuser::GetWindowLongPtrA(hwnd, winuser::GWLP_USERDATA);
                let window = &mut *(window_ptr as *mut WindowContext);

                let x = (l_param & 0xffff) as i32;
                let y = (l_param >> 16) as i32;
                let size = Coordinates::new(x, y);

                window.event_queue.push_back(InputEvent::WindowSizeChange { size });
            }
            winuser::WM_MOUSELEAVE => {
                let window_ptr = winuser::GetWindowLongPtrA(hwnd, winuser::GWLP_USERDATA);
                let window = &mut *(window_ptr as *mut WindowContext);

                window.event_queue.push_back(InputEvent::MouseLeave);
                window.cursor_in_window = false;
            }
            winuser::WM_CLOSE => {
                if winuser::DestroyWindow(hwnd) == 0 {
                    panic!("{}", errhandlingapi::GetLastError());
                }

                return 0;
            }
            winuser::WM_DESTROY => {
                let window_ptr = winuser::GetWindowLongPtrA(hwnd, winuser::GWLP_USERDATA);
                let window = &mut *(window_ptr as *mut WindowContext);

                window.hwnd = ptr::null_mut();
                window.hdc = ptr::null_mut();

                winuser::PostQuitMessage(0);

                return 0;
            }
            _ => {}
        }

        winuser::DefWindowProcA(hwnd, message, w_param, l_param)
    }
}

pub fn map_key(key: usize) -> Key {
    match key {
        0x0d => Key::Enter,
        0x1b => Key::Escape,
        0x08 => Key::Backspace,
        0x20 => Key::Space,
        0x11 => Key::Control,
        0x10 => Key::Shift,
        0x12 => Key::Alt,

        0x25 => Key::ArrowLeft,
        0x26 => Key::ArrowUp,
        0x27 => Key::ArrowRight,
        0x28 => Key::ArrowDown,

        0x30 => Key::Key0,
        0x31 => Key::Key1,
        0x32 => Key::Key2,
        0x33 => Key::Key3,
        0x34 => Key::Key4,
        0x35 => Key::Key5,
        0x36 => Key::Key6,
        0x37 => Key::Key7,
        0x38 => Key::Key8,
        0x39 => Key::Key9,

        0x70 => Key::F1,
        0x71 => Key::F2,
        0x72 => Key::F3,
        0x73 => Key::F4,
        0x74 => Key::F5,
        0x75 => Key::F6,
        0x76 => Key::F7,
        0x77 => Key::F8,
        0x78 => Key::F9,
        0x79 => Key::F10,
        0x7a => Key::F11,
        0x7b => Key::F12,

        0x41 => Key::KeyA,
        0x42 => Key::KeyB,
        0x43 => Key::KeyC,
        0x44 => Key::KeyD,
        0x45 => Key::KeyE,
        0x46 => Key::KeyF,
        0x47 => Key::KeyG,
        0x48 => Key::KeyH,
        0x49 => Key::KeyI,
        0x4a => Key::KeyJ,
        0x4b => Key::KeyK,
        0x4c => Key::KeyL,
        0x4d => Key::KeyM,
        0x4e => Key::KeyN,
        0x4f => Key::KeyO,
        0x50 => Key::KeyP,
        0x51 => Key::KeyQ,
        0x52 => Key::KeyR,
        0x53 => Key::KeyS,
        0x54 => Key::KeyT,
        0x55 => Key::KeyU,
        0x56 => Key::KeyV,
        0x57 => Key::KeyW,
        0x58 => Key::KeyX,
        0x59 => Key::KeyY,
        0x5a => Key::KeyZ,

        0x60 => Key::Num0,
        0x61 => Key::Num1,
        0x62 => Key::Num2,
        0x63 => Key::Num3,
        0x64 => Key::Num4,
        0x65 => Key::Num5,
        0x66 => Key::Num6,
        0x67 => Key::Num7,
        0x68 => Key::Num8,
        0x69 => Key::Num9,

        _ => Key::Unknown,
    }
}
