use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum Language {
    Japanese,
    English,
    Other(String),
}

impl From<&str> for Language {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            s if s.starts_with("ja") => Language::Japanese,
            s if s.starts_with("en") => Language::English,
            _ => Language::Other(s.to_string()),
        }
    }
}

impl AsRef<str> for Language {
    fn as_ref(&self) -> &str {
        match self {
            Language::Japanese => "Japanese",
            Language::English => "English",
            Language::Other(lang) => lang,
        }
    }
}

impl Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}
