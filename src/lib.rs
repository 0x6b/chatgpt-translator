//! # chatgpt-translator
//!
//! An OpenAI-powered Markdown document translator. Translate your clipboard text into from/to any
//! language.
pub use translator::{Translator, TranslatorConfiguration};

mod model;
mod translator;
