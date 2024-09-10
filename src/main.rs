use std::io::IsTerminal;

use anyhow::Result;
use arboard::Clipboard;
use chatgpt_translator::{Document, Translator, TranslatorConfiguration};
use clap::Parser;
use log::info;
use markdown::{to_html_with_options, Options};
use markdown_split::split;
use tokio::io::{stdin, AsyncReadExt};
use tracing_log::AsTrace;

#[derive(Parser, Debug)]
pub struct Args {
    #[command(flatten)]
    pub config: TranslatorConfiguration,

    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,

    /// Translate input → two-column HTML table → system clipboard
    #[arg(short = 'g', long)]
    pub two_column: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    tracing_subscriber::fmt()
        .with_max_level(args.verbose.log_level_filter().as_trace())
        .init();

    let text = if std::io::stdin().is_terminal() {
        info!("Reading text from clipboard");
        Clipboard::new()
            .expect("failed to access system clipboard")
            .get_text()?
            .trim()
            .to_string()
    } else {
        info!("Reading text from stdin");
        let mut buffer = String::new();
        stdin().read_to_string(&mut buffer).await?;
        buffer.trim().to_string()
    };

    let translator = Translator::from(args.config)?;

    if args.two_column {
        info!("Parsing text into fragments");
        let document = Document::try_from(text)?;

        info!("Translating fragments");
        let translated = document.translate(&translator).await?;

        let options = Options::gfm();

        let html = format!(
            "<table>{}</table>",
            document
                .fragments
                .iter()
                .zip(&translated)
                .fold(String::new(), |acc, (o, t)| {
                    format!(
                        "{}<tr><td>{}</td><td>{}</td></tr>",
                        acc,
                        to_html_with_options(o, &options).unwrap(),
                        to_html_with_options(t, &options).unwrap()
                    )
                })
        );

        let text = document
            .fragments
            .iter()
            .chain(&translated)
            .fold(String::new(), |acc, t| format!("{}\n{}\n", acc, t.trim()));

        info!("Setting translated text to clipboard");
        Clipboard::new()
            .expect("failed to access system clipboard")
            .set_html(html, Some(text))?;
    } else {
        for fragment in split(&text, None)? {
            translator
                .translate(fragment)
                .await?
                .iter()
                .for_each(|l| println!("{l}"));
            println!();
        }
    }

    Ok(())
}
