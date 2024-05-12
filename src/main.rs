use anyhow::Result;
use chatgpt_translator::Translator;

#[tokio::main]
async fn main() -> Result<()> {
    let mut translator = Translator::try_new()?;
    let result = translator.run().await?;
    println!("{}", result.join("\n"));

    Ok(())
}
