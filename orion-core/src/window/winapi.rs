use super::*;
use ::winapi::shared::basetsd::*;
use ::winapi::shared::minwindef::*;
use ::winapi::shared::windef::*;
use ::winapi::um::errhandlingapi;
use ::winapi::um::libloaderapi;
use ::winapi::um::wingdi::*;
use ::winapi::um::winuser;
use ::winapi::um::winuser::*;
use anyhow::bail;
use anyhow::Result;
use glow::Context;
use log::Level;
use std::collections::VecDeque;
use std::ffi::CString;
use std::mem;
use std::ptr;

pub type WGLCHOOSEPIXELFORMATARB = unsafe extern "C" fn(_: HDC, _: *const INT, _: *const FLOAT, _: UINT, _: *mut INT, _: *mut UINT) -> BOOL;
pub type WGLCREATECONTEXTATTRIBSARB = unsafe extern "C" fn(_: HDC, _: HGLRC, _: *const INT) -> HGLRC;

pub struct WindowContext {
    pub hwnd: HWND,
    pub hdc: HDC,
    pub wgl_context: Option<HGLRC>,

    pub size: Coordinates,
    pub cursor_position: Coordinates,
    pub cursor_in_window: bool,
    pub mouse_state: Vec<bool>,
    pub keyboard_state: Vec<bool>,

    phantom: bool,
    event_queue: VecDeque<InputEvent>,
}

impl WindowContext {
    pub fn new(title: &str, style: WindowStyle) -> Result<Box<Self>> {
        simple_logger::init_with_level(Level::Debug)?;

        unsafe {
            let title_cstr = CString::new(title).unwrap();
            let class_cstr = CString::new("OrionWindow").unwrap();
            let app_icon_cstr = CString::new("APP_ICON").unwrap();
            let cursor_icon_cstr = CString::new("CURSOR_ICON").unwrap();
            let module_handle = libloaderapi::GetModuleHandleA(ptr::null_mut());

            let window_class = WNDCLASSA {
                lpfnWndProc: Some(wnd_proc),
                hInstance: module_handle,
                hbrBackground: COLOR_BACKGROUND as HBRUSH,
                lpszClassName: class_cstr.as_ptr(),
                style: CS_OWNDC,
                cbClsExtra: 0,
                cbWndExtra: 0,
                hIcon: winuser::LoadImageA(module_handle, app_icon_cstr.as_ptr(), IMAGE_ICON, 0, 0, LR_DEFAULTSIZE) as HICON,
                hCursor: winuser::LoadImageA(module_handle, cursor_icon_cstr.as_ptr(), IMAGE_ICON, 0, 0, LR_DEFAULTSIZE) as HICON,
                lpszMenuName: ptr::null_mut(),
            };

            if winuser::RegisterClassA(&window_class) == 0 {
                bail!("RegisterClassA error: {}", errhandlingapi::GetLastError());
            }

            let mut context = Box::new(Self {
                hwnd: ptr::null_mut(),
                hdc: ptr::null_mut(),
                wgl_context: None,

                size: Coordinates::new(1, 1),
                cursor_position: Default::default(),
                cursor_in_window: false,
                mouse_state: vec![false; MouseButton::Unknown as usize],
                keyboard_state: vec![false; Key::Unknown as usize],

                phantom: false,
                event_queue: Default::default(),
            });

            let hwnd = winuser::CreateWindowExA(
                0,
                window_class.lpszClassName,
                title_cstr.as_ptr(),
                WS_OVERLAPPEDWINDOW,
                0,
                0,
                1,
                1,
                ptr::null_mut(),
                ptr::null_mut(),
                module_handle,
                context.as_mut() as *mut _ as LPVOID,
            );

            if hwnd.is_null() {
                bail!("CreateWindowExA error: {}", errhandlingapi::GetLastError());
            }

            // Wait for WM_CREATE, where the context is initialized
            while context.hdc.is_null() {}

            context.set_style(style);
            context.init_gl_context()?;

            winapi::SetForegroundWindow(context.hwnd);

            Ok(context)
        }
    }

