use super::syntax::{Context, ContextElement, Match, MatchAction, Scope, Syntax, MatchValue};
use crate::{color_scheme::ColorScheme, point::Point, render::RenderFrame};
use anyhow::Result;
use core::ops::Range;
use ropey::RopeSlice;
use std::collections::HashMap;
use std::rc::Rc;
use wgpu_glyph::{Scale, Section};
use std::borrow::Cow;

#[derive(Debug, Clone)]
struct Node {
    context_stack: Vec<StackValue>,
    scope: Option<Scope>,
    char_range: Range<usize>,
    prev: Option<Rc<Node>>,
}

#[derive(Debug)]
struct ScopeMatch {
    scope: Option<Scope>,
    char_range: Range<usize>,
}

struct ContextMatchIter<'a> {
    stack: Vec<&'a [ContextElement]>,
    contexts: &'a HashMap<String, Context>,
}

#[derive(Debug, Clone)]
enum StackValue {
    Name(String),
    Anon(usize),
}

impl<'a> ContextMatchIter<'a> {
    fn new(
        root_context: &'a Context,
        contexts: &'a HashMap<String, Context>,
    ) -> ContextMatchIter<'a> {
        ContextMatchIter {
            stack: vec![root_context.elements.as_slice()],
            contexts,
        }
    }
}

impl<'a> Iterator for ContextMatchIter<'a> {
    type Item = &'a Match;
    fn next(&mut self) -> Option<Self::Item> {
        if self.stack.len() == 0 {
            None
        } else if self.stack[self.stack.len() - 1].len() == 0 {
            self.stack.pop();
            self.next()
        } else {
            let last_el = self.stack.len() - 1;
            if let Some((element, rest)) = self.stack[last_el].split_first() {
                self.stack[last_el] = rest;
                match element {
                    ContextElement::Match(ref m) => Some(m),
                    ContextElement::Include(i) => {
                        if let Some(ctx) = self.contexts.get(i) {
                            self.stack.push(ctx.elements.as_slice());
                            self.next()
                        } else {
                            eprintln!("Bad include: {}", i);
                            self.next()
                        }
                    }
                }
            } else {
                self.next()
            }
        }
    }
}

#[derive(Debug)]
struct PotentialMatch<'a> {
    range: Range<usize>,
    matches: Vec<ScopeMatch>,
    m: &'a Match,
}

use flamer::flame;
#[flame]
fn consume_next_match<'a>(
    slice: RopeSlice,
    context: &'a Context,
    char_offset: usize,
    contexts: &'a HashMap<String, Context>,
) -> (Option<&'a MatchAction>, Vec<ScopeMatch>) {
    if let Some(line_slice) = slice.lines().next() {
        let line: Cow<str> = line_slice.into();
        let mut first_match: Option<PotentialMatch> = None;
        for m in ContextMatchIter::new(context, contexts) {
            if let Ok(Some(captures)) = m.regex.captures(&line) {
                let total_range = captures
                    .get(0)
                    .map(|c| c.start()..c.end())
                    .unwrap_or(char_offset..char_offset);
                let backup_scope = m.scope.clone().or(context.meta_scope.clone());
                let mut next_match = Vec::with_capacity(captures.len() + 1);
                if let Some(ref captured_scopes) = m.captures {
                    for (scope, capture) in captured_scopes.iter().zip(captures.iter().skip(1)) {
                        if let Some(capture) = capture {
                            next_match.push(ScopeMatch {
                                scope: Some(scope.clone()),
                                char_range: (char_offset + capture.start())
                                    ..(char_offset + capture.end()),
                            });
                        }
                    }
                }
                // Fill in the gaps
                let filled_match = if next_match.len() > 0 {
                    let mut buffer = Vec::with_capacity(next_match.len() + 1);
                    let mut cursor = char_offset; //captures.get(0).map(|c| c.start()).unwrap_or(char_offset);
                    for group in next_match.into_iter() {
                        if group.char_range.start > cursor {
                            buffer.push(ScopeMatch {
                                scope: backup_scope.clone(),
                                char_range: cursor..group.char_range.start,
                            });
                        }
                        cursor = group.char_range.start;
                        buffer.push(group);
                    }
                    buffer
                } else if let Some(whole) = captures.get(0) {
                    vec![ScopeMatch {
                        scope: backup_scope,
                        char_range: (char_offset + whole.start())..(char_offset + whole.end()),
                    }]
                } else {
                    unreachable!(); // I hope?
                };
                if let Some(PotentialMatch {
                    ref range,
                    ..
                }) = first_match
                {
                    if range.start > total_range.start {
                        first_match = Some(PotentialMatch {
                            m: &m,
                            matches: filled_match,
                            range: total_range,
                        });
                    } else if range.start == total_range.start
                        && range.end < total_range.end
                    {
                        first_match = Some(PotentialMatch {
                            m: &m,
                            matches: filled_match,
                            range: total_range,
                        });
                    }
                } else {
                    first_match = Some(PotentialMatch {
                        m: &m,
                        matches: filled_match,
                        range: total_range,
                    });
                }
            }
        }

        // Fill in characters we skipped over with the default scope
        if let Some(PotentialMatch { m, mut matches, .. }) = first_match {
            if matches[0].char_range.start > 0 {
                matches.insert(
                    0,
                    ScopeMatch {
                        scope: context.meta_scope.clone(),
                        char_range: char_offset..(matches[0].char_range.start),
                    },
                );
            }
            (m.action.as_ref(), matches)
        } else {
            (
                None,
                vec![ScopeMatch {
                    scope: None,
                    char_range: char_offset..(char_offset + line.len()),
                }],
            )
        }
    } else {
        (None, Vec::new())
    }
}

