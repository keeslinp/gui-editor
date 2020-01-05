use crate::{
    error::Result,
    
};

use serde::Deserialize;

pub struct Color {
    r: f32,
    g: f32,
    b: f32,
}

#[derive(Deserialize)]
struct RuleRaw {
    name: String,
    scope: String,
    foreground: Option<String>,
    background: Option<String>,
    font_style: Option<String>,
}

#[derive(Deserialize)]
struct ColorSchemeRaw {
    name: String,
    author: String,
    variables: HashMap<String, String>,
    globals: HashMap<String, String>,
    rules: Vec<String>,
}

pub struct ColorScheme {
}

impl ColorScheme {
    pub fn build() -> Result<ColorScheme> {
        let contents = include!("./colors.sublime-color-scheme");
        let raw: ColorSchemeRaw = serde_json::from_str(contents)?;
        unimplemented!();
    }
}

impl From<ColorSchemeRaw> for ColorScheme {
    fn from(raw: ColorSchemeRaw) -> ColorScheme {
        unimplemented!();
    }
}