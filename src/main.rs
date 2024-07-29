use anyhow::Result;
use arboard::Clipboard;
use chatgpt_translator::{TranslationConfiguration, Translator};
use clap::Parser;
use markdown_split::split;

#[tokio::main]
async fn main() -> Result<()> {
    let text = Clipboard::new()
        .expect("failed to access system clipboard")
        .get_text()?
        .trim()
        .to_string();

    let translator = Translator::from(TranslationConfiguration::parse())?;

    for part in split(&text, None)? {
        translator.translate(part).await?.iter().for_each(|l| println!("{l}"));
        println!();
    }

    Ok(())
}
