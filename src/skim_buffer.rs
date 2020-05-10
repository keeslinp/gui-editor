use crate::{
    buffer::get_visible_lines,
    mode::Mode,
    msg::{Cmd, Direction, Msg},
    text_buffer::TextBuffer,
};

use anyhow::Result;

use winit::event_loop::EventLoopProxy;

use fuzzy_matcher::skim::fuzzy_match;
use walkdir::WalkDir;
use winit::dpi::PhysicalSize;

#[derive(Default)]
pub struct SkimBuffer {
    buffer: TextBuffer,
    selected_option: usize,
    files: Option<Vec<std::path::PathBuf>>,
    sorted_files: Option<Vec<(usize, String)>>,
}

impl SkimBuffer {
    pub fn refresh_files(&mut self) {
        let mut files = Vec::new();
        for file in WalkDir::new("./").into_iter().filter_map(|entry| {
            entry
                .map(|entry| entry.into_path())
                .ok()
                .and_then(|path_buf| {
                    if path_buf.is_file() {
                        Some(path_buf)
                    } else {
                        None
                    }
                })
        }) {
            files.push(file);
        }
        self.files = Some(files);
        self.sorted_files = None;
    }
    pub fn handle_command(&mut self, cmd: Cmd, msg_sender: EventLoopProxy<Msg>) -> Result<bool> {
        let should_render = match cmd {
            Cmd::Submit => {
                if let Some(ref files) = self.files {
                    if let Some(real_index) = self
                        .sorted_files
                        .as_ref()
                        .map(|files| files[self.selected_option].0)
                    {
                        msg_sender
                            .send_event(Msg::Cmd(Cmd::LoadFile(files[real_index].clone())))
                            .expect("sending load file command");
                    }
                    msg_sender
                        .send_event(Msg::Cmd(Cmd::ChangeMode(Mode::Normal)))
                        .expect("changing mode");
                    self.buffer.clear();
                    true
                } else {
                    false
                }
            }
            Cmd::MoveCursor(Direction::Up) => {
                self.selected_option += 1;
                if self.selected_option > self.sorted_files.as_ref().map(|v| v.len()).unwrap_or(0) {
                    self.selected_option = 0;
                }
                true
            }
            Cmd::MoveCursor(Direction::Down) => {
                if self.selected_option > 0 {
                    self.selected_option -= 1;
                }
                false
            }
            cmd => {
                let updated = self.buffer.handle_command(cmd)?;
                if updated {
                    self.update_filtered_entries();
                }
                updated
            }
        };
        if self.files.is_none() {
            self.refresh_files();
        }
        Ok(should_render)
    }
    fn update_filtered_entries(&mut self) {
        use itertools::Itertools;
        if let Some(ref files) = self.files {
            self.sorted_files = Some(
                files
                    .iter()
                    .enumerate()
                    .filter_map(|(index, buf)| {
                        buf.to_str().and_then(|str_path| {
                            fuzzy_match(str_path, self.buffer.as_str())
                                .map(|score| (index, score, str_path))
                        })
                    })
                    .sorted()
                    .map(|(index, _, text)| (index, text.to_owned()))
                    .rev()
                    .collect(),
            );
        }
        self.selected_option = 0;
    }
    // TODO: render in window overtop
    pub fn render(&self, ui: &imgui::Ui) {
        if let Some(ref files) = self.sorted_files {
            let [width, _height] = ui.window_content_region_max();
            let lines = get_visible_lines(ui);
            for (index, entry) in files.iter().take(lines).enumerate().rev() {
                if self.selected_option == index {
                    let [_cx, cy] = ui.cursor_pos();
                    ui.get_window_draw_list()
                        .add_rect([0., cy], [width, cy + 20.], [1., 1., 1., 0.2])
                        .filled(true)
                        .build();
                }
                let im_string = imgui::ImString::new(&entry.1);
                ui.text(im_string);
            }
        }
    }
    pub fn render_bar(&self, ui: &imgui::Ui) {
        self.buffer.render(ui);
    }
}
