use syntect::{parsing::SyntaxReference, highlighting::Style};
use ropey::RopeSlice;

use crate::state::Config;

use std::ops::Range;

pub struct HighlightContainer {
    syntax: SyntaxReference,
    lines: Vec<Vec<(Style, Range<usize>)>>,
}

impl HighlightContainer {
    pub fn new(syntax: SyntaxReference) -> Self {
        HighlightContainer {
            syntax,
            lines: Vec::new(),
        }
    }

    /// Highlights whole file
    pub fn highlight(&mut self, text: &RopeSlice, config: &Config) {
        self.lines.clear();
        use syntect::easy::HighlightLines;
        let mut h = HighlightLines::new(&self.syntax, &config.theme);
        for line_slice in text.lines() {
            let line: std::borrow::Cow<str> = line_slice.into();
            let chunks = h.highlight(&line, &config.syntax_set);
            let mut cursor = 0;
            self.lines.push(
                chunks
                    .into_iter()
                    .map(|(style, val)| {
                        // TODO: use an iter directly instead of a temp vec
                        let rng = cursor..cursor + val.len();
                        cursor = rng.end;
                        (style, rng)
                    })
                    .collect(),
            )
        }
    }

    /// Renders what is already highlighted
    pub fn render(&self, text: &RopeSlice, ui: &imgui::Ui) {
        for (chunks, line_slice) in self.lines.iter().zip(text.lines()) {
            let line: std::borrow::Cow<str> = line_slice.into();
            if ui.is_cursor_rect_visible([10., 10.]) {
                ui.group(|| {
                    for (style, rng) in chunks {
                        let val = &line[rng.clone()];
                        let syntect::highlighting::Color { r, g, b, a } = style.foreground;
                        ui.text_colored(
                            [
                                r as f32 / 255.,
                                g as f32 / 255.,
                                b as f32 / 255.,
                                a as f32 / 255.,
                            ],
                            val,
                        );
                        ui.same_line(0.);
                        let [cursor_x, cursor_y] = ui.cursor_pos();
                        ui.set_cursor_pos([cursor_x - 0.25, cursor_y]); // HACK: I can't figure out how to stop the stupid spacing
                    }
                });
            } else {
                ui.new_line();
            }
        }
    }
}
