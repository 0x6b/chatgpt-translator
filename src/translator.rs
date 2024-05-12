use std::ops::{Deref, DerefMut};

use anyhow::Result;
use arboard::Clipboard;
use async_openai::types::{
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
    CreateChatCompletionRequestArgs,
};
use clap::Parser;

use crate::state::{Initialized, State, Uninitialized};

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
    pub fn new() -> Result<Translator<Initialized>> {
        let Uninitialized {
            openai_api_key,
            model,
            max_tokens,
            temperature,
            frequency_penalty,
            ..
        } = Uninitialized::parse();

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(max_tokens)
            .model(model)
            .temperature(temperature)
            .frequency_penalty(frequency_penalty)
            .build()?;

        Ok(Translator {
            state: Initialized::new(openai_api_key, "".to_string(), request),
        })
    }

    pub fn try_new() -> Result<Translator<Initialized>> {
        let Uninitialized {
            openai_api_key,
            model,
            max_tokens,
            temperature,
            frequency_penalty,
            input,
        } = Uninitialized::parse();

        let input = match &input {
            Some(input) => input.join(" "),
            None => Clipboard::new()
                .expect("failed to access system clipboard")
                .get_text()?,
        }
        .trim()
        .to_string();

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(max_tokens)
            .model(model)
            .temperature(temperature)
            .frequency_penalty(frequency_penalty)
            .messages([
                ChatCompletionRequestSystemMessageArgs::default()
                    .content("You are a helpful English technical writing assistant.")
                    .build()?
                    .into(),
                ChatCompletionRequestUserMessageArgs::default()
                    .content(format!(r#"I am translating the documentation. I want you to act as an expert and technical English translator. Translate the Markdown content I'll paste later into English. You must strictly follow the rules below.

- Never change the Markdown markup structure. Don't add or remove links. Do not change any URL.
- Never change the contents of code blocks even if they appear to have a bug.
- Always preserve the original line breaks. Do not add or remove blank lines.
- Do not include any explanations nor additional punctuations, only provide a translated markdown.
---
{input}
"#))
                    .build()?
                    .into(),
            ])
            .build()?;

        Ok(Translator {
            state: Initialized::new(openai_api_key, input, request),
        })
    }
}

impl Translator<Initialized> {
    pub fn set_input(&mut self, input: String) {
        self.input = input;
    }

    pub async fn run(&mut self) -> Result<Vec<String>> {
        self.chat().await
    }
}
