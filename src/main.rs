use anyhow::Result;
use arboard::Clipboard;
use chatgpt_translator::Translator;
use markdown_split::split;

#[tokio::main]
async fn main() -> Result<()> {
    let text = Clipboard::new()
        .expect("failed to access system clipboard")
        .get_text()?
        .trim()
        .to_string();

    let parts = split(&text, None)?;

    let translator = Translator::new()?;

    for part in parts {
        translator.translate(part).await?.iter().for_each(|l| println!("{l}"));
        println!();
    }

    Ok(())
}
