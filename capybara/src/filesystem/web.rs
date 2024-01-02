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
use web_sys::ProgressEvent;
use web_sys::Storage;
use web_sys::XmlHttpRequest;
use web_sys::XmlHttpRequestResponseType;

pub struct FileSystem {
    pub input: Rc<RefCell<String>>,
    pub status: Rc<RefCell<FileLoadingStatus>>,
    pub buffer: Rc<RefCell<Vec<u8>>>,
    pub progress: Rc<RefCell<f32>>,

    storage: Storage,
    request: Rc<RefCell<Option<XmlHttpRequest>>>,
    onload_closure: Rc<RefCell<Closure<dyn FnMut(ProgressEvent)>>>,
    onprogress_closure: Rc<RefCell<Closure<dyn FnMut(ProgressEvent)>>>,
}

impl FileSystem {
    pub fn new() -> Self {
        let input = Rc::new(RefCell::new(String::new()));
        let status = Rc::new(RefCell::new(FileLoadingStatus::Idle));
        let buffer = Rc::new(RefCell::new(Vec::new()));
        let progress = Rc::new(RefCell::new(0.0));
        let request: Rc<RefCell<Option<XmlHttpRequest>>> = Rc::new(RefCell::new(None));

        let status_clone = status.clone();
        let buffer_clone = buffer.clone();
        let progress_clone = progress.clone();
        let request_clone = request.clone();

        let onload_closure = Rc::new(RefCell::new(Closure::<dyn FnMut(_)>::new(move |event: ProgressEvent| {
            let request = request_clone.as_ref().borrow();
            let request = match request.as_ref() {
                Some(request) => request,
                None => {
                    *status_clone.borrow_mut() = FileLoadingStatus::Error;
                    error_return!("Loading is not in progress")
                }
            };

            let response = match request.response() {
                Ok(response) => response,
                Err(_) => {
                    *status_clone.borrow_mut() = FileLoadingStatus::Error;
                    error_return!("Response is not ready")
                }
            };

            let buffer = match response.dyn_into::<ArrayBuffer>() {
                Ok(response) => response,
                Err(_) => {
                    *status_clone.borrow_mut() = FileLoadingStatus::Error;
                    error_return!("Failed to cast response")
                }
            };

            let array = Uint8Array::new(&buffer);
            let mut buffer = buffer_clone.borrow_mut();

            buffer.resize(array.byte_length() as usize, 0);
            array.copy_to(&mut buffer);

            *status_clone.borrow_mut() = FileLoadingStatus::Finished;
        })));

        let onprogress_closure = Rc::new(RefCell::new(Closure::<dyn FnMut(_)>::new(move |event: ProgressEvent| {
            *progress_clone.borrow_mut() = (event.loaded() / event.total()) as f32;
        })));

        let window = web_sys::window().unwrap();
        let storage = window.local_storage().unwrap().unwrap();

        Self { input, status, buffer, progress, storage, request, onload_closure, onprogress_closure }
    }

    pub fn read(&mut self, path: &str) -> FileLoadingStatus {
        if matches!(*self.status.borrow(), FileLoadingStatus::Finished | FileLoadingStatus::Error) && *self.input.borrow() != path {
            *self.status.borrow_mut() = FileLoadingStatus::Idle;
        }

        if *self.status.borrow() == FileLoadingStatus::Idle {
            info!("Reading from file {}", path);
        }

        let status = *self.status.borrow();
        if let FileLoadingStatus::Idle = status {
            let request = XmlHttpRequest::new().unwrap();
            let onload_closure_clone = self.onload_closure.clone();
            let onprogress_closure_clone = self.onprogress_closure.clone();

            request.open_with_async("GET", path, true).unwrap();
            request.set_onload(Some(onload_closure_clone.borrow().as_ref().unchecked_ref()));
            request.set_onprogress(Some(onprogress_closure_clone.borrow().as_ref().unchecked_ref()));
            request.set_response_type(XmlHttpRequestResponseType::Arraybuffer);
            #[cfg(debug_assertions)]
            request.set_request_header("Cache-Control", "no-cache").unwrap();
            request.send().unwrap();

            *self.input.borrow_mut() = path.to_string();
            *self.status.borrow_mut() = FileLoadingStatus::Loading;
            *self.progress.borrow_mut() = 0.0;
            *self.request.borrow_mut() = Some(request);
        }

        *self.status.borrow()
    }

    pub fn write(&self, _path: &str, _content: &str) {
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
