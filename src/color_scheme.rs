use crate::buffer::highlighter::syntax::Scope;
use anyhow::Result;

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
struct RuleRaw {
    scope: String,
    foreground: Option<String>,
    background: Option<String>,
}

#[derive(Deserialize)]
struct ColorSchemeRaw {
    variables: HashMap<String, String>,
    rules: Vec<RuleRaw>,
    // TODO: Figure out globals stuff
}

#[derive(Clone, Debug)]
pub struct Rule {
    scopes: Vec<String>,
    foreground: Option<[f32; 4]>,
    background: Option<[f32; 4]>,
}

#[derive(Debug)]
pub struct ColorScheme {
    rules: Vec<Rule>,
}

use std::convert::{TryFrom, TryInto};

impl ColorScheme {
    pub fn build() -> Result<ColorScheme> {
        let contents = include_str!("./colors.sublime-color-scheme");
        let raw: ColorSchemeRaw = serde_json::from_str(contents)?;
        raw.try_into()
    }

    pub fn get_fg_color_for_scope(&self, scope: &Scope) -> Option<[f32; 4]> {
        let mut max_depth: usize = 0;
        let mut rule_match: Option<[f32; 4]> = None;
        for rule in self.rules.iter() {
            for rule_scope in rule.scopes.iter() {
                if rule_scope.len() > max_depth {
                    if scope.name.starts_with(rule_scope.as_str()) {
                        max_depth = rule_scope.len();
                        rule_match = rule.foreground;
                    }
                }
            }
        }
        rule_match
    }
}

fn parse_rule_string(raw: &str, variables: &HashMap<String, [f32; 4]>) -> Option<[f32; 4]> {
    use lazy_static::lazy_static;
    use regex::Regex;
    lazy_static! {
        static ref VAR: Regex = Regex::new(r"var\(([^\)]+)\)").unwrap();
    }
    VAR.captures(raw)?
        .get(1)
        .and_then(|v| variables.get(v.as_str()))
        .copied() // TODO: Smarter parsing
}

fn build_rule(raw: &RuleRaw, variables: &HashMap<String, [f32; 4]>) -> Rule {
    Rule {
        scopes: raw.scope.split(",").map(|r| r.trim().to_owned()).collect(),
        foreground: raw
            .foreground
            .as_ref()
            .and_then(|v| parse_rule_string(v.as_str(), variables)),
        background: raw
            .background
            .as_ref()
            .and_then(|v| parse_rule_string(v.as_str(), variables)),
    }
}

impl TryFrom<ColorSchemeRaw> for ColorScheme {
    type Error = anyhow::Error;
    fn try_from(raw: ColorSchemeRaw) -> Result<ColorScheme> {
        let variables = {
            let mut map = HashMap::new();
            for (k, v) in raw.variables.iter() {
                let (r, g, b, a) = colorsys::Rgb::from_hex_str(v)?.into();
                map.insert(
                    k.to_owned(),
                    [r as f32 / 255., g as f32 / 255., b as f32 / 255., a as f32],
                ); // gotta normalize
            }
            map
        };
        Ok(ColorScheme {
            rules: raw
                .rules
                .into_iter()
                .map(|raw_rule| build_rule(&raw_rule, &variables))
                .collect(),
        })
    }
}
