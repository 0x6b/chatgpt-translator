use std::{
    fs::read_to_string,
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use anyhow::{bail, Result};
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequest, CreateChatCompletionRequestArgs,
    },
    Client,
};
use clap::Parser;
use xdg::BaseDirectories;

use crate::state::{ReadyForTranslation, State, Uninitialized};

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
        let Uninitialized {
            openai_api_key,
            model,
            max_tokens,
            temperature,
            frequency_penalty,
            prompt_file,
            source_language,
            target_language,
        } = Uninitialized::parse();

        let client = Client::with_config(OpenAIConfig::default().with_api_key(openai_api_key));

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(max_tokens)
            .model(model)
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
    let path = match path {
        Some(p) => p,
        None => {
            BaseDirectories::with_prefix("chatgpt_translator")?.place_config_file("prompt.txt")?
        }
    };
    let prompt = (match read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            bail!("Couldn't open prompt at: {path:?}. {e}");
        }
    })
    .replace("{source}", source_language)
    .replace("{target}", target_language);

    Ok(prompt)
}
