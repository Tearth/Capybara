use super::*;
use crate::error_return;
use anyhow::bail;
use anyhow::Result;
use js_sys::ArrayBuffer;
use js_sys::Uint8Array;
use log::error;
use log::info;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::Request;
use web_sys::RequestCache;
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
            let blob = match blob.dyn_into::<ArrayBuffer>() {
                Ok(blob) => blob,
                Err(_) => {
                    *status_clone.borrow_mut() = FileLoadingStatus::Error;
                    error_return!("Failed to cast blob")
                }
            };

            info!("Retrieved blob from response ({} bytes)", blob.byte_length());

            let array = Uint8Array::new(&blob);
            let mut buffer = buffer_clone.borrow_mut();

            buffer.resize(blob.byte_length() as usize, 0);
            array.copy_to(&mut buffer);

            *status_clone.borrow_mut() = FileLoadingStatus::Finished;
            info!("Fetching finished");
        });

        let status_clone = status.clone();
        let fetch_closure = Rc::new(RefCell::new(Closure::<dyn FnMut(_)>::new(move |response: JsValue| {
            let response = match response.dyn_into::<Response>() {
                Ok(response) => response,
                Err(_) => {
                    *status_clone.borrow_mut() = FileLoadingStatus::Error;
                    error_return!("Failed to cast response")
                }
            };

            info!("Response received with code {} ({})", response.status(), response.status_text());

            let array_buffer = match response.array_buffer() {
                Ok(array_buffer) => array_buffer,
                Err(_) => {
                    *status_clone.borrow_mut() = FileLoadingStatus::Error;
                    error_return!("Failed to load array buffer from response")
                }
            };

            let _ = array_buffer.then(&blob_closure);
        })));

        let window = web_sys::window().unwrap();
        let storage = window.local_storage().unwrap().unwrap();

        Self { input, status, buffer, window, storage, fetch_closure }
    }

    pub fn read(&mut self, path: &str) -> FileLoadingStatus {
        let status = *self.status.borrow_mut();

        if status == FileLoadingStatus::Idle {
            info!("Reading from file {}", path);
        }

        if (status == FileLoadingStatus::Finished || status == FileLoadingStatus::Error) && *self.input.borrow() != path {
            *self.status.borrow_mut() = FileLoadingStatus::Idle;
        }

        if let FileLoadingStatus::Idle = status {
            let mut init = RequestInit::new();

            #[cfg(debug_assertions)]
            init.cache(RequestCache::NoStore);

            #[cfg(not(debug_assertions))]
            init.cache(RequestCache::NoCache);

            let request = Request::new_with_str_and_init(path, &init).unwrap();
            let fetch_closure_clone = self.fetch_closure.clone();
            let _ = self.window.fetch_with_request(&request).then(&fetch_closure_clone.borrow());

            info!("Fetching {}", path);

            *self.input.borrow_mut() = path.to_string();
            *self.status.borrow_mut() = FileLoadingStatus::Loading;
        }

        *self.status.borrow()
    }

    pub fn write(&self, _: &str, _: &str) {
        error!("Writing files not supported on Web")
    }

    pub fn read_local(&self, path: &str) -> Result<String> {
        info!("Reading from local file {}", path);

        if let Ok(settings) = self.storage.get(path) {
            if let Some(settings) = settings {
                return Ok(settings);
            } else {
                return Ok(String::new());
            }
        }

        bail!("Local storage is not available")
    }

    pub fn write_local(&self, path: &str, content: &str) {
        info!("Writing to local file {} ({} bytes)", path, content.len());

        if self.storage.set(path, content).is_err() {
            error!("Failed to write into local storage");
        }
    }
}

impl Default for FileSystem {
    fn default() -> Self {
        Self::new()
    }
}
