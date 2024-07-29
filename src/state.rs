use std::path::PathBuf;

use async_openai::{config::OpenAIConfig, types::CreateChatCompletionRequest, Client};
use clap::Parser;

use crate::model::Model;

pub trait State {}

impl State for Uninitialized {}

impl State for ReadyForTranslation {}

#[derive(Parser)]
#[clap(about, version)]
pub struct Uninitialized {
    /// OpenAI API key. You can also set the `OPENAI_API_KEY` environment variable.
    #[arg(short, long, env = "OPENAI_API_KEY")]
    pub openai_api_key: String,

    /// Model to use. `4o` - gpt-4o, `mini` - gpt-4o-mini, `4` - gpt-4-turbo, `35` - gpt-3.5-turbo
    #[arg(short, long, default_value = "mini")]
    pub model: Model,

    /// The maximum number of tokens to generate in the completion.
    #[arg(long, default_value = "16384")]
    pub max_tokens: u16,

    /// What sampling temperature to use. Higher values means the model will take more risks. Try
    /// 0.9 for more creative applications, and 0 (argmax sampling) for ones with a well-defined
    /// answer.
    #[arg(long, default_value = "0")]
    pub temperature: f32,

    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on their existing
    /// frequency in the text so far, decreasing the model's likelihood to repeat the same line
    /// verbatim.
    #[arg(long, default_value = "1.0")]
    pub frequency_penalty: f32,

    /// A path to a file containing prompt to use for the translation. If not provided or failed to
    /// read a provided path, the default prompt will be used.
    #[arg(long)]
    pub prompt_file: Option<PathBuf>,

    /// Original language of the text to translate
    #[arg(short, long, default_value = "Japanese")]
    pub source_language: String,

    /// Target language of the text to translate
    #[arg(short, long, default_value = "English")]
    pub target_language: String,
}

pub struct ReadyForTranslation {
    pub(crate) client: Client<OpenAIConfig>,
    pub(crate) request: CreateChatCompletionRequest,
    pub prompt: String,
}
