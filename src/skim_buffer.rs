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
use walkdir::{DirEntry, WalkDir};
use winit::dpi::PhysicalSize;

#[derive(Default)]
pub struct SkimBuffer {
    buffer: TextBuffer,
    selected_option: usize,
    files: Vec<(i64, std::path::PathBuf)>,
}

impl SkimBuffer {
    pub fn handle_command(&mut self, cmd: Cmd, msg_sender: EventLoopProxy<Msg>) -> Result<bool> {
        let should_render = match cmd {
            Cmd::InsertChar('\n', _) => {
                msg_sender
                    .send_event(Msg::Cmd(Cmd::LoadFile(
                        self.files[self.files.len() - self.selected_option - 1]
                            .1
                            .clone(),
                    )))
                    .expect("sending load file command");
                msg_sender
                    .send_event(Msg::Cmd(Cmd::ChangeMode(Mode::Normal)))
                    .expect("changing mode");
                self.buffer.clear();
                true
            }
            cmd => self.buffer.handle_command(cmd)?,
        };
        let mut files = Vec::new();
        // TODO: SOOOOOOO SLOOOOOOWWWWWWWWWWWW
        // Seriously though, this is ratchet and just a first pass
        // Need to cache some values and just make this better
        for file in WalkDir::new("./")
            .into_iter()
            .filter_map(|entry| {
                entry
                    .map(|entry| entry.into_path())
                    .ok()
                    .and_then(|path_buff| {
                        path_buff
                            .to_str()
                            .map(|str_ref| str_ref.to_owned())
                            .map(|path_str| {
                                (fuzzy_match(&path_str, self.buffer.as_str()), path_buff)
                            })
                    })
            })
            .filter_map(|(maybe_score, path_buff)| maybe_score.map(|score| (score, path_buff)))
        {
            files.push(file);
        }
        files.sort_unstable();
        self.files = files;
        Ok(should_render)
    }
    pub fn render(&self, render_frame: &mut RenderFrame, window_size: PhysicalSize) {
        let lines = get_visible_lines(window_size);
        for (index, (_, path_buff)) in self.files.iter().take(lines).enumerate() {
            if let Some(text) = path_buff.to_str() {
                render_frame.queue_text(Section {
                    text,
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
