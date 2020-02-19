use crate::error::Error;
use anyhow::Result;
use fancy_regex::Regex as FRegex;
use regex::Regex;
use once_cell::sync::OnceCell;
use serde::Deserialize;

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
pub enum MatchAction {
    Push(MatchValue),
    Pop,
    Set(MatchValue),
}

impl From<fancy_regex::Error> for Error {
    fn from(_err: fancy_regex::Error) -> Error {
        Error::BuildingSyntax
    }
}

#[derive(Debug, Clone)]
pub enum MatchValue {
    Name(String),
    Inline(usize), // Indexes into anon_contexts
    List(Vec<MatchValue>)
}

impl MatchValue {
    fn build(raw: &MatchActionRaw, variables: &HashMap<String, String>, anon_contexts: &mut Vec<Context>) -> Result<MatchValue> {
        Ok(match raw {
            MatchActionRaw::Name(name) => MatchValue::Name(name.clone()),
            MatchActionRaw::Inline(ctx) => {
                let new_ctx = Context::build(ctx, variables, anon_contexts)?;
                anon_contexts.push(new_ctx);
                MatchValue::Inline(anon_contexts.len() - 1)
            },
            MatchActionRaw::List(values) => MatchValue::List(values.iter().filter_map(|value| MatchValue::build(value, variables, anon_contexts).ok()).collect())
        })
    }
}

#[derive(Debug)]
pub struct Match {
    pub regex: FRegex,
    pub action: Option<MatchAction>,
    pub scope: Option<Scope>,
    pub captures: Option<Vec<Scope>>,
}

#[derive(Debug)]
pub enum ContextElement {
    Include(String),
    Match(Match),
}

#[derive(Debug)]
pub struct Context {
    pub elements: Vec<ContextElement>,
    pub meta_scope: Option<Scope>,
    pub meta_content_scope: Option<Scope>,
    pub meta_include_prototype: Option<bool>,
    pub clear_scopes: Option<u8>,
}

use std::collections::HashMap;



const RAW_SYNTAXES: &[&'static str] = &[
    include_str!("./Rust.sublime-syntax"),
    include_str!("./JavaScript.sublime-syntax")
];

