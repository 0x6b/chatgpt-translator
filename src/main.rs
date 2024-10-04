use anyhow::Result;
use chatgpt_translator::{Document, Translator, TranslatorConfiguration};
use clap::Parser;
use log::info;
use markdown::{to_html_with_options, Options};
use markdown_split::split;
use stdin_or_clipboard::get_text_from_stdin_or_clipboard;
use tracing_log::AsTrace;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
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

    let (text, clipboard) = get_text_from_stdin_or_clipboard().await?;
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
        if let Some(mut clipboard) = clipboard {
            if clipboard.set_html(&html, Some(&text)).is_ok() {
                println!("{text}");
                return Ok(());
            }
        }
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
