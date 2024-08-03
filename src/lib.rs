//! # chatgpt-translator
//!
//! An OpenAI-powered Markdown document translator. Translate your clipboard text into from/to any
//! language.
pub use document::Document;
pub use translator::{ReadyForTranslation, Translator, TranslatorConfiguration};

mod document;
mod model;
mod translator;
