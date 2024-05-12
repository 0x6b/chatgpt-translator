use anyhow::Result;
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequest,
    },
    Client,
};
use clap::Parser;

use crate::model::Model;

pub trait State {}

impl State for Uninitialized {}

impl State for Initialized {}

#[derive(Parser)]
#[clap(about, version)]
pub struct Uninitialized {
    /// OpenAI API key.
    #[arg(short, long, env = "OPENAI_API_KEY")]
    pub openai_api_key: String,

    /// Model to use. If the arg contains "4", it will use gpt-4-turbo. If the arg contains "35",
    /// it will use gpt-3.5-turbo.
    #[arg(short, long, default_value = "4")]
    pub model: Model,

    /// The maximum number of tokens to generate in the completion.
    #[arg(long, default_value = "2000")]
    pub max_tokens: u16,

    /// What sampling temperature to use. Higher values means the model will take more risks. Try
    /// 0.9 for more creative applications, and 0 (argmax sampling) for ones with a well-defined
    /// answer.
    #[arg(long, default_value = "0.6")]
    pub temperature: f32,

    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on their existing
    /// frequency in the text so far, decreasing the model's likelihood to repeat the same line
    /// verbatim.
    #[arg(long, default_value = "1.0")]
    pub frequency_penalty: f32,

    /// Your text to naturalize. Multiple inputs will be concatenated with a space.
    #[arg()]
    pub input: Option<Vec<String>>,
}

pub struct Initialized {
    pub input: String,
    client: Client<OpenAIConfig>,
    request: CreateChatCompletionRequest,
}

impl Initialized {
    pub fn new(api_key: String, input: String, request: CreateChatCompletionRequest) -> Self {
        let config = OpenAIConfig::default().with_api_key(api_key);
        let client = Client::with_config(config);
        Self { client, input, request }
    }

    pub async fn chat(&mut self) -> Result<Vec<String>> {
        self.request.messages = vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content("You are a helpful English technical writing assistant.")
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(format!(r#"Please translate the markdown text below from Japanese to US English. Do not include any explanations nor additional punctuations, only provide an edited text.\n---\n
                {input}
                "#, input = self.input))
                .build()?
                .into(),
        ];

        Ok(self
            .client
            .chat()
            .create(self.request.clone())
            .await?
            .choices
            .into_iter()
            .filter_map(|c| c.message.content)
            .collect::<Vec<_>>())
    }
}