    fn init_gl_context(&mut self) -> Result<()> {
        unsafe {
            let phantom_title_cstr = CString::new("Phantom").unwrap();
            let phantom_class_cstr = CString::new("OrionPhantom").unwrap();
            let phantom_module_handle = libloaderapi::GetModuleHandleA(ptr::null_mut());

            let phantom_window_class = WNDCLASSA {
                lpfnWndProc: Some(wnd_proc),
                hInstance: phantom_module_handle,
                hbrBackground: COLOR_BACKGROUND as HBRUSH,
                lpszClassName: phantom_class_cstr.as_ptr(),
                style: CS_OWNDC,
                cbClsExtra: 0,
                cbWndExtra: 0,
                hIcon: ptr::null_mut(),
                hCursor: ptr::null_mut(),
                lpszMenuName: ptr::null_mut(),
            };

            if winuser::RegisterClassA(&phantom_window_class) == 0 {
                bail!("RegisterClassA error: {}", errhandlingapi::GetLastError());
            }

            let mut phantom_context = Box::new(Self {
                hwnd: ptr::null_mut(),
                hdc: ptr::null_mut(),
                wgl_context: None,

                size: Coordinates::new(1, 1),
                cursor_position: Default::default(),
                cursor_in_window: false,
                mouse_state: Vec::new(),
                keyboard_state: Vec::new(),

                phantom: true,
                event_queue: Default::default(),
            });

            let phantom_hwnd = winuser::CreateWindowExA(
                0,
                phantom_window_class.lpszClassName,
                phantom_title_cstr.as_ptr(),
                WS_OVERLAPPEDWINDOW,
                0,
                0,
                1,
                1,
                ptr::null_mut(),
                ptr::null_mut(),
                phantom_module_handle,
                phantom_context.as_mut() as *mut _ as LPVOID,
            );

            if phantom_hwnd.is_null() {
                bail!("CreateWindowExA error: {}", errhandlingapi::GetLastError());
            }

            // Wait for WM_CREATE, where the context is initialized
            while phantom_context.hdc.is_null() {}

            let phantom_pixel_format_attributes = PIXELFORMATDESCRIPTOR {
                nSize: mem::size_of::<PIXELFORMATDESCRIPTOR>() as u16,
                nVersion: 1,
                dwFlags: PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER,
                iPixelType: PFD_TYPE_RGBA,
                cColorBits: 32,
                cRedBits: 0,
                cRedShift: 0,
                cGreenBits: 0,
                cGreenShift: 0,
                cBlueBits: 0,
                cBlueShift: 0,
                cAlphaBits: 0,
                cAlphaShift: 0,
                cAccumBits: 0,
                cAccumRedBits: 0,
                cAccumGreenBits: 0,
                cAccumBlueBits: 0,
                cAccumAlphaBits: 0,
                cDepthBits: 24,
                cStencilBits: 8,
                cAuxBuffers: 0,
                iLayerType: PFD_MAIN_PLANE,
                bReserved: 0,
                dwLayerMask: 0,
                dwVisibleMask: 0,
                dwDamageMask: 0,
            };

            let phantom_pixel_format = winapi::ChoosePixelFormat(phantom_context.hdc, &phantom_pixel_format_attributes);
            if winapi::SetPixelFormat(phantom_context.hdc, phantom_pixel_format, &phantom_pixel_format_attributes) == 0 {
                bail!("SetPixelFormat error: {}", errhandlingapi::GetLastError());
            }

            let phantom_gl_context = winapi::wglCreateContext(phantom_context.hdc);
            if winapi::wglMakeCurrent(phantom_context.hdc, phantom_gl_context) == 0 {
                bail!("wglMakeCurrent error: {}", errhandlingapi::GetLastError());
            }

            let wgl_choose_pixel_format_arb_cstr = CString::new("wglChoosePixelFormatARB").unwrap();
            let wgl_choose_pixel_format_arb_proc = winapi::wglGetProcAddress(wgl_choose_pixel_format_arb_cstr.as_ptr());
            let wgl_choose_pixel_format_arb = mem::transmute_copy::<_, WGLCHOOSEPIXELFORMATARB>(&wgl_choose_pixel_format_arb_proc);

            let wgl_create_context_attribs_arb_cstr = CString::new("wglCreateContextAttribsARB").unwrap();
            let wgl_create_context_attribs_arb_proc = winapi::wglGetProcAddress(wgl_create_context_attribs_arb_cstr.as_ptr());
            let wgl_create_context_attribs_arb = mem::transmute_copy::<_, WGLCREATECONTEXTATTRIBSARB>(&wgl_create_context_attribs_arb_proc);

            winapi::wglDeleteContext(phantom_gl_context);
            winapi::DestroyWindow(phantom_hwnd);

            let mut wgl_attributes = [
                8193, /* WGL_DRAW_TO_WINDOW_ARB */
                1,    /* true */
                8208, /* WGL_SUPPORT_OPENGL_ARB */
                1,    /* true */
                8209, /* WGL_DOUBLE_BUFFER_ARB */
                1,    /* true */
                8211, /* WGL_PIXEL_TYPE_ARB */
                8235, /* WGL_TYPE_RGBA_ARB */
                8212, /* WGL_COLOR_BITS_ARB */
                32,   /* 32 bits */
                8226, /* WGL_DEPTH_BITS_ARB */
                24,   /* 24 bits */
                8227, /* WGL_STENCIL_BITS_ARB */
                8,    /* 8 bits */
                8257, /* WGL_SAMPLE_BUFFERS_ARB */
                1,    /* true */
                8258, /* WGL_SAMPLES_ARB */
                16,   /* 16 samples */
                0,
            ];

            let mut pixel_format = 0;
            let mut formats_count = 0;
            let wgl_attributes_ptr = wgl_attributes.as_mut_ptr() as *const i32;

            if (wgl_choose_pixel_format_arb)(self.hdc, wgl_attributes_ptr, ptr::null_mut(), 1, &mut pixel_format, &mut formats_count) == 0 {
                bail!("wglChoosePixelFormatARB error");
            }

            if winapi::SetPixelFormat(self.hdc, pixel_format, &phantom_pixel_format_attributes) == 0 {
                bail!("SetPixelFormat error: {}", errhandlingapi::GetLastError());
            }

            let mut wgl_context_attributes = [8337 /* wgl::WGL_CONTEXT_MAJOR_VERSION_ARB */, 3, 8338 /* wgl::WGL_CONTEXT_MINOR_VERSION_ARB */, 3, 0];
            let wgl_context_attributes_ptr = wgl_context_attributes.as_mut_ptr() as *const i32;
            let wgl_context = (wgl_create_context_attribs_arb)(self.hdc, ptr::null_mut(), wgl_context_attributes_ptr);

            if winapi::wglMakeCurrent(self.hdc, wgl_context) == 0 {
                bail!("wglMakeCurrent error: {}", errhandlingapi::GetLastError());
            }

            self.wgl_context = Some(wgl_context);
        }

        Ok(())
    }

