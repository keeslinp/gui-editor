use crate::{
    buffer::get_visible_lines,
    error::Result,
    mode::Mode,
    msg::{Cmd, Msg},
    render::RenderFrame,
    text_buffer::TextBuffer,
};

use winit::event_loop::EventLoopProxy;

use wgpu_glyph::{Scale, Section};

use fuzzy_matcher::skim::fuzzy_match;
use walkdir::{WalkDir};
use winit::dpi::PhysicalSize;

#[derive(Default)]
pub struct SkimBuffer {
    buffer: TextBuffer,
    selected_option: usize,
    files: Option<Vec<std::path::PathBuf>>,
}

impl SkimBuffer {
    pub fn refresh_files(&mut self) {
        let mut files = Vec::new();
        for file in WalkDir::new("./")
            .into_iter()
            .filter_map(|entry| {
                entry
                    .map(|entry| entry.into_path())
                    .ok()
            })
        {
            files.push(file);
        }
        self.files = Some(files);
    }
    pub fn handle_command(&mut self, cmd: Cmd, msg_sender: EventLoopProxy<Msg>) -> Result<bool> {
        let should_render = match cmd {
            Cmd::InsertChar('\n', _) => {
                if let Some(ref files) = self.files {
                    msg_sender
                        .send_event(Msg::Cmd(Cmd::LoadFile(
                            files[files.len() - self.selected_option - 1]
                                .clone(),
                        )))
                        .expect("sending load file command");
                    msg_sender
                        .send_event(Msg::Cmd(Cmd::ChangeMode(Mode::Normal)))
                        .expect("changing mode");
                    self.buffer.clear();
                    true
                } else {
                    false
                }
            }
            cmd => self.buffer.handle_command(cmd)?,
        };
        if self.files.is_none() {
            self.refresh_files();
        }
        Ok(should_render)
    }
    pub fn render(&self, render_frame: &mut RenderFrame, window_size: PhysicalSize) {
        let lines = get_visible_lines(window_size);
        if let Some(ref files) = self.files {
            use itertools::Itertools;
            for (index, entry) in files.iter().filter_map(|buf| {
                    buf.to_str().and_then(|str_path| {
                        fuzzy_match(str_path, self.buffer.as_str()).map(|score| (score, str_path))
                    })
                }).sorted().take(lines).enumerate() {
                render_frame.queue_text(Section {
                    text: entry.1,
                    screen_position: (30., 10. + index as f32 * 25.),
                    color: [0.514, 0.58, 0.588, 1.],
                    scale: Scale { x: 30., y: 30. },
                    ..Section::default()
                });
            }
        }
        self.buffer.render(render_frame, window_size);
        // println!("{:?}", self.files);
    }
}
