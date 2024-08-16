use std::convert::TryFrom;

use anyhow::Result;
use log::info;
use markdown_split::split;

use crate::{ReadyForTranslation, Translator};

/// Represents a document to translate. It just holds the vector of strings. Each string is a
/// section of the original text.
pub struct Document {
    /// Fragments of the text.
    pub fragments: Vec<String>,
}

impl TryFrom<String> for Document {
    type Error = anyhow::Error;

    /// Try to create a new document from the given text. Text is split into sections based on
    /// Markdown headings (h1-h6).
    fn try_from(text: String) -> Result<Self> {
        Ok(Self {
            fragments: split(&text, None)?
                .into_iter()
                .map(String::from)
                .collect::<Vec<String>>(),
        })
    }
}

impl Document {
    /// Translate the document using the provided translator.
    ///
    /// # Arguments
    ///
    /// - `translator` - The translator to use.
    ///
    /// # Returns
    ///
    /// The [`Vec`] of translated text.
    pub async fn translate(
        &self,
        translator: &Translator<ReadyForTranslation>,
    ) -> Result<Vec<String>> {
        let mut result = Vec::new();

        let mut count = 1;
        for fragment in &self.fragments {
            info!("Translating fragment {}/{}", count, self.fragments.len());
            let translations = translator.translate(fragment).await?;
            result.extend(translations.iter().map(|t| t.to_string()));
            count += 1;
        }

        Ok(result)
    }
}
