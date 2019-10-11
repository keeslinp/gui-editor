use crate::{
    buffer::{Buffer, BufferKey},
    command::CommandBuffer,
    mode::Mode,
};
use slotmap::{SecondaryMap, SlotMap};

pub struct State {
    pub buffer_keys: SlotMap<BufferKey, ()>,
    pub buffers: SecondaryMap<BufferKey, Buffer>,
    pub current_buffer: BufferKey,
    pub mode: Mode,
    pub command_buffer: CommandBuffer,
}

impl State {
    pub fn new() -> State {
        let mut buffer_keys = SlotMap::new();
        let current_buffer = buffer_keys.insert(());
        let mut buffers = SecondaryMap::new();
        buffers.insert(current_buffer, Buffer::new());
        State {
            buffers,
            buffer_keys,
            current_buffer,
            mode: Mode::Normal,
            command_buffer: CommandBuffer::default(),
        }
    }
}
