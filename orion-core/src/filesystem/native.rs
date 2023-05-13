use super::*;
use anyhow::Result;
use std::cell::RefCell;
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

    pub fn load(&mut self, input: &str) -> Result<FileLoadingStatus> {
        let mut buffer = self.buffer.borrow_mut();
        let mut file = File::open(input)?;

        buffer.clear();
        file.read_to_end(&mut buffer)?;

        *self.input.borrow_mut() = input.to_string();
        *self.status.borrow_mut() = FileLoadingStatus::Finished;

        Ok(*self.status.borrow())
    }
}

impl Default for FileSystem {
    fn default() -> Self {
        Self::new()
    }
}
