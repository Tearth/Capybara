use super::*;
use crate::*;
use ::winapi::shared::basetsd::*;
use ::winapi::shared::minwindef::*;
use ::winapi::shared::windef::*;
use ::winapi::um::errhandlingapi;
use ::winapi::um::libloaderapi;
use ::winapi::um::wingdi;
use ::winapi::um::wingdi::*;
use ::winapi::um::winuser;
use ::winapi::um::winuser::*;
use anyhow::bail;
use anyhow::Result;
use glow::Context;
use glow::HasContext;
use log::debug;
use log::error;
use log::info;
use log::Level;
use std::collections::VecDeque;
use std::ffi::CString;
use std::mem;
use std::ptr;

pub type WGLCHOOSEPIXELFORMATARB = unsafe extern "C" fn(_: HDC, _: *const INT, _: *const FLOAT, _: UINT, _: *mut INT, _: *mut UINT) -> BOOL;
pub type WGLCREATECONTEXTATTRIBSARB = unsafe extern "C" fn(_: HDC, _: HGLRC, _: *const INT) -> HGLRC;
pub type WGLSWAPINTERVALEXT = unsafe extern "C" fn(_: INT) -> BOOL;

pub struct WindowContextWinApi {
    pub hwnd: HWND,
    pub hdc: HDC,
    pub wgl_context: Option<HGLRC>,
    pub wgl_extensions: Option<WglExtensions>,

    pub size: Coordinates,
    pub cursor_visible: bool,
    pub cursor_position: Coordinates,
    pub cursor_in_window: bool,
    pub mouse_state: Vec<bool>,
    pub keyboard_state: Vec<bool>,

    phantom: bool,
    event_queue: VecDeque<InputEvent>,
}

pub struct WglExtensions {
    pub wgl_choose_pixel_format_arb: Option<WGLCHOOSEPIXELFORMATARB>,
    pub wgl_create_context_attribs_arb: Option<WGLCREATECONTEXTATTRIBSARB>,
    pub wgl_swap_interval_ext: Option<WGLSWAPINTERVALEXT>,
}