    pub fn set_style(&mut self, style: WindowStyle) {
        unsafe {
            if let WindowStyle::Fullscreen = style {
                winuser::ChangeDisplaySettingsA(ptr::null_mut(), 0);
            }

            match style {
                WindowStyle::Window { size } => {
                    let mut desktop_rect = mem::zeroed();
                    let mut rect = RECT { left: 0, top: 0, right: size.x, bottom: size.y };
                    let style = WS_OVERLAPPEDWINDOW | WS_VISIBLE;

                    winuser::GetWindowRect(winuser::GetDesktopWindow(), &mut desktop_rect);
                    winuser::SetWindowLongA(self.hwnd, GWL_STYLE, style as i32);
                    winuser::AdjustWindowRect(&mut rect, WS_OVERLAPPEDWINDOW, 0);

                    let width = rect.right - rect.left;
                    let height = rect.bottom - rect.top;

                    winuser::MoveWindow(self.hwnd, desktop_rect.right / 2 - width / 2, desktop_rect.bottom / 2 - height / 2, width, height, 1);

                    self.size = size;
                }
                WindowStyle::Borderless => {
                    let mut desktop_rect = mem::zeroed();
                    let style = WS_SYSMENU | WS_POPUP | WS_CLIPCHILDREN | WS_CLIPSIBLINGS | WS_VISIBLE;

                    winuser::GetWindowRect(winuser::GetDesktopWindow(), &mut desktop_rect);
                    winuser::SetWindowLongA(self.hwnd, GWL_STYLE, style as i32);
                    winuser::MoveWindow(self.hwnd, 0, 0, desktop_rect.right - desktop_rect.left, desktop_rect.bottom - desktop_rect.top, 1);
                }
                WindowStyle::Fullscreen => {
                    let mut desktop_rec = mem::zeroed();
                    let style = WS_SYSMENU | WS_POPUP | WS_CLIPCHILDREN | WS_CLIPSIBLINGS | WS_VISIBLE;

                    winuser::GetWindowRect(winuser::GetDesktopWindow(), &mut desktop_rec);
                    winuser::SetWindowLongA(self.hwnd, GWL_STYLE, style as i32);
                    winuser::MoveWindow(self.hwnd, 0, 0, desktop_rec.right - desktop_rec.left, desktop_rec.bottom - desktop_rec.top, 1);

                    let mut mode: DEVMODEA = mem::zeroed();
                    mode.dmSize = mem::size_of::<DEVMODEA>() as u16;
                    mode.dmPelsWidth = (desktop_rec.right - desktop_rec.left) as u32;
                    mode.dmPelsHeight = (desktop_rec.bottom - desktop_rec.top) as u32;
                    mode.dmBitsPerPel = 32;
                    mode.dmFields = DM_PELSWIDTH | DM_PELSHEIGHT | DM_BITSPERPEL;

                    winuser::ChangeDisplaySettingsA(&mut mode, CDS_FULLSCREEN);
                }
            }
        }
    }

