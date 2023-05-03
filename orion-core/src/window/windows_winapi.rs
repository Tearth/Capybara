use super::*;
use anyhow::bail;
use anyhow::Result;
use log::Level;
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

    cursor_in_window: bool,
    event_queue: VecDeque<InputEvent>,
}

impl Window {
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

                            self.event_queue.push_back(InputEvent::MouseEnter(Coordinates::new(x, y)));
                            self.cursor_in_window = true;
                        }

                        self.event_queue.push_back(InputEvent::MouseMove(Coordinates::new(x, y)));
                    }
                    winuser::WM_QUIT => self.event_queue.push_back(InputEvent::WindowClose),
                    _ => {}
                }
            }

            self.event_queue.pop_front()
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
            winuser::WM_SIZE => {
                let window_ptr = winuser::GetWindowLongPtrA(hwnd, winuser::GWLP_USERDATA);
                let window = &mut *(window_ptr as *mut Window);

                let x = (l_param & 0xffff) as i32;
                let y = (l_param >> 16) as i32;

                window.event_queue.push_back(InputEvent::WindowSizeChange(Coordinates::new(x, y)));
            }
            winuser::WM_MOUSELEAVE => {
                let window_ptr = winuser::GetWindowLongPtrA(hwnd, winuser::GWLP_USERDATA);
                let window = &mut *(window_ptr as *mut Window);

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
                let window = &mut *(window_ptr as *mut Window);

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