impl WindowContextWinApi {
    pub fn new(title: &str, style: WindowStyle) -> Result<Box<Self>> {
        unsafe {
            #[cfg(debug_assertions)]
            simple_logger::init_with_level(Level::Info)?;

            #[cfg(not(debug_assertions))]
            simple_logger::init_with_level(Level::Error)?;

            info!("Capybara {}", VERSION);
            info!("Window initialization");

            let title_cstr = CString::new(title).unwrap();
            let class_cstr = CString::new("CapybaraWindow").unwrap();
            let app_icon_cstr = CString::new("APP_ICON").unwrap();
            let cursor_icon_cstr = CString::new("CURSOR_ICON").unwrap();
            let module_handle = libloaderapi::GetModuleHandleA(ptr::null_mut());

            let window_class = WNDCLASSA {
                lpfnWndProc: Some(wnd_proc),
                hInstance: module_handle,
                hbrBackground: wingdi::CreateSolidBrush(0x00000000),
                lpszClassName: class_cstr.as_ptr(),
                style: CS_OWNDC,
                cbClsExtra: 0,
                cbWndExtra: 0,
                hIcon: winuser::LoadImageA(module_handle, app_icon_cstr.as_ptr(), IMAGE_ICON, 0, 0, LR_DEFAULTSIZE) as HICON,
                hCursor: winuser::LoadImageA(module_handle, cursor_icon_cstr.as_ptr(), IMAGE_ICON, 0, 0, LR_DEFAULTSIZE) as HICON,
                lpszMenuName: ptr::null_mut(),
            };

            if winuser::RegisterClassA(&window_class) == 0 {
                bail!("Failed to register window class, code {}", errhandlingapi::GetLastError());
            }

            let mut context = Box::new(Self {
                hwnd: ptr::null_mut(),
                hdc: ptr::null_mut(),
                wgl_context: None,
                wgl_extensions: None,

                size: Coordinates::new(1, 1),
                cursor_visible: true,
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
                bail!("Failed to create window, code {}", errhandlingapi::GetLastError());
            }

            // Wait for WM_CREATE, where the context is initialized
            while context.hdc.is_null() {}

            context.init_gl_context()?;
            context.set_style(style);

            if winapi::SetForegroundWindow(context.hwnd) == 0 {
                error!("Failed to set foreground window");
            }

            Ok(context)
        }
    }

    fn init_gl_context(&mut self) -> Result<()> {
        unsafe {
            info!("OpenGL context initialization");

            let phantom_title_cstr = CString::new("Phantom").unwrap();
            let phantom_class_cstr = CString::new("CapybaraPhantom").unwrap();
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
                bail!("Failed to register phantom window class, code {}", errhandlingapi::GetLastError());
            }

            let mut phantom_context = Box::new(Self {
                hwnd: ptr::null_mut(),
                hdc: ptr::null_mut(),
                wgl_context: None,
                wgl_extensions: None,

                size: Coordinates::new(1, 1),
                cursor_visible: true,
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
                bail!("Failed to create phantom window, code {}", errhandlingapi::GetLastError());
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
                bail!("Failed to set phantom pixel format, code {}", errhandlingapi::GetLastError());
            }

            let phantom_gl_context = winapi::wglCreateContext(phantom_context.hdc);
            if winapi::wglMakeCurrent(phantom_context.hdc, phantom_gl_context) == 0 {
                bail!("Failed to make phantom current context, code {}", errhandlingapi::GetLastError());
            }

            let phantom_wgl_extensions = WglExtensions::new();

            if winapi::wglDeleteContext(phantom_gl_context) == 0 {
                error!("Failed to delete phantom context, code {}", errhandlingapi::GetLastError());
            }

            if winapi::DestroyWindow(phantom_hwnd) == 0 {
                error!("Failed to destroy phantom window, code {}", errhandlingapi::GetLastError());
            }

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

            if let Some(wgl_choose_pixel_format_arb) = phantom_wgl_extensions.wgl_choose_pixel_format_arb {
                if (wgl_choose_pixel_format_arb)(self.hdc, wgl_attributes_ptr, ptr::null_mut(), 1, &mut pixel_format, &mut formats_count) == 0 {
                    bail!("Failed to choose pixel format");
                }
            } else {
                bail!("Failed to choose pixel format");
            }

            if winapi::SetPixelFormat(self.hdc, pixel_format, &phantom_pixel_format_attributes) == 0 {
                bail!("Failed to set pixel format, code {}", errhandlingapi::GetLastError());
            }

            let mut wgl_context_attributes = [
                8337, /* wgl::WGL_CONTEXT_MAJOR_VERSION_ARB */
                3,    /* */
                8338, /* wgl::WGL_CONTEXT_MINOR_VERSION_ARB */
                3,    /* */
                0,
            ];
            let wgl_context_attributes_ptr = wgl_context_attributes.as_mut_ptr() as *const i32;

            let wgl_context = if let Some(wgl_create_context_attribs_arb) = phantom_wgl_extensions.wgl_create_context_attribs_arb {
                (wgl_create_context_attribs_arb)(self.hdc, ptr::null_mut(), wgl_context_attributes_ptr)
            } else {
                bail!("Failed to create WGL context");
            };

            if winapi::wglMakeCurrent(self.hdc, wgl_context) == 0 {
                bail!("Failed to make current context, code {}", errhandlingapi::GetLastError());
            }

            self.wgl_context = Some(wgl_context);
            self.wgl_extensions = Some(WglExtensions::new());

            Ok(())
        }
    }

    pub fn load_gl_pointers(&self) -> Context {
        unsafe {
            let opengl32_dll_cstr = CString::new("opengl32.dll").unwrap();
            let opengl32_dll_handle = libloaderapi::LoadLibraryA(opengl32_dll_cstr.as_ptr());

            let gl = glow::Context::from_loader_function(|name| {
                let name_cstr = CString::new(name).unwrap();
                let mut proc = winapi::wglGetProcAddress(name_cstr.as_ptr());

                if proc.is_null() {
                    proc = libloaderapi::GetProcAddress(opengl32_dll_handle, name_cstr.as_ptr());
                }

                if proc.is_null() {
                    debug!("GL function {} unavailable", name);
                } else {
                    debug!("GL function {} loaded ({:?})", name, proc);
                }

                proc as *const _
            });

            let version = gl.version();
            info!("OpenGL {}.{} {}", version.major, version.minor, version.vendor_info);

            gl
        }
    }

    pub fn set_style(&mut self, style: WindowStyle) {
        unsafe {
            if let WindowStyle::Fullscreen = style {
                if winuser::ChangeDisplaySettingsA(ptr::null_mut(), 0) != winapi::DISP_CHANGE_SUCCESSFUL {
                    error!("Failed to change display settings, code {}", errhandlingapi::GetLastError());
                }
            }

            match style {
                WindowStyle::Window { size } => {
                    let mut desktop_rect = mem::zeroed();
                    let mut rect = RECT { left: 0, top: 0, right: size.x, bottom: size.y };
                    let style = WS_OVERLAPPEDWINDOW | WS_VISIBLE;

                    if winuser::GetWindowRect(winuser::GetDesktopWindow(), &mut desktop_rect) == 0 {
                        error!("Failed to retrieve window rect, code {}", errhandlingapi::GetLastError());
                    }

                    errhandlingapi::SetLastError(0);
                    winuser::SetWindowLongA(self.hwnd, GWL_STYLE, style as i32);

                    if errhandlingapi::GetLastError() != 0 {
                        error!("Failed to set window long, code {}", errhandlingapi::GetLastError());
                    }

                    if winuser::AdjustWindowRect(&mut rect, WS_OVERLAPPEDWINDOW, 0) == 0 {
                        error!("Failed adjust window rect, code {}", errhandlingapi::GetLastError());
                    }

                    let width = rect.right - rect.left;
                    let height = rect.bottom - rect.top;
                    let x = desktop_rect.right / 2 - width / 2;
                    let y = desktop_rect.bottom / 2 - height / 2;

                    if winuser::MoveWindow(self.hwnd, x, y, width, height, 1) == 0 {
                        error!("Failed to move window, code {}", errhandlingapi::GetLastError());
                    }

                    self.size = size;
                }
                WindowStyle::Borderless => {
                    let mut desktop_rect = mem::zeroed();
                    let style = WS_SYSMENU | WS_POPUP | WS_CLIPCHILDREN | WS_CLIPSIBLINGS | WS_VISIBLE;

                    if winuser::GetWindowRect(winuser::GetDesktopWindow(), &mut desktop_rect) == 0 {
                        error!("Failed to retrieve window rect, code {}", errhandlingapi::GetLastError());
                    }

                    errhandlingapi::SetLastError(0);
                    winuser::SetWindowLongA(self.hwnd, GWL_STYLE, style as i32);

                    if errhandlingapi::GetLastError() != 0 {
                        error!("Failed to set window long, code {}", errhandlingapi::GetLastError());
                    }

                    let width = desktop_rect.right - desktop_rect.left;
                    let height = desktop_rect.bottom - desktop_rect.top;

                    if winuser::MoveWindow(self.hwnd, 0, 0, width, height, 1) == 0 {
                        error!("Failed to move window, code {}", errhandlingapi::GetLastError());
                    }
                }
                WindowStyle::Fullscreen => {
                    let mut desktop_rec = mem::zeroed();
                    let style = WS_SYSMENU | WS_POPUP | WS_CLIPCHILDREN | WS_CLIPSIBLINGS | WS_VISIBLE;

                    if winuser::GetWindowRect(winuser::GetDesktopWindow(), &mut desktop_rec) == 0 {
                        error!("Failed to retrieve window rect, code {}", errhandlingapi::GetLastError());
                    }

                    errhandlingapi::SetLastError(0);
                    winuser::SetWindowLongA(self.hwnd, GWL_STYLE, style as i32);

                    if errhandlingapi::GetLastError() != 0 {
                        error!("Failed to set window long, code {}", errhandlingapi::GetLastError());
                    }

                    let width = desktop_rec.right - desktop_rec.left;
                    let height = desktop_rec.bottom - desktop_rec.top;

                    if winuser::MoveWindow(self.hwnd, 0, 0, width, height, 1) == 0 {
                        error!("Failed to move window, code {}", errhandlingapi::GetLastError());
                    }

                    let mut mode: DEVMODEA = mem::zeroed();
                    mode.dmSize = mem::size_of::<DEVMODEA>() as u16;
                    mode.dmPelsWidth = (desktop_rec.right - desktop_rec.left) as u32;
                    mode.dmPelsHeight = (desktop_rec.bottom - desktop_rec.top) as u32;
                    mode.dmBitsPerPel = 32;
                    mode.dmFields = DM_PELSWIDTH | DM_PELSHEIGHT | DM_BITSPERPEL;

                    if winuser::ChangeDisplaySettingsA(&mut mode, CDS_FULLSCREEN) != winapi::DISP_CHANGE_SUCCESSFUL {
                        error!("Failed to change display settings, code {}", errhandlingapi::GetLastError());
                    }
                }
            }
        }
    }

    pub fn poll_event(&mut self) -> Option<InputEvent> {
        unsafe {
            let mut event: MSG = mem::zeroed();

            while winuser::PeekMessageA(&mut event, ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                winuser::TranslateMessage(&event);
                winuser::DispatchMessageA(&event);

                match event.message {
                    WM_MOUSEMOVE => {
                        let x = (event.lParam as i32) & 0xffff;
                        let y = (event.lParam as i32) >> 16;

                        if !self.cursor_in_window {
                            let mut mouse_event = TRACKMOUSEEVENT {
                                cbSize: mem::size_of::<TRACKMOUSEEVENT>() as u32,
                                dwFlags: TME_LEAVE,
                                hwndTrack: self.hwnd,
                                dwHoverTime: 0,
                            };

                            if winuser::TrackMouseEvent(&mut mouse_event) == 0 {
                                error!("Failed to track mouse event, code {}", errhandlingapi::GetLastError());
                            }

                            let coordinates = Coordinates::new(x, y);
                            let modifiers = self.get_modifiers();

                            self.event_queue.push_back(InputEvent::MouseEnter { position: coordinates, modifiers });
                            self.cursor_in_window = true;
                        }

                        let coordinates = Coordinates::new(x, y);
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

    pub fn set_cursor_visibility(&mut self, visible: bool) {
        unsafe {
            match visible {
                true => while winapi::ShowCursor(1) < 0 {},
                false => while winapi::ShowCursor(0) >= 0 {},
            };

            self.cursor_visible = visible;
        }
    }

    pub fn set_swap_interval(&self, interval: u32) {
        unsafe {
            if let Some(wgl_extension) = &self.wgl_extensions {
                if let Some(wgl_swap_interval_ext) = wgl_extension.wgl_swap_interval_ext {
                    if (wgl_swap_interval_ext)(interval as i32) == 0 {
                        error!("Failed to change swap interval, code {}", errhandlingapi::GetLastError());
                    }
                } else {
                    error!("WGL extension wglSwapIntervalEXT not available");
                }
            } else {
                error!("WGL extensions not loaded");
            }
        }
    }

    pub fn swap_buffers(&self) {
        unsafe {
            if winapi::SwapBuffers(self.hdc) == 0 {
                error!("Failed to swap buffer, code {}", errhandlingapi::GetLastError());
            }
        }
    }

    pub fn close(&self) {
        unsafe {
            if winapi::DestroyWindow(self.hwnd) == 0 {
                error!("Failed to close window, code {}", errhandlingapi::GetLastError());
            }
        }
    }
}

extern "system" fn wnd_proc(hwnd: HWND, message: u32, w_param: usize, l_param: isize) -> isize {
    unsafe {
        match message {
            WM_CREATE => {
                let create_struct = &mut *(l_param as *mut CREATESTRUCTA);
                let window = &mut *(create_struct.lpCreateParams as *mut WindowContext);
                let hdc: HDC = winuser::GetDC(hwnd);

                errhandlingapi::SetLastError(0);

                // Save pointer to the window context, so it can be used in all future events
                winuser::SetWindowLongPtrA(hwnd, GWLP_USERDATA, window as *mut _ as LONG_PTR);

                if errhandlingapi::GetLastError() != 0 {
                    error!("Failed to set window long, code {}", errhandlingapi::GetLastError());
                }

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

impl WglExtensions {
    pub fn new() -> Self {
        Self {
            wgl_choose_pixel_format_arb: load_extension::<_>("wglChoosePixelFormatARB"),
            wgl_create_context_attribs_arb: load_extension::<_>("wglCreateContextAttribsARB"),
            wgl_swap_interval_ext: load_extension::<_>("wglSwapIntervalEXT"),
        }
    }
}

impl Default for WglExtensions {
    fn default() -> Self {
        Self::new()
    }
}

fn load_extension<T>(name: &str) -> Option<T> {
    unsafe {
        let extension_cstr = CString::new(name).unwrap();
        let extension_proc = winapi::wglGetProcAddress(extension_cstr.as_ptr());

        if extension_proc.is_null() {
            debug!("WGL extension {} not available", name);
            return None;
        }

        debug!("WGL extension {} loaded ({:?})", name, extension_proc);
        Some(mem::transmute_copy::<_, T>(&extension_proc))
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
