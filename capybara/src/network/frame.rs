#[derive(Debug, Clone)]
pub enum Frame {
    Text { text: String },
    Binary { data: Vec<u8> },
}

impl Frame {
    pub fn new_text(text: String) -> Self {
        Frame::Text { text }
    }

    pub fn new_binary(data: Vec<u8>) -> Self {
        Frame::Binary { data }
    }

    pub fn into_data(self) -> Vec<u8> {
        Vec::new()
    }
}
