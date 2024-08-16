use std::convert::TryFrom;

use anyhow::Result;
use log::info;
use markdown_split::split;

use crate::{ReadyForTranslation, Translator};

pub struct Document {
    pub fragments: Vec<String>,
}

impl TryFrom<String> for Document {
    type Error = anyhow::Error;

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
