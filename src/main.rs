use anyhow::Result;
use arboard::Clipboard;
use chatgpt_translator::{Document, Translator, TranslatorConfiguration};
use clap::Parser;
use log::info;
use markdown::{to_html_with_options, Options};
use markdown_split::split;
use tracing_log::AsTrace;

#[derive(Parser, Debug)]
pub struct Args {
    #[command(flatten)]
    pub config: TranslatorConfiguration,

    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,

    /// Translate input → two-column rich text → system clipboard
    #[arg(short = 'g', long)]
    pub two_column: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    tracing_subscriber::fmt()
        .with_max_level(args.verbose.log_level_filter().as_trace())
        .init();

    info!("Reading text from clipboard");
    let text = Clipboard::new()
        .expect("failed to access system clipboard")
        .get_text()?
        .trim()
        .to_string();

    let translator = Translator::from(args.config)?;

    if args.two_column {
        info!("Parsing text into fragments");
        let document = Document::try_from(text)?;

        info!("Translating fragments");
        let (original, translated) = document.translate(&translator).await?;

        let options = Options::gfm();

        let text = original
            .into_iter()
            .zip(translated)
            .fold(String::new(), |acc, (o, t)| {
                format!(
                    "{}<tr><td>{}</td><td>{}</td></tr>",
                    acc,
                    to_html_with_options(&o, &options).unwrap(),
                    to_html_with_options(&t, &options).unwrap()
                )
            });

        info!("Setting translated text to clipboard");
        Clipboard::new()
            .expect("failed to access system clipboard")
            .set_html(format!("<table>{text}</table>"), Some(text))
            .unwrap();
    } else {
        for part in split(&text, None)? {
            translator.translate(part).await?.iter().for_each(|l| println!("{l}"));
            println!();
        }
    }

    Ok(())
}
