use anyhow::Result;
use arboard::Clipboard;
use chatgpt_translator::{TranslationConfiguration, Translator};
use clap::Parser;
use markdown_split::split;
use tracing_log::AsTrace;

#[derive(Parser, Debug)]
pub struct Args {
    #[command(flatten)]
    pub translation: TranslationConfiguration,

    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

impl From<Args> for TranslationConfiguration {
    fn from(args: Args) -> Self {
        args.translation
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    tracing_subscriber::fmt()
        .with_max_level(args.verbose.log_level_filter().as_trace())
        .init();

    let text = Clipboard::new()
        .expect("failed to access system clipboard")
        .get_text()?
        .trim()
        .to_string();

    let translator = Translator::from(args.into())?;

    for part in split(&text, None)? {
        translator.translate(part).await?.iter().for_each(|l| println!("{l}"));
        println!();
    }

    Ok(())
}
