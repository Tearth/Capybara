use super::*;
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

pub struct FileSystem {
    pub input: Rc<RefCell<String>>,
    pub status: Rc<RefCell<FileLoadingStatus>>,
    pub buffer: Rc<RefCell<Vec<u8>>>,

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

        Self { input, status, buffer, fetch_closure }
    }

    pub fn load(&mut self, input: &str) -> Result<FileLoadingStatus> {
        let status = *self.status.borrow_mut();

        if let FileLoadingStatus::Idle = status {
            let mut opts = RequestInit::new();
            opts.method("GET");

            let window = web_sys::window().unwrap();
            let request = Request::new_with_str_and_init(input, &opts).unwrap();
            let fetch_closure_clone = self.fetch_closure.clone();
            let _ = window.fetch_with_request(&request).then(&fetch_closure_clone.borrow());

            *self.input.borrow_mut() = input.to_string();
            *self.status.borrow_mut() = FileLoadingStatus::Loading;
        }

        Ok(*self.status.borrow())
    }
}

impl Default for FileSystem {
    fn default() -> Self {
        Self::new()
    }
}