impl Node {
    fn from_scope_match(scope_match: ScopeMatch, stack: &[StackValue], prev: Option<Rc<Node>>) -> Node {
        Node {
            prev,
            context_stack: stack.to_vec(),
            scope: scope_match.scope,
            char_range: scope_match.char_range,
        }
    }
}

pub struct Highlighter {
    tail: Option<Rc<Node>>,
    syntax: &'static Syntax,
}

fn add_value_to_stack(stack: &mut Vec<StackValue>, value: &MatchValue) {
    match value {
        MatchValue::Name(name) => {
            stack.push(StackValue::Name(name.clone()));
        },
        MatchValue::Inline(index) => {
            stack.push(StackValue::Anon(*index));
        },
        MatchValue::List(values) => {
            for value in values {
                add_value_to_stack(stack, value);
            }
        }
    }
}

impl Highlighter {
    pub fn new(file_extension: &str) -> Result<Self> {
        let syntax: &'static Syntax = Syntax::new(file_extension)?;
        &syntax.contexts.get("main");
        Ok(Highlighter {
            syntax: syntax,
            tail: None,
        })
    }

    pub fn mark_dirty(&mut self, index: usize) {
        loop {
            if let Some(node) = self.tail.as_ref() {
                if node.char_range.end < index {
                    break;
                }
                self.tail = node.prev.clone();
            } else {
                break;
            }
        }
    }


    #[flame("highlighter")]
    pub fn parse(&mut self, slice: RopeSlice) {
        // Only continues from previous tail's end point
        // Need to mark dirty first if there are changes
        let mut stack = self.tail.as_ref().map(|n| n.context_stack.clone()).unwrap_or_else(|| vec![StackValue::Name("main".to_owned())]);
        let mut end_cursor = self.tail.as_ref().map(|n| n.char_range.end).unwrap_or(0);
        let contexts = &self.syntax.contexts;
        let anon_contexts = self.syntax.anon_contexts.as_slice();
        loop {
            flame::start("node::loop::iter");
            let current_context = match stack[stack.len() - 1] {
                StackValue::Name(ref name) => &contexts[name.as_str()],
                StackValue::Anon(ref index) => &anon_contexts[*index],
            };
            if end_cursor >= slice.len_chars() {
                flame::end("node::loop::iter");
                break;
            }
            dbg!(&current_context);
            if stack.len() > 50 {
                panic!("Stack is overflowing");
            }
            let (action, scope_matches) = consume_next_match(
                slice.slice(end_cursor..),
                current_context,
                end_cursor,
                contexts,
            );
            if scope_matches.len() > 0 {
                end_cursor = scope_matches[0].char_range.end;
                for scope_match in scope_matches {
                    if scope_match.char_range.end > end_cursor {
                        end_cursor = scope_match.char_range.end
                    }
                    self.tail = Some(Rc::new(
                        Node::from_scope_match(scope_match, stack.as_slice(), self.tail.clone()),
                    ));
                }
            } else {
                flame::end("node::loop::iter");
                break;
            }
            match action {
                Some(MatchAction::Push(match_value)) => {
                    add_value_to_stack(&mut stack, match_value);
                }
                // Some(MatchAction::PushList(entries)) => {
                //     for entry in entries {
                //         stack.push(entry.clone());
                //     }
                // }
                Some(MatchAction::Pop) => {
                    stack.pop();
                }
                Some(MatchAction::Set(match_value)) => {
                    stack.pop();
                    add_value_to_stack(&mut stack, match_value);
                }
                None => {}
            }
            flame::end("node::loop::iter");
        }
    }

    #[flame("highlighter")]
    pub fn render(
        &self,
        render_frame: &mut RenderFrame,
        char_range: Range<usize>,
        slice: RopeSlice,
        start_x: f32,
        y_offset: f32,
        color_scheme: &ColorScheme,
    ) {
        let mut current_node = self.tail.as_ref();
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
            let text: Cow<str> = slice.slice(node.char_range.clone()).into();
            render_frame.queue_text(Section {
                text: &text,
                screen_position: (
                    start_x + (point.x as f32 * 15.),
                    (point.y as f32 * 25.) - y_offset,
                ),
                color: node
                    .scope
                    .as_ref()
                    .and_then(|scope| color_scheme.get_fg_color_for_scope(scope))
                    .unwrap_or([1., 1., 1., 1.]),
                scale: Scale { x: 30., y: 30. },
                ..Section::default()
            });
            current_node = node.prev.as_ref();
        }
    }
}
