use super::syntax::{Scope, Syntax, Context, MatchAction};
use crate::error::{Result, Error};
use std::rc::Rc;
use ropey::RopeSlice;
use std::collections::HashMap;
use core::ops::Range;

#[derive(Debug, Clone)]
struct Node {
    context_stack: Vec<String>,
    scope: Option<Scope>,
    char_range: Range<usize>,
    prev: Option<Rc<Node>>,
}

struct ScopeMatch {
    scope: Option<Scope>,
    char_range: Range<usize>,
}

fn consume_next_match(slice: RopeSlice, context: &Context, char_offset: usize, contexts: &HashMap<String, Context>) -> Vec<ScopeMatch> {
    if let Some(line) = slice.lines().next().and_then(|l| l.as_str()) {
        if (line.trim() == "") {
            return vec![ScopeMatch {
                scope: None,
                char_range: char_offset..(char_offset+ line.len()),
            }];
        }
        for m in context.matches.iter().chain(context.includes.iter().flat_map(|include| contexts.get(include).unwrap().matches.iter())) {
            if let Ok(Some(captures)) = m.regex.captures(line) {
                if let Some(c) = captures.get(0) {
                    let scope_match = ScopeMatch {
                        scope: m.scope.clone().or(context.meta_scope.clone()), // TODO: Figure out what the hell to do here
                        char_range: (char_offset + c.start()..(char_offset + c.end())),
                    };
                    return vec![scope_match];
                } else {
                    return Vec::new();
                }
            }
        }
        Vec::new()
    } else {
        Vec::new()
    }
}

impl Node {
    fn from_scope_match(scope_match: ScopeMatch, stack: &[String]) -> Node {
        Node {
            prev: None,
            context_stack: stack.to_vec(),
            scope: scope_match.scope,
            char_range: scope_match.char_range,
        }
    }
    fn build(prev: Option<Rc<Node>>, slice: RopeSlice, contexts: &HashMap<String, Context>) -> Option<Rc<Node>> {
        let mut last_node: Option<Rc<Node>> = None;
        let mut stack = vec!["main".to_owned()];
        loop {
            let current_context = &contexts[&stack[stack.len() - 1]];
            let offset = last_node.as_ref().map(|node| node.char_range.end + 1).unwrap_or(0);
            let scope_matches = consume_next_match(slice.slice(offset..), current_context, offset, contexts);
            if scope_matches.len() == 0 {
                break;
            }
            for scope_match in scope_matches {
                last_node = Some(Rc::new(Node::from_scope_match(scope_match, stack.as_slice()).set_prev(last_node)));
            }
        }
        last_node
    }

    fn set_prev(mut self, prev: Option<Rc<Node>>) -> Node {
        self.prev = prev;
        self
    }
}

pub struct Highlighter {
    tail: Option<Rc<Node>>,
    syntax: Syntax,
}

const RUST_SYNTAX: &'static str = include_str!("./Rust.sublime-syntax");

impl Highlighter {
    pub fn new() -> Result<Self> {
        let syntax: Syntax = Syntax::new(serde_yaml::from_str(RUST_SYNTAX)?)?;
        Ok(Highlighter {
            syntax: dbg!(syntax),
            tail: None,
        })
    }

    pub fn parse(&mut self, slice: RopeSlice) {
        dbg!(Node::build(None, slice, &self.syntax.contexts));
    }
}
