#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Mode {
    Normal,
    Insert,
    Command,
    Jump,
    Skim,
    // Select,
}

impl Mode {
    fn as_str(self) -> &'static str {
        match self {
            Mode::Normal => "Normal",
            Mode::Insert => "Insert",
            Mode::Command => "Command",
            Mode::Jump => "Jump",
            Mode::Skim => "Skim",
        }
    }
    pub fn render(self, ui: &imgui::Ui) {
        let value = self.as_str();
        let [width, height] = ui.window_size();
        let im_string = imgui::ImString::new(value);
        let [text_width, text_height] = ui.calc_text_size(&im_string, false, width);
        ui.set_cursor_pos([width - text_width - 15., 0.]);
        ui.text(im_string);
    }
}
