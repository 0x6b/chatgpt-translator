use std::{fmt, fmt::Display, str::FromStr};

use anyhow::{bail, Error, Result};

#[derive(Clone)]
pub enum Model {
    Gpt4Turbo,
    Gpt35Turbo,
}

impl From<Model> for String {
    fn from(m: Model) -> Self {
        m.to_string()
    }
}

impl FromStr for Model {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if s.contains('4') {
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
                Self::Gpt4Turbo => "gpt-4-turbo",
                Self::Gpt35Turbo => "gpt-3.5-turbo",
            }
        )
    }
}
