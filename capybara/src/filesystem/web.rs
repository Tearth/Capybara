use super::*;
use anyhow::bail;
use anyhow::Result;
use js_sys::ArrayBuffer;
use js_sys::Uint8Array;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::Request;
use web_sys::RequestInit;
use web_sys::Response;
use web_sys::Storage;
use web_sys::Window;

pub struct FileSystem {
    pub input: Rc<RefCell<String>>,
    pub status: Rc<RefCell<FileLoadingStatus>>,
    pub buffer: Rc<RefCell<Vec<u8>>>,

    window: Window,
    storage: Storage,
    fetch_closure: Rc<RefCell<Closure<dyn FnMut(JsValue)>>>,
}

impl FileSystem {
    pub fn new() -> Self {
        let input = Rc::new(RefCell::new(String::new()));
        let status = Rc::new(RefCell::new(FileLoadingStatus::Idle));
        let buffer = Rc::new(RefCell::new(Vec::new()));

        let status_clone = status.clone();
        let buffer_clone = buffer.clone();

        let blob_closure = Closure::<dyn FnMut(_)>::new(move |blob: JsValue| {
            let blob = blob.dyn_into::<ArrayBuffer>().unwrap();
            let array = Uint8Array::new(&blob);
            let mut buffer = buffer_clone.borrow_mut();

            buffer.resize(blob.byte_length() as usize, 0);
            array.copy_to(&mut buffer);

            *status_clone.borrow_mut() = FileLoadingStatus::Finished;
        });

        let fetch_closure = Rc::new(RefCell::new(Closure::<dyn FnMut(_)>::new(move |response: JsValue| {
            let response = response.dyn_into::<Response>().unwrap();
            let _ = response.array_buffer().unwrap().then(&blob_closure);
        })));

        let window = web_sys::window().unwrap();
        let storage = window.local_storage().unwrap().unwrap();

        Self { input, status, buffer, window, storage, fetch_closure }
    }

    pub fn read(&mut self, input: &str) -> Result<FileLoadingStatus> {
        let status = *self.status.borrow_mut();

        if status == FileLoadingStatus::Finished && *self.input.borrow() != input {
            *self.status.borrow_mut() = FileLoadingStatus::Idle;
        }

        if let FileLoadingStatus::Idle = status {
            let request = Request::new_with_str_and_init(input, &RequestInit::new()).unwrap();
            let fetch_closure_clone = self.fetch_closure.clone();
            let _ = self.window.fetch_with_request(&request).then(&fetch_closure_clone.borrow());

            *self.input.borrow_mut() = input.to_string();
            *self.status.borrow_mut() = FileLoadingStatus::Loading;
        }

        Ok(*self.status.borrow())
    }

    pub fn write(&self, _: &str, _: &str) -> Result<()> {
        bail!("Writing files not supported on Web")
    }

    pub fn read_local(&self, path: &str) -> Result<String> {
        if let Ok(settings) = self.storage.get(path) {
            if let Some(settings) = settings {
                return Ok(settings);
            } else {
                return Ok("".to_string());
            }
        }

        bail!("Local storage is not available")
    }

    pub fn write_local(&self, path: &str, content: &str) -> Result<()> {
        self.storage.set(path, content).unwrap();
        Ok(())
    }
}

impl Default for FileSystem {
    fn default() -> Self {
        Self::new()
    }
}
