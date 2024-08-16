use std::{fmt, fmt::Display, str::FromStr};

use anyhow::{bail, Error, Result};

/// The ChatGPT model to use for the translation. See [Models - OpenAI API](https://platform.openai.com/docs/models/chatgpt) for more information.
#[derive(Clone, Debug)]
pub enum Model {
    /// GPT-4o
    Gpt4O,

    /// GPT-4o mini
    Gpt4OMini,

    /// GPT-4 Turbo
    Gpt4Turbo,

    /// GPT-3.5 Turbo
    Gpt35Turbo,
}

impl From<Model> for String {
    fn from(m: Model) -> Self {
        m.to_string()
    }
}

/// Implement the conversion from a string to a [`Model`]. Simply when the string contains the
/// partial name, it returns the corresponding model.
impl FromStr for Model {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if s.contains("4o") {
            Ok(Self::Gpt4O)
        } else if s.contains("mini") {
            Ok(Self::Gpt4OMini)
        } else if s.contains('4') {
            Ok(Self::Gpt4Turbo)
        } else if s.contains("35") {
            Ok(Self::Gpt35Turbo)
        } else {
            bail!("{s} is not a valid model")
        }
    }
}

impl Display for Model {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Gpt4O => "gpt-4o",
                Self::Gpt4OMini => "gpt-4o-mini",
                Self::Gpt4Turbo => "gpt-4-turbo",
                Self::Gpt35Turbo => "gpt-3.5-turbo",
            }
        )
    }
}
