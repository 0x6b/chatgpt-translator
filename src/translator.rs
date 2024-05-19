use std::ops::{Deref, DerefMut};

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
            source_language,
            target_language,
        } = Uninitialized::parse();

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(max_tokens)
            .model(model)
            .temperature(temperature)
            .frequency_penalty(frequency_penalty)
            .build()?;

        let client = Client::with_config(OpenAIConfig::default().with_api_key(openai_api_key));

        Ok(Translator {
            state: ReadyForTranslation { client, request, source_language, target_language },
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
                .content(format!(r#"I am translating the documentation from {source} to {target}. I want you to act as an expert and technical {target} translator. Translate the Markdown content I'll paste later into {target}. You must strictly follow the rules below:

1. **Preserve Markdown Structure**: Never change the Markdown markup structure. Don't add or remove links, and do not change any URLs.
2. **Code Blocks**: Never change the contents of code blocks, even if they appear to have a bug.
3. **Line Breaks**: Always preserve the original line breaks. Do not add or remove blank lines.
4. **No Additional Content**: Do not include any explanations or additional punctuation. Only provide the translated Markdown content.

The Markdown text to be translated is after the "====" line.

====
{input}
"#,
                                 source = self.source_language,
                                 target = self.target_language,
                                 input = input))
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
