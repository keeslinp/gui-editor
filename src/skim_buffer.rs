use crate::{
    buffer::get_visible_lines,
    error::Result,
    mode::Mode,
    msg::{Cmd, Direction, Msg},
    render::RenderFrame,
    text_buffer::TextBuffer,
};

use winit::event_loop::EventLoopProxy;

use wgpu_glyph::{Scale, Section};

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
    pub fn render(&self, render_frame: &mut RenderFrame, window_size: PhysicalSize) {
        let lines = get_visible_lines(window_size);
        if let Some(ref files) = self.sorted_files {
            for (index, entry) in files.iter().take(lines).enumerate() {
                let y_pos = window_size.height as f32 - (50. + index as f32 * 25.);
                if self.selected_option == index {
                    render_frame.queue_quad(0., y_pos, window_size.width as f32, 30.);
                }
                render_frame.queue_text(Section {
                    text: &entry.1,
                    screen_position: (30., y_pos),
                    color: [0.514, 0.58, 0.588, 1.],
                    scale: Scale { x: 30., y: 30. },
                    ..Section::default()
                });
            }
        }
        self.buffer.render(render_frame, window_size);
    }
}
