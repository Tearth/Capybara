#[derive(Default)]
pub struct Console {
    pub input_content: String,
    pub output_content: String,

    changed: bool,
}

impl Console {
    pub fn test(&mut self) {
        if !self.output_content.is_empty() {
            self.output_content += "\n";
        }

        self.output_content += &self.input_content.trim();
        self.input_content.clear();
        self.changed = true;
    }

    pub fn is_changed(&mut self) -> bool {
        let original_value = self.changed;
        self.changed = false;

        original_value
    }
}