    pub fn poll_event(&mut self) -> Option<InputEvent> {
        unsafe {
            let mut event: MSG = mem::zeroed();

            while winuser::PeekMessageA(&mut event, ptr::null_mut(), 0, 0, PM_REMOVE) > 0 {
                winuser::TranslateMessage(&event);
                winuser::DispatchMessageA(&event);

                match event.message {
                    WM_MOUSEMOVE => {
                        let x = (event.lParam as i32) & 0xffff;
                        let y = (event.lParam as i32) >> 16;

                        if !self.cursor_in_window {
                            winuser::TrackMouseEvent(&mut TRACKMOUSEEVENT {
                                cbSize: mem::size_of::<TRACKMOUSEEVENT>() as u32,
                                dwFlags: TME_LEAVE,
                                hwndTrack: self.hwnd,
                                dwHoverTime: 0,
                            });

                            let coordinates = Coordinates::new(x, self.size.y - y);
                            let modifiers = self.get_modifiers();

                            self.event_queue.push_back(InputEvent::MouseEnter { position: coordinates, modifiers });
                            self.cursor_in_window = true;
                        }

                        let coordinates = Coordinates::new(x, self.size.y - y);
                        let modifiers = self.get_modifiers();

                        self.event_queue.push_back(InputEvent::MouseMove { position: coordinates, modifiers });
                        self.cursor_position = coordinates;
                    }
                    WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN => {
                        let button = match event.message {
                            WM_LBUTTONDOWN => MouseButton::Left,
                            WM_RBUTTONDOWN => MouseButton::Right,
                            WM_MBUTTONDOWN => MouseButton::Middle,
                            _ => unreachable!(),
                        };
                        let position = self.cursor_position;
                        let modifiers = self.get_modifiers();

                        self.event_queue.push_back(InputEvent::MouseButtonPress { button, position, modifiers });
                        self.mouse_state[button as usize] = true;
                    }
                    WM_LBUTTONUP | WM_RBUTTONUP | WM_MBUTTONUP => {
                        let button = match event.message {
                            WM_LBUTTONUP => MouseButton::Left,
                            WM_RBUTTONUP => MouseButton::Right,
                            WM_MBUTTONUP => MouseButton::Middle,
                            _ => unreachable!(),
                        };
                        let position = self.cursor_position;
                        let modifiers = self.get_modifiers();

                        self.event_queue.push_back(InputEvent::MouseButtonRelease { button, position, modifiers });
                        self.mouse_state[button as usize] = false;
                    }
                    WM_MOUSEWHEEL => {
                        let direction = if ((event.wParam as i32) >> 16) > 0 { MouseWheelDirection::Up } else { MouseWheelDirection::Down };
                        let modifiers = self.get_modifiers();

                        self.event_queue.push_back(InputEvent::MouseWheelRotated { direction, modifiers });
                    }
                    WM_KEYDOWN => {
                        let key = map_key(event.wParam);

                        if key != Key::Unknown {
                            let repeat = (event.lParam & (1 << 30)) != 0;
                            let modifiers = self.get_modifiers();

                            self.event_queue.push_back(InputEvent::KeyPress { key, repeat, modifiers });
                            self.keyboard_state[key as usize] = true;
                        }
                    }
                    WM_KEYUP => {
                        let key = map_key(event.wParam);

                        if key != Key::Unknown {
                            let modifiers = self.get_modifiers();

                            self.event_queue.push_back(InputEvent::KeyRelease { key, modifiers });
                            self.keyboard_state[key as usize] = false;
                        }
                    }
                    WM_CHAR => {
                        let character = char::from_u32(event.wParam as u32).unwrap();
                        let repeat = (event.lParam & (1 << 30)) != 0;
                        let modifiers = self.get_modifiers();

                        self.event_queue.push_back(InputEvent::CharPress { character, repeat, modifiers })
                    }
                    WM_QUIT => self.event_queue.push_back(InputEvent::WindowClose),
                    _ => {}
                }
            }

            self.event_queue.pop_front()
        }
    }

