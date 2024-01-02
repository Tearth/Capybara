use super::*;
use anyhow::Result;
use log::error;
use log::info;
use std::cell::RefCell;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;

pub struct FileSystem {
    pub input: Rc<RefCell<String>>,
    pub status: Rc<RefCell<FileLoadingStatus>>,
    pub buffer: Rc<RefCell<Vec<u8>>>,
    pub progress: Rc<RefCell<f32>>,
}

impl FileSystem {
    pub fn new() -> Self {
        let input = Rc::new(RefCell::new(String::new()));
        let status = Rc::new(RefCell::new(FileLoadingStatus::Idle));
        let buffer = Rc::new(RefCell::new(Vec::new()));
        let progress = Rc::new(RefCell::new(0.0));

        Self { input, status, buffer, progress }
    }

    pub fn read(&mut self, path: &str) -> FileLoadingStatus {
        let mut input = self.input.borrow_mut();
        let mut buffer = self.buffer.borrow_mut();
        let mut status = self.status.borrow_mut();
        let mut progress = self.progress.borrow_mut();

        if matches!(*status, FileLoadingStatus::Finished | FileLoadingStatus::Error) && *input != path {
            *status = FileLoadingStatus::Idle;
            *progress = 0.0;
        }

        if *status == FileLoadingStatus::Idle {
            info!("Reading from file {}", path);

            let mut file = match File::open(path) {
                Ok(file) => file,
                Err(err) => {
                    error!("Failed to open file ({})", err);
                    *status = FileLoadingStatus::Error;
                    return *status;
                }
            };

            buffer.clear();

            if let Err(err) = file.read_to_end(&mut buffer) {
                error!("Failed to read file ({})", err);
                *status = FileLoadingStatus::Error;
                return *status;
            }

            *input = path.to_string();
            *status = FileLoadingStatus::Finished;
            *progress = 1.0;
        }

        *status
    }

    pub fn write(&self, path: &str, content: &str) {
        info!("Writing to file {} ({} bytes)", path, content.len());

        if let Err(err) = fs::write(path, content) {
            error!("Failed to write into {} ({})", path, err);
        }
    }

    pub fn read_local(&mut self, path: &str) -> Result<String> {
        info!("Reading from local file {}", path);
        Ok(fs::read_to_string(path)?)
    }

    pub fn write_local(&self, path: &str, content: &str) {
        info!("Writing to local file {} ({} bytes)", path, content.len());

        if let Err(err) = fs::write(path, content) {
            error!("Failed to write into {} ({})", path, err);
        }
    }
}

impl Default for FileSystem {
    fn default() -> Self {
        Self::new()
    }
}
