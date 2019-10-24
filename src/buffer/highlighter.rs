use crate::error::{Error, Result};
use fancy_regex::Regex as FRegex;
use regex::Regex;

#[derive(Debug)]
struct Scope {
    name: String,
}

impl From<String> for Scope {
    fn from(val: String) -> Scope {
        Scope { name: val }
    }
}

impl core::ops::Deref for Scope {
    type Target = String;
    fn deref(&self) -> &String {
        &self.name
    }
}

#[derive(Debug)]
enum MatchAction {
    Push(String),
    PushInLine(Context),
    Pop,
    Set(String),
}

#[derive(Debug)]
struct Match {
    regex: FRegex,
    action: MatchAction,
    scope: Option<Scope>,
    captures: Option<Vec<Scope>>,
}

impl From<fancy_regex::Error> for Error {
    fn from(_err: fancy_regex::Error) -> Error {
        Error::BuildingSyntax
    }
}

use lazy_static::lazy_static;

impl Match {
    fn new(map: &serde_yaml::Mapping, variables: &HashMap<String, String>) -> Result<Match> {
        lazy_static! {
            static ref VAR_RE: Regex = Regex::new(r"\{\{(.*)\}\}").unwrap();
        }
        let regex: FRegex = map
            .get(&serde_yaml::Value::String("match".to_string()))
            .and_then(|v| v.as_str())
            .map(|s| {
                VAR_RE.replace_all(s, |captures: &regex::Captures| {
                    captures
                        .get(1)
                        .map(|c| c.as_str())
                        .and_then(|s| variables.get(s))
                        .expect("failed to replace variable")
                })
            })
            .ok_or(Error::BuildingSyntax)
            .and_then(|s| FRegex::new(s.as_ref()).map_err(|e| e.into()))?;
        let captures: Option<Vec<Scope>> = map
            .get(&serde_yaml::Value::String("captures".to_string()))
            .and_then(|v| v.as_mapping())
            .map(|seq| {
                seq.iter()
                    .flat_map(|(_k, v)| v.as_str())
                    .map(|s| s.to_string().into())
                    .collect()
            });
        let scope: Option<Scope> = map
            .get(&serde_yaml::Value::String("scope".to_string()))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string().into());
        let action: MatchAction =
            if let Some(push) = map.get(&serde_yaml::Value::String("push".to_string())) {
                if let Some(push_str) = push.as_str() {
                    MatchAction::Push(push_str.to_string())
                } else if let Some(push_context) = push.as_sequence() {
                    MatchAction::PushInLine(Context::new(push_context, variables)?)
                } else {
                    return Err(Error::BuildingSyntax);
                }
            } else if let Some(_pop) = map.get(&serde_yaml::Value::String("pop".to_string())) {
                MatchAction::Pop
            } else {
                return Err(Error::BuildingSyntax);
            };
        Ok(Match {
            regex,
            action,
            captures,
            scope,
        })
    }
}

#[derive(Debug)]
struct Context {
    matches: Vec<Match>,
    meta_scope: Option<String>,
}

impl Context {
    fn new(value: &serde_yaml::Sequence, variables: &HashMap<String, String>) -> Result<Context> {
        let matches = value
            .iter()
            .flat_map(|v| v.as_mapping())
            .flat_map(|m| Match::new(m, &variables))
            .collect();
        let meta_scope = value
            .iter()
            .flat_map(|v| v.get("meta_scope"))
            .flat_map(|meta_scope| meta_scope.as_str())
            .map(|s| s.to_string())
            .next();
        Ok(Context {
            matches,
            meta_scope,
        })
    }
}

use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
struct Syntax {
    contexts: HashMap<String, Context>,
    name: String,
    file_extensions: Vec<String>,
    scope: String,
    variables: HashMap<String, String>,
}

impl Syntax {
    pub fn new(values: serde_yaml::Value) -> Result<Syntax> {
        if values.is_mapping() {
            let name: String = values
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or(Error::BuildingSyntax)?
                .to_string();
            let scope: String = values
                .get("scope")
                .and_then(|v| v.as_str())
                .ok_or(Error::BuildingSyntax)?
                .to_string();
            let variables: HashMap<String, String> = values
                .get("variables")
                .and_then(|v| v.as_mapping())
                .ok_or(Error::BuildingSyntax)?
                .iter()
                .flat_map(|(k, v)| {
                    if let (Some(k), Some(v)) = (k.as_str(), v.as_str()) {
                        Some((k.to_string(), v.to_string()))
                    } else {
                        None
                    }
                })
                .collect();

            // let contexts: Option<HashMap<String, String>> = values.get("contexts").and_then(|v|.v.as_mapping()).map(
            let contexts: HashMap<String, Context> = values
                .get("contexts")
                .and_then(|v| v.as_mapping())
                .ok_or(Error::BuildingSyntax)?
                .iter()
                .flat_map(|(k, v)| {
                    if let (Some(k), Some(v)) = (k.as_str(), v.as_sequence()) {
                        Ok((k.to_string(), Context::new(v, &variables)?))
                    } else {
                        Err(Error::BuildingSyntax)
                    }
                })
                .collect::<HashMap<String, Context>>();

            let file_extensions: Vec<String> = values
                .get("file_extensions")
                .and_then(|v| v.as_sequence())
                .ok_or(Error::BuildingSyntax)?
                .iter()
                .flat_map(|extension| extension.as_str().map(|s| s.to_string()))
                .collect();
            Ok(Syntax {
                name,
                file_extensions,
                variables,
                contexts,
                scope,
            })
        } else {
            Err(Error::BuildingSyntax)
        }
    }
}

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
