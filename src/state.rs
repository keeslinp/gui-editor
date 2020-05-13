use crate::{
    buffer::{Buffer, BufferKey, get_visible_lines},
    command::CommandBuffer,
    mode::Mode,
    skim_buffer::SkimBuffer,
};

use syntect::{highlighting::Theme, parsing::SyntaxSet};

use anyhow::{anyhow, Result};
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
    pub line_count: usize,
}

const SYNTAXES: &[&str] = &[
    include_str!("./syntaxes/RustEnhanced.sublime-syntax"),
    include_str!("./syntaxes/TypeScript.sublime-syntax"),
    include_str!("./syntaxes/TypeScriptReact.sublime-syntax"),
];

fn build_syntax_set() -> Result<SyntaxSet> {
    use syntect::parsing::{syntax_definition::SyntaxDefinition, SyntaxSetBuilder};
    let mut set = SyntaxSetBuilder::new();
    for syntax in SYNTAXES {
        set.add(
            SyntaxDefinition::load_from_str(syntax, true, None)
                .map_err(|_| anyhow!("failed to load syntax"))?,
        );
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
            theme: syntect::highlighting::ThemeSet::load_defaults().themes["base16-ocean.dark"]
                .clone(),
            syntax_set: build_syntax_set()?,
            line_count: 0,
        })
    }

    pub fn update_from_ui(&mut self, ui: &imgui::Ui) {
        self.line_count = get_visible_lines(ui);
    }
}
