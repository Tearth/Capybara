use super::*;
use anyhow::bail;
use anyhow::Result;
use std::collections::VecDeque;
use std::ffi::CString;
use std::mem;
use std::ptr;
use winapi::shared::basetsd;
use winapi::shared::minwindef;
use winapi::shared::windef;
use winapi::um::errhandlingapi;
use winapi::um::libloaderapi;
use winapi::um::winuser;
use winapi::um::winuser::WNDCLASSA;

pub struct Window {
    pub hwnd: windef::HWND,
    pub hdc: windef::HDC,
    pub initialized: bool,

    wnd_proc_events: VecDeque<WndProcEvent>,
}

pub struct WndProcEvent {
    message: u32,
    l_param: isize,
}

impl Window {
    pub fn new(title: &str) -> Result<Box<Self>> {
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

            let mut context = Box::new(Self { hwnd: ptr::null_mut(), hdc: ptr::null_mut(), initialized: false, wnd_proc_events: Default::default() });
            let title_cstr = CString::new(title).unwrap();

            let hwnd = winuser::CreateWindowExA(
                0,
                window_class.lpszClassName,
                title_cstr.as_ptr(),
                winuser::WS_OVERLAPPEDWINDOW | winuser::WS_VISIBLE,
                0,
                0,
                800,
                600,
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

    pub fn poll_event(&mut self) -> Vec<InputEvent> {
        unsafe {
            let mut event: winuser::MSG = mem::zeroed();

            if winuser::PeekMessageA(&mut event, ptr::null_mut(), 0, 0, winuser::PM_REMOVE) > 0 {
                winuser::TranslateMessage(&event);
                winuser::DispatchMessageA(&event);

                match event.message {
                    winuser::WM_MOUSEMOVE => {
                        let x = (event.lParam as i32) & 0xffff;
                        let y = (event.lParam as i32) >> 16;

                        return vec![InputEvent::MouseMoved(x, y)];
                    }
                    _ => {}
                }
            }

            if let Some(event) = self.wnd_proc_events.pop_back() {
                match event.message {
                    _ => return Vec::new(),
                }
            }

            Vec::new()
        }
    }
}

extern "system" fn wnd_proc(hwnd: windef::HWND, message: u32, w_param: usize, l_param: isize) -> isize {
    unsafe {
        match message {
            winuser::WM_CREATE => {
                let create_struct = &mut *(l_param as *mut winuser::CREATESTRUCTA);
                let window = &mut *(create_struct.lpCreateParams as *mut Window);
                let hdc: windef::HDC = winuser::GetDC(hwnd);

                // Save pointer to the window context, so it can be used in all future events
                winuser::SetWindowLongPtrA(hwnd, winuser::GWLP_USERDATA, window as *mut _ as basetsd::LONG_PTR);

                window.hwnd = hwnd;
                window.hdc = hdc;
                window.initialized = true;
            }
            winuser::WM_MOVE | winuser::WM_SIZE => {
                let window_ptr = winuser::GetWindowLongPtrA(hwnd, winuser::GWLP_USERDATA);
                let window = &mut *(window_ptr as *mut Window);

                window.wnd_proc_events.push_front(WndProcEvent { message, l_param });
            }
            winuser::WM_CLOSE => {
                if winuser::DestroyWindow(hwnd) == 0 {
                    panic!("{}", errhandlingapi::GetLastError());
                }

                return 0;
            }
            winuser::WM_DESTROY => {
                let window_ptr = winuser::GetWindowLongPtrA(hwnd, winuser::GWLP_USERDATA);
                let window = &mut *(window_ptr as *mut Window);

                window.hwnd = ptr::null_mut();
                window.hdc = ptr::null_mut();

                return 0;
            }
            _ => {}
        }

        winuser::DefWindowProcA(hwnd, message, w_param, l_param)
    }
}
