use super::syntax::{Scope, Syntax};
use crate::error::{Result};
use std::rc::Rc;

struct Node {
    prev: Rc<Node>,
    child_tail: Rc<Node>,
    context: String,
    scope: Scope,
}

pub struct Highlighter {
    // tail: Rc<Node>
    syntax: Syntax,
}

const RUST_SYNTAX: &'static str = include_str!("./Rust.sublime-syntax");

impl Highlighter {
    pub fn new() -> Result<Self> {
        let syntax: Syntax = Syntax::new(serde_yaml::from_str(RUST_SYNTAX)?)?;
        Ok(Highlighter {
            syntax: dbg!(syntax),
        })
    }
}
