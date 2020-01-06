use crate::{
    buffer::{Buffer, BufferKey},
    color_scheme::ColorScheme,
    command::CommandBuffer,
    mode::Mode,
    skim_buffer::SkimBuffer,
};

use anyhow::Result;
use slotmap::{SecondaryMap, SlotMap};

pub struct State {
    pub buffer_keys: SlotMap<BufferKey, ()>,
    pub buffers: SecondaryMap<BufferKey, Buffer>,
    pub current_buffer: BufferKey,
    pub mode: Mode,
    pub command_buffer: CommandBuffer,
    pub status: Option<String>,
    pub skim_buffer: SkimBuffer,
    pub color_scheme: ColorScheme,
}

impl State {
    pub fn new() -> Result<State> {
        let mut buffer_keys = SlotMap::new();
        let current_buffer = buffer_keys.insert(());
        let mut buffers = SecondaryMap::new();
        buffers.insert(current_buffer, Buffer::new()?);
        Ok(State {
            buffers,
            buffer_keys,
            current_buffer,
            mode: Mode::Normal,
            command_buffer: CommandBuffer::default(),
            status: None,
            skim_buffer: SkimBuffer::default(),
            color_scheme: ColorScheme::build()?,
        })
    }
}
