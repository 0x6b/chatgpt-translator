use anyhow::{bail, Result};
use arboard::Clipboard;
use chatgpt_translator::Translator;
use regex::Regex;

#[tokio::main]
async fn main() -> Result<()> {
    let text = Clipboard::new()
        .expect("failed to access system clipboard")
        .get_text()?
        .trim()
        .to_string();

    let parts = split_markdown_by_headings(&text)?;

    let translator = Translator::new()?;

    for part in parts {
        translator.translate(part).await?.iter().for_each(|l| println!("{l}"));
        println!();
    }

    Ok(())
}

fn split_markdown_by_headings(text: &str) -> Result<Vec<&str>> {
    if text.is_empty() {
        bail!("empty text")
    }

    let re = Regex::new(r"(?m)^#.*$").unwrap();
    let mut result = Vec::new();
    let mut last = 0;

    for mat in re.find_iter(text) {
        let range = mat.range();
        if range.start != last {
            result.push(text[last..range.start].trim());
        }
        last = range.start;
    }

    if last < text.len() {
        result.push(text[last..].trim());
    }

    Ok(result)
}
