use super::syntax::{Scope, Syntax, Context, MatchAction, Match};
use crate::{
    error::{Result, Error},
    render::RenderFrame,
    point::Point,
};
use wgpu_glyph::{Scale, Section};
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

#[derive(Debug)]
struct ScopeMatch {
    scope: Option<Scope>,
    char_range: Range<usize>,
}

fn consume_next_match<'a>(slice: RopeSlice, context: &'a Context, char_offset: usize, contexts: &'a HashMap<String, Context>) -> (Option<&'a MatchAction>, Vec<ScopeMatch>) {
    if let Some(line) = slice.lines().next().and_then(|l| l.as_str()) {
        if (line.trim() == "") {
            return (None, vec![ScopeMatch {
                scope: None,
                char_range: char_offset..(char_offset + line.len()),
            }]);
        }
        let mut first_match: Option<(&Match, ScopeMatch)> = None;
        for m in context.matches.iter().chain(context.includes.iter().flat_map(|include| contexts.get(include).unwrap().matches.iter())) {
            if let Ok(Some(captures)) = m.regex.captures(line) {
                if let Some(c) = captures.get(0) {
                    let next_match = ScopeMatch {
                        scope: m.scope.clone().or(context.meta_scope.clone()), // TODO: Figure out what the hell to do here
                        char_range: (char_offset + c.start()..(char_offset + c.end())),
                    };
                    if let Some((_, ref scope_match)) = first_match {
                        if scope_match.char_range.start > next_match.char_range.start {
                            first_match = Some((&m, next_match));
                        }
                    } else {
                        first_match = Some((&m, next_match));
                    }
                }
            }
        }
        if let Some((m, scope_match)) = first_match {
            if scope_match.char_range.start > 0 {
                (m.action.as_ref(), vec![ScopeMatch {
                    scope: None,
                    char_range: char_offset..(scope_match.char_range.start),
                }, scope_match])
            } else {
                (m.action.as_ref(), vec![scope_match])
            }
        } else {
            (None, vec![ScopeMatch {
                scope: None,
                char_range: char_offset..(char_offset + line.len()),
            }])
        }
    } else {
        (None, Vec::new())
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
        let mut inline_ctx: Option<&Context> = None;
        loop {
            let current_context = inline_ctx.unwrap_or(&contexts[&stack[stack.len() - 1]]);
            let offset = last_node.as_ref().map(|node| node.char_range.end).unwrap_or(0);
            if offset >= slice.len_chars() {
                break;
            }
            let (action, scope_matches) = consume_next_match(slice.slice(offset..), current_context, offset, contexts);
            if scope_matches.len() == 0 {
                break;
            }
            for scope_match in scope_matches {
                last_node = Some(Rc::new(Node::from_scope_match(scope_match, stack.as_slice()).set_prev(last_node)));
            }
            match action {
                Some(MatchAction::Push(stack_entry)) => {
                    stack.push(stack_entry.clone());
                }
                Some(MatchAction::PushList(entries)) => {
                    for entry in entries {
                        stack.push(entry.clone());
                    }
                }
                Some(MatchAction::Pop) => {
                    if inline_ctx.is_some() {
                        inline_ctx = None;
                    } else {
                        stack.pop();
                    }
                },
                Some(MatchAction::PushInLine(ctx)) => {
                    inline_ctx = Some(ctx);
                }
                Some(MatchAction::Set(stack_entry)) => {
                    if inline_ctx.is_some() {
                        inline_ctx = None;
                    } else {
                        stack.pop();
                    }
                    stack.push(stack_entry.clone());
                }
                None => {},
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

fn get_color_for_scope(scope: &Option<Scope>) -> [f32;4] {
    match scope.as_ref().map(|scope| scope.name.as_str()) {
        Some("keyword.other.rust") => [0., 0., 1., 1.],
        Some("storage.type.module.rust") => [0., 1., 1., 1.],
        _ => [0.514, 0.58, 0.588, 1.],
    }
}

impl Highlighter {
    pub fn new() -> Result<Self> {
        let syntax: Syntax = Syntax::new(serde_yaml::from_str(RUST_SYNTAX)?)?;
        Ok(Highlighter {
            syntax: syntax,
            tail: None,
        })
    }

    pub fn parse(&mut self, slice: RopeSlice) {
        self.tail = Node::build(None, slice, &self.syntax.contexts);
    }

    pub fn render(&self, render_frame: &mut RenderFrame, char_range: Range<usize>, slice: RopeSlice, start_x: f32, y_offset: f32) {
        let mut current_node= self.tail.as_ref();
        loop {
            if let Some(node) = current_node {
                if node.char_range.start < char_range.end {
                    break;
                }
                current_node = node.prev.as_ref();
            } else {
                break;
            }
        }
        while let Some(node) = current_node {
            if node.char_range.end < char_range.start {
                break; // If we can't see it, stop
            }
            let point = Point::from_index(node.char_range.start, &slice);
            if let Some(text) = slice.slice(node.char_range.clone()).as_str() {
                render_frame.queue_text(Section {
                    text,
                    screen_position: (start_x + (point.x as f32 * 15.), (point.y as f32 * 25.) - y_offset),
                    color: get_color_for_scope(&node.scope),
                    scale: Scale { x: 30., y: 30. },
                    ..Section::default()
                });
            }
            current_node = node.prev.as_ref();
        }
    }
}
