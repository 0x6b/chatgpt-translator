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
use log::debug;

use crate::model::Model;

static DEFAULT_SYSTEM_PROMPT: &str = include_str!("../assets/default-system-prompt.txt");
static DEFAULT_USER_PROMPT: &str = include_str!("../assets/default-user-prompt.txt");

/// A marker trait to represent the state of the [`Translator`]. The state transitions from
/// [`Uninitialized`] to [`ReadyForTranslation`]. The type system enforces the state transitions to
/// prevent using the translator in an invalid state. The methods of the [`Translator`] struct are
/// implemented based on the state, which means that the compiler helps to prevent using the
/// translator in an invalid state.
pub trait State {}

/// The initial state of the translator is [`Uninitialized`], or its alias
/// [`TranslatorConfiguration`].
impl State for TranslatorConfiguration {}

/// An alias for the [`TranslatorConfiguration`]. This alias is just for clarity and consistency to
/// have a type name that represents the uninitialized state.
pub type Uninitialized = TranslatorConfiguration;

/// After the configuration is parsed, the state transitions to [`ReadyForTranslation`], and as you
/// can imagine, it's ready to translate given text.
impl State for ReadyForTranslation {}

/// Configuration for the [`Translator`]. The structs derive the [`Parser`] trait from [`clap`] to
/// be conveniently constructed from the command line arguments.
#[derive(Parser, Debug, Clone)]
#[clap(about, version)]
pub struct TranslatorConfiguration {
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

    /// A path to a file containing system prompt to use for the translation. If not provided or
    /// failed to read a provided path, the default system prompt will be used. The system prompt
    /// is a fixed message that will be used for all translations.
    #[arg(long)]
    pub system_prompt_file: Option<PathBuf>,

    /// A path to a file containing user prompt to use for the translation. If not provided or
    /// failed to read a provided path, the default prompt will be used. The prompt can contain
    /// `{source}` and `{target}` placeholders which will be replaced with the source and
    /// target language options, respectively.
    #[arg(long)]
    pub user_prompt_file: Option<PathBuf>,

    /// System prompt text to use for the translation. If provided, it will override the system
    /// prompt file.
    #[arg(long)]
    pub system_prompt_text: Option<String>,

    /// User prompt text to use for the translation. If provided, it will override the user prompt
    /// file.
    #[arg(long)]
    pub user_prompt_text: Option<String>,

    /// Original language of the text to translate
    #[arg(short, long, default_value = "Japanese")]
    pub source_language: String,

    /// Target language of the text to translate
    #[arg(short, long, default_value = "English")]
    pub target_language: String,
}

/// Represents the state of the translator after the configuration is parsed and ready to translate.
pub struct ReadyForTranslation {
    pub(crate) client: Client<OpenAIConfig>,
    pub(crate) request: CreateChatCompletionRequest,
    pub user_prompt: String,
    pub system_prompt: String,
}

/// The main struct that represents the translator. It holds the state of the translator.
pub struct Translator<S>
where
    S: State,
{
    state: S,
}

/// Implement deref and deref_mut for the [`Translator`] struct to access its inner state easily.
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
    /// Create a new translator with parsing the command line arguments.
    pub fn new() -> Result<Translator<ReadyForTranslation>> {
        Self::from(TranslatorConfiguration::parse())
    }

    /// Create a new translator from given [`TranslatorConfiguration`].
    pub fn from(config: TranslatorConfiguration) -> Result<Translator<ReadyForTranslation>> {
        let TranslatorConfiguration {
            openai_api_key,
            model,
            max_tokens,
            temperature,
            frequency_penalty,
            system_prompt_file,
            user_prompt_file,
            system_prompt_text,
            user_prompt_text,
            source_language,
            target_language,
        } = config;
        debug!("Translation configuration:");
        debug!("- OpenAI API key: <redacted>");
        debug!("- Model: {}", model);
        debug!("- Max tokens: {}", max_tokens);
        debug!("- Temperature: {}", temperature);
        debug!("- Frequency penalty: {}", frequency_penalty);
        debug!("- System prompt file: {:?}", system_prompt_file);
        debug!("- User prompt file: {:?}", user_prompt_file);
        debug!("- System prompt text: {:?}", system_prompt_text);
        debug!("- User prompt text: {:?}", user_prompt_text);
        debug!("- Source language: {}", source_language);
        debug!("- Target language: {}", target_language);

        let client = Client::with_config(OpenAIConfig::default().with_api_key(openai_api_key));

        let request = CreateChatCompletionRequestArgs::default()
            .model(model)
            .max_tokens(max_tokens)
            .temperature(temperature)
            .frequency_penalty(frequency_penalty)
            .build()?;

        let mut system_prompt = get_prompt(
            system_prompt_file,
            DEFAULT_SYSTEM_PROMPT,
            &source_language,
            &target_language,
        )?;
        if let Some(t) = system_prompt_text {
            debug!("Overriding system prompt with provided text");
            system_prompt = t;
        }
        debug!("System prompt:\n----------------------------------------\n{system_prompt}\n----------------------------------------");

        let mut user_prompt =
            get_prompt(user_prompt_file, DEFAULT_USER_PROMPT, &source_language, &target_language)?;
        if let Some(t) = user_prompt_text {
            debug!("Overriding user prompt with provided text");
            user_prompt = t;
        }
        debug!("User prompt:\n----------------------------------------\n{user_prompt}\n----------------------------------------");

        Ok(Translator {
            state: ReadyForTranslation { client, request, system_prompt, user_prompt },
        })
    }
}

impl Translator<ReadyForTranslation> {
    /// Translate the provided text.
    pub async fn translate(&self, input: &str) -> Result<Vec<String>> {
        let mut request = self.request.clone();
        debug!("Provided input:\n----------------------------------------\n{input}\n----------------------------------------");
        request.messages = vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content(&self.system_prompt)
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(format!("{}\n{input}", self.user_prompt))
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

/// Get the prompt from the provided path or use the default prompt.
fn get_prompt(
    path: Option<PathBuf>,
    default: &str,
    source_language: &str,
    target_language: &str,
) -> Result<String> {
    let prompt = match path {
        Some(p) => read_to_string(&p).unwrap_or(default.to_string()),
        None => default.to_string(),
    }
    .replace("{source}", source_language)
    .replace("{target}", target_language);

    Ok(prompt.to_string())
}