    pub fn get_modifiers(&self) -> Modifiers {
        Modifiers::new(self.keyboard_state[Key::Control as usize], self.keyboard_state[Key::Alt as usize], self.keyboard_state[Key::Shift as usize])
    }

    pub fn load_gl_pointers(&self) -> Context {
        unsafe {
            let opengl32_dll_cstr = CString::new("opengl32.dll").unwrap();
            let opengl32_dll_handle = libloaderapi::LoadLibraryA(opengl32_dll_cstr.as_ptr());

            glow::Context::from_loader_function(|name| {
                let name_cstr = CString::new(name).unwrap();
                let mut proc = winapi::wglGetProcAddress(name_cstr.as_ptr());

                if proc.is_null() {
                    proc = libloaderapi::GetProcAddress(opengl32_dll_handle, name_cstr.as_ptr());
                }

                proc as *const _
            })
        }
    }
}

extern "system" fn wnd_proc(hwnd: HWND, message: u32, w_param: usize, l_param: isize) -> isize {
    unsafe {
        match message {
            WM_CREATE => {
                let create_struct = &mut *(l_param as *mut CREATESTRUCTA);
                let window = &mut *(create_struct.lpCreateParams as *mut WindowContext);
                let hdc: HDC = GetDC(hwnd);

                // Save pointer to the window context, so it can be used in all future events
                winuser::SetWindowLongPtrA(hwnd, GWLP_USERDATA, window as *mut _ as LONG_PTR);

                window.hwnd = hwnd;
                window.hdc = hdc;
            }
            WM_SIZE => {
                let window_ptr = winuser::GetWindowLongPtrA(hwnd, GWLP_USERDATA);
                let window = &mut *(window_ptr as *mut WindowContext);

                let x = (l_param & 0xffff) as i32;
                let y = (l_param >> 16) as i32;
                let size = Coordinates::new(x, y);

                window.event_queue.push_back(InputEvent::WindowSizeChange { size });
                window.size = size;
            }
            WM_MOUSELEAVE => {
                let window_ptr = winuser::GetWindowLongPtrA(hwnd, GWLP_USERDATA);
                let window = &mut *(window_ptr as *mut WindowContext);

                window.event_queue.push_back(InputEvent::MouseLeave);
                window.cursor_in_window = false;
            }
            WM_DESTROY => {
                let window_ptr = winuser::GetWindowLongPtrA(hwnd, GWLP_USERDATA);
                let window = &mut *(window_ptr as *mut WindowContext);

                window.hwnd = ptr::null_mut();
                window.hdc = ptr::null_mut();

                if !window.phantom {
                    winuser::PostQuitMessage(0);
                }

                return 0;
            }
            _ => {}
        }

        winuser::DefWindowProcA(hwnd, message, w_param, l_param)
    }
}

fn map_key(key: usize) -> Key {
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
