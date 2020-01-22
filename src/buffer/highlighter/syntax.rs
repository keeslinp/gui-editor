use crate::error::Error;
use anyhow::Result;
use fancy_regex::Regex as FRegex;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct Scope {
    pub name: String,
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

#[derive(Debug, Clone)]
pub enum StackValue {
    Name(String),
    Anon(usize),
}

#[derive(Debug, Clone)]
pub enum MatchAction {
    Push(StackValue),
    PushList(Vec<StackValue>),
    Pop,
    Set(StackValue),
}

#[derive(Debug)]
pub struct Match {
    pub regex: FRegex,
    pub action: Option<MatchAction>,
    pub scope: Option<Scope>,
    pub captures: Option<Vec<Scope>>,
}

impl From<fancy_regex::Error> for Error {
    fn from(_err: fancy_regex::Error) -> Error {
        Error::BuildingSyntax
    }
}

use lazy_static::lazy_static;

impl Match {
    fn new(
        map: &serde_yaml::Mapping,
        variables: &HashMap<String, String>,
        anon_contexts: &mut Vec<Context>,
    ) -> Result<Match> {
        lazy_static! {
            static ref VAR_RE: Regex = Regex::new(r"\{\{([^\}]*)\}\}").unwrap();
        }
        let regex: FRegex = map
            .get(&serde_yaml::Value::String("match".to_string()))
            .and_then(|v| v.as_str())
            .map(|s| {
                let mut stable;
                let mut temp = s.to_owned();
                loop {
                    stable = true;
                    temp = VAR_RE
                        .replace_all(temp.as_str(), |captures: &regex::Captures| {
                            stable = false;
                            captures
                                .get(1)
                                .map(|c| c.as_str())
                                .and_then(|s| variables.get(s))
                                .expect("failed to replace variable")
                        })
                        .into_owned();
                    if stable {
                        break;
                    }
                }
                temp
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
        let action: Option<MatchAction> =
            if let Some(push) = map.get(&serde_yaml::Value::String("push".to_string())) {
                if let Some(push_str) = push.as_str() {
                    Some(MatchAction::Push(StackValue::Name(push_str.to_string())))
                } else if let Some(push_sequence) = push.as_sequence() {
                    if push_sequence[0].is_mapping() {
                        let anon = Context::new(push_sequence, variables, anon_contexts)?;
                        let index = anon_contexts.len();
                        anon_contexts.push(anon);
                        Some(MatchAction::Push(StackValue::Anon(index)))
                    } else {
                        Some(MatchAction::PushList(
                            push_sequence
                                .iter()
                                .flat_map(|v| v.as_str())
                                .map(|s| StackValue::Name(s.to_string()))
                                .collect(),
                        ))
                    }
                } else {
                    return Err(Error::BuildingSyntax.anyhow());
                }
            } else if let Some(_pop) = map.get(&serde_yaml::Value::String("pop".to_string())) {
                Some(MatchAction::Pop)
            } else if let Some(set) = map.get(&serde_yaml::Value::String("set".to_string())) {
                set.as_str()
                    .map(|s| MatchAction::Set(StackValue::Name(s.to_string())))
            } else {
                None
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
pub enum ContextElement {
    Include(String),
    Match(Match),
}

impl ContextElement {
    fn new(
        map: &serde_yaml::Mapping,
        variables: &HashMap<String, String>,
        anon_contexts: &mut Vec<Context>,
    ) -> Result<ContextElement> {
        if map.contains_key(&serde_yaml::Value::String("match".to_string())) {
            Match::new(map, variables, anon_contexts).map(|m| ContextElement::Match(m))
        } else if let Some(include) = map.get(&serde_yaml::Value::String("include".to_string())) {
            if let Some(val) = include.as_str() {
                Ok(ContextElement::Include(val.to_string()))
            } else {
                Err(Error::BuildingSyntax.anyhow())
            }
        } else {
            Err(Error::BuildingSyntax.anyhow())
        }
    }
}

#[derive(Debug)]
pub struct Context {
    pub elements: Vec<ContextElement>,
    pub meta_scope: Option<Scope>,
}

impl Context {
    fn new(
        value: &serde_yaml::Sequence,
        variables: &HashMap<String, String>,
        anon_contexts: &mut Vec<Context>,
    ) -> Result<Context> {
        let elements = value
            .iter()
            .flat_map(|v| v.as_mapping())
            .flat_map(|m| ContextElement::new(m, &variables, anon_contexts))
            .collect();
        let meta_scope = value
            .iter()
            .flat_map(|v| v.get("meta_scope"))
            .flat_map(|meta_scope| meta_scope.as_str())
            .map(|s| s.to_string().into())
            .next();
        Ok(Context {
            elements,
            meta_scope,
        })
    }
}

use std::collections::HashMap;

#[derive(Debug)]
pub struct Syntax {
    pub contexts: HashMap<String, Context>,
    pub anon_contexts: Vec<Context>,
    pub name: String,
    pub file_extensions: Vec<String>,
    pub scope: String,
    pub variables: HashMap<String, String>,
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
            let mut anon_contexts: Vec<Context> = Vec::new(); // These get populated by the contexts as they are built - Anonymous inline contexts need a place to live
            let contexts: HashMap<String, Context> = values
                .get("contexts")
                .and_then(|v| v.as_mapping())
                .ok_or(Error::BuildingSyntax)?
                .iter()
                .flat_map(|(k, v)| {
                    if let (Some(k), Some(v)) = (k.as_str(), v.as_sequence()) {
                        Ok((
                            k.to_string(),
                            Context::new(v, &variables, &mut anon_contexts)?,
                        ))
                    } else {
                        Err(Error::BuildingSyntax.anyhow())
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
                anon_contexts,
                scope,
            })
        } else {
            Err(Error::BuildingSyntax.anyhow())
        }
    }
}
