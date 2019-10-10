use slotmap::{SlotMap, SecondaryMap};
use crate::{
    buffer::{Buffer, BufferKey},
};

pub struct State {
    pub buffer_keys: SlotMap<BufferKey, ()>,
    pub buffers: SecondaryMap<BufferKey, Buffer>,
    pub current_buffer: BufferKey,
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
        }
    }
}
