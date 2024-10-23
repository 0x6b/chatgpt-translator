//! # chatgpt-translator
//!
//! An OpenAI-powered Markdown document translator. Translate your text into from/to any language
//! (as long as it's supported by OpenAI).
pub use document::Document;
pub use language::Language;
pub use model::Model;
pub use translator::{ReadyForTranslation, Translator, TranslatorConfiguration};

mod document;
mod language;
mod model;
mod translator;
