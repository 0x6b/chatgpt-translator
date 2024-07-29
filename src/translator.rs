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

use crate::{
    state::{ReadyForTranslation, State, Uninitialized},
    TranslationConfiguration,
};

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

impl Translator<TranslationConfiguration> {
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

static DEFAULT_PROMPT: &str = include_str!("../default-prompt.txt");

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
