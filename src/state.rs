use crate::{
    buffer::{Buffer, BufferKey},
    command::CommandBuffer,
    error::{Error, Result},
    mode::Mode,
    skim_buffer::SkimBuffer,
};
use slotmap::{SecondaryMap, SlotMap};

pub struct State {
    pub buffer_keys: SlotMap<BufferKey, ()>,
    pub buffers: SecondaryMap<BufferKey, Buffer>,
    pub current_buffer: BufferKey,
    pub mode: Mode,
    pub command_buffer: CommandBuffer,
    pub error: Option<Error>,
    pub skim_buffer: SkimBuffer,
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
            error: None,
            skim_buffer: SkimBuffer::default(),
        })
    }
}
