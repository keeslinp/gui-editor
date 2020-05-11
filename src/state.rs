use crate::{
    buffer::{Buffer, BufferKey},
    command::CommandBuffer,
    mode::Mode,
    skim_buffer::SkimBuffer,
};

use syntect::{highlighting::{ThemeSet, Theme}, parsing::SyntaxSet};

use anyhow::{Result, anyhow};
use slotmap::{SecondaryMap, SlotMap};

pub struct State {
    pub buffer_keys: SlotMap<BufferKey, ()>,
    pub buffers: SecondaryMap<BufferKey, Buffer>,
    pub current_buffer: BufferKey,
    pub mode: Mode,
    pub command_buffer: CommandBuffer,
    pub status: Option<String>,
    pub skim_buffer: SkimBuffer,
    pub syntax_set: SyntaxSet,
    pub theme: Theme,
}

const SYNTAXES: &[&str] = &[
    include_str!("./syntaxes/RustEnhanced.sublime-syntax"),
    include_str!("./syntaxes/TypeScript.sublime-syntax"),
    include_str!("./syntaxes/TypeScriptReact.sublime-syntax"),
];

fn build_syntax_set() -> Result<SyntaxSet> {
    use syntect::parsing::{SyntaxSetBuilder, syntax_definition::SyntaxDefinition};
    let mut set = SyntaxSetBuilder::new();
    for syntax in SYNTAXES {
        set.add(SyntaxDefinition::load_from_str(syntax, true, None).map_err(|_| anyhow!("failed to load syntax"))?);
    }
    Ok(set.build())
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
            theme: syntect::highlighting::ThemeSet::load_defaults().themes["base16-ocean.dark"].clone(),
            syntax_set: build_syntax_set()?,
        })
    }
}
