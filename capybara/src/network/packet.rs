#[derive(Debug, Clone)]
pub enum Packet {
    Text { text: String },
    Binary { data: Vec<u8> },
}

impl Packet {
    pub fn new_text(text: String) -> Self {
        Packet::Text { text }
    }

    pub fn new_binary(data: Vec<u8>) -> Self {
        Packet::Binary { data }
    }

    pub fn into_data(self) -> Vec<u8> {
        Vec::new()
    }
}