static SYNTAXES: OnceCell<Vec<Syntax>> = OnceCell::new();

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum MatchActionRaw {
    Name(String),
    Inline(ContextRaw),
    List(Vec<MatchActionRaw>),
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum ContextSegmentRaw {
    Include {
        include: String,
    },
    Match {
        #[serde(rename = "match")]
        match_val: String,
        scope: Option<String>,
        captures: Option<HashMap<usize, String>>,
        push: Option<MatchActionRaw>,
        pop: Option<bool>,
        set: Option<MatchActionRaw>,
    },
    MetaScope {
        meta_scope: String,
    },
    MetaContentScope{
        meta_content_scope: String,
    },
    MetaIncludePrototype {
        meta_include_prototype: bool,
    },
    ClearScopes {
        clear_scopes: u8,
    },
}

// Just for debugging
#[derive(Deserialize, Debug)]
struct Empty {
}

type ContextRaw = Vec<ContextSegmentRaw>;

#[derive(Deserialize, Debug)]
struct SyntaxRaw {
    file_extensions: Vec<String>,
    name: String,
    variables: HashMap<String, String>,
    contexts: HashMap<String, ContextRaw>,
    scope: String,
}

#[derive(Debug)]
pub struct Syntax {
    pub contexts: HashMap<String, Context>,
    pub anon_contexts: Vec<Context>,
    pub name: String,
    pub file_extensions: Vec<String>,
    pub scope: String,
    pub variables: HashMap<String, String>,
}

use std::convert::{TryFrom, TryInto};

use lazy_static::lazy_static;

impl Context {
    fn build(raw: &ContextRaw, variables: &HashMap<String, String>, anon_contexts: &mut Vec<Context>) -> Result<Context> {
        lazy_static! {
            static ref VAR_RE: Regex = Regex::new(r"\{\{([^\}]*)\}\}").unwrap();
        }
        let meta_scope = raw.iter().find_map(|segment| {
            if let ContextSegmentRaw::MetaScope { meta_scope } = segment {
                Some(meta_scope.clone().into())
            } else {
                None
            }
        });
        let clear_scopes = raw.iter().find_map(|segment| {
            if let ContextSegmentRaw::ClearScopes { clear_scopes } = segment {
                Some(*clear_scopes)
            } else {
                None
            }
        });
        let meta_include_prototype = raw.iter().find_map(|segment| {
            if let ContextSegmentRaw::MetaIncludePrototype { meta_include_prototype } = segment {
                Some(*meta_include_prototype)
            } else {
                None
            }
        });
        let meta_content_scope = raw.iter().find_map(|segment| {
            if let ContextSegmentRaw::MetaContentScope { meta_content_scope } = segment {
                Some(meta_content_scope.clone().into())
            } else {
                None
            }
        });
        let elements = raw.iter().filter_map(|segment| match segment {
            ContextSegmentRaw::Include { include } => Some(ContextElement::Include(include.clone())),
            ContextSegmentRaw::Match {
                 push,
                 pop,
                 set,
                 captures,
                 scope,
                 match_val,
            } => Some(ContextElement::Match(Match {
                action: match (push, pop, set) {
                    (Some(push), None, None) => Some(MatchAction::Push(MatchValue::build(push, variables, anon_contexts).expect("Building MatchValue from Raw"))),
                    (None, Some(_), None) => Some(MatchAction::Pop),
                    (None, None, Some(set)) => Some(MatchAction::Set(MatchValue::build(set, variables, anon_contexts).expect("Building MatchValue from Raw"))),
                    (None, None, None) => None,
                    _ => panic!("Only one of push, pop, or set is allowed"),
                },
                captures: captures.as_ref().map(|captures| captures.values().map(|scope| scope.clone().into()).collect()),
                scope: scope.as_ref().map(|s| s.clone().into()),
                regex: {
                    let mut stable;
                    let mut temp = match_val.to_owned();
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
                    if let Ok(regex) = FRegex::new(temp.as_ref()) {
                        regex
                    } else {
                        eprintln!("regex failed: {:?}", &temp);
                        return None; // There are some weird backref problems I haven't solved
                    }
                },
            })),
            _ => None,
        }).collect();
        Ok(Context {
            meta_scope,
            clear_scopes,
            meta_content_scope,
            meta_include_prototype,
            elements,
        })
    }
}

impl TryFrom<SyntaxRaw> for Syntax {
    type Error = anyhow::Error;
    fn try_from(raw: SyntaxRaw) -> Result<Syntax> {
        let (contexts, anon_contexts) = {
            let mut contexts = HashMap::new();
            let mut anon_contexts = Vec::new();
            for (name, context_segments) in raw.contexts.iter() {
                contexts.insert(name.clone(), Context::build(context_segments, &raw.variables, &mut anon_contexts)?);
            }
            (contexts, anon_contexts)
        };
        Ok(Syntax {
            variables: raw.variables,
            name: raw.name,
            file_extensions: raw.file_extensions,
            scope: raw.scope,
            contexts,
            anon_contexts,
        })
    }
}

impl Syntax {
    pub fn new(file_extension: &str) -> Result<&'static Syntax> {
        let syntaxes = SYNTAXES.get_or_init(|| {
            RAW_SYNTAXES.iter().filter_map(|raw| {
                serde_yaml::from_str::<SyntaxRaw>(raw).ok()?.try_into().ok()
            }).collect()
        });
        for syntax in syntaxes.iter() {
            if syntax.file_extensions.iter().any(|fe| fe == file_extension) {
                return Ok(syntax);
            }
        }
        Err(Error::UnknownSyntax.anyhow())
    }
}

