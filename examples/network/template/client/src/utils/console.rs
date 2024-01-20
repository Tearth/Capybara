use std::collections::VecDeque;

#[derive(Default)]
pub struct Console {
    pub input_content: String,
    pub output_content: String,
    pub commands: VecDeque<String>,

    changed: bool,
}

impl Console {
    pub fn apply_input(&mut self) {
        if !self.output_content.is_empty() {
            self.output_content += "\n";
        }

        let command = self.input_content.trim();
        self.commands.push_back(command.to_string());

        self.output_content += ">>> ";
        self.output_content += &command;
        self.input_content.clear();
        self.changed = true;
    }

    pub fn apply_output(&mut self, content: &str) {
        self.output_content += "\n";
        self.output_content += content.trim();
    }

    pub fn is_changed(&mut self) -> bool {
        let original_value = self.changed;
        self.changed = false;

        original_value
    }

    pub fn poll_command(&mut self) -> Option<String> {
        self.commands.pop_front()
    }
}
