use super::*;
use anyhow::Result;
use std::cell::RefCell;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;

pub struct FileSystem {
    pub input: Rc<RefCell<String>>,
    pub status: Rc<RefCell<FileLoadingStatus>>,
    pub buffer: Rc<RefCell<Vec<u8>>>,
}

impl FileSystem {
    pub fn new() -> Self {
        let input = Rc::new(RefCell::new(String::new()));
        let status = Rc::new(RefCell::new(FileLoadingStatus::Idle));
        let buffer = Rc::new(RefCell::new(Vec::new()));

        Self { input, status, buffer }
    }

    pub fn read(&mut self, path: &str) -> Result<FileLoadingStatus> {
        let mut buffer = self.buffer.borrow_mut();
        let mut file = File::open(path)?;

        buffer.clear();
        file.read_to_end(&mut buffer)?;

        *self.input.borrow_mut() = path.to_string();
        *self.status.borrow_mut() = FileLoadingStatus::Finished;

        Ok(*self.status.borrow())
    }

    pub fn write(&self, path: &str, content: &str) -> Result<()> {
        Ok(fs::write(path, content)?)
    }

    pub fn read_local(&mut self, path: &str) -> Result<String> {
        Ok(fs::read_to_string(path)?)
    }

    pub fn write_local(&self, path: &str, content: &str) -> Result<()> {
        Ok(fs::write(path, content)?)
    }
}

impl Default for FileSystem {
    fn default() -> Self {
        Self::new()
    }
}
