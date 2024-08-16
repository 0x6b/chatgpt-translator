//! # chatgpt-translator
//!
//! An OpenAI-powered Markdown document translator. Translate your text into from/to any language
//! (as long as it's supported by OpenAI).
pub use document::Document;
pub use translator::{ReadyForTranslation, Translator, TranslatorConfiguration};

mod document;
mod model;
mod translator;
