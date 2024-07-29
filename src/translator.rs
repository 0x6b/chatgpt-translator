use std::{
    fs::read_to_string,
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use anyhow::Result;
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequest, CreateChatCompletionRequestArgs,
    },
    Client,
};
use clap::Parser;

use crate::model::Model;

static DEFAULT_PROMPT: &str = include_str!("../default-prompt.txt");

pub trait State {}

impl State for TranslationConfiguration {}

impl State for ReadyForTranslation {}

/// An alias for the `TranslationConfiguration` which represents the uninitialized state, for
/// consistency.
pub type Uninitialized = TranslationConfiguration;

#[derive(Parser)]
#[clap(about, version)]
pub struct TranslationConfiguration {
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
    /// read a provided path, the default prompt will be used. The prompt can contain `{source}`
    /// and `{target}` placeholders which will be replaced with the source and target language
    /// options, respectively.
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

pub struct Translator<S>
where
    S: State,
{
    state: S,
}

impl<S> Deref for Translator<S>
where
    S: State,
{
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<S> DerefMut for Translator<S>
where
    S: State,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

impl Translator<Uninitialized> {
    pub fn new() -> Result<Translator<ReadyForTranslation>> {
        Self::from(TranslationConfiguration::parse())
    }

    pub fn from(config: TranslationConfiguration) -> Result<Translator<ReadyForTranslation>> {
        let Uninitialized {
            openai_api_key,
            model,
            max_tokens,
            temperature,
            frequency_penalty,
            prompt_file,
            source_language,
            target_language,
        } = config;

        let client = Client::with_config(OpenAIConfig::default().with_api_key(openai_api_key));

        let request = CreateChatCompletionRequestArgs::default()
            .model(model)
            .max_tokens(max_tokens)
            .temperature(temperature)
            .frequency_penalty(frequency_penalty)
            .build()?;

        let prompt = get_prompt(prompt_file, &source_language, &target_language)?;

        Ok(Translator {
            state: ReadyForTranslation { client, request, prompt },
        })
    }
}

impl Translator<ReadyForTranslation> {
    pub async fn translate(&self, input: &str) -> Result<Vec<String>> {
        let mut request = self.request.clone();
        request.messages = vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content("You are a helpful English technical writing assistant.")
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(format!("{}\n{input}", self.prompt))
                .build()?
                .into(),
        ];

        self.send(request).await
    }

    async fn send(&self, request: CreateChatCompletionRequest) -> Result<Vec<String>> {
        Ok(self
            .client
            .chat()
            .create(request)
            .await?
            .choices
            .into_iter()
            .filter_map(|c| c.message.content)
            .collect::<Vec<_>>())
    }
}

fn get_prompt(
    path: Option<PathBuf>,
    source_language: &str,
    target_language: &str,
) -> Result<String> {
    let prompt = match path {
        Some(p) => read_to_string(&p).unwrap_or_else(|_| DEFAULT_PROMPT.to_string()),
        None => DEFAULT_PROMPT.to_string(),
    };
    let prompt = prompt
        .replace("{source}", source_language)
        .replace("{target}", target_language);

    Ok(prompt)
}
