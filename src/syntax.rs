use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use once_cell::sync::Lazy;

pub static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
pub static THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);
