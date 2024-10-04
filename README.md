# chatgpt-translator

An OpenAI-powered Markdown document translator. Translate your text into from/to any language.

## Usage

```console
$ ct --help
An OpenAI-powered Markdown document translator. Translate your clipboard/stdin text into from/to any language.

Usage: 

Options:
  -o, --openai-api-key <OPENAI_API_KEY>          OpenAI API key. You can also set the `OPENAI_API_KEY` environment variable [env: OPENAI_API_KEY=sk-...]
  -m, --model <MODEL>                            Model to use. `4o` - gpt-4o, `mini` - gpt-4o-mini, `4` - gpt-4-turbo, `35` - gpt-3.5-turbo [default: mini]
      --max-tokens <MAX_TOKENS>                  The maximum number of tokens to generate in the completion [default: 16384]
      --temperature <TEMPERATURE>                What sampling temperature to use. Higher values means the model will take more risks. Try 0.9 for more creative
                                                 applications, and 0 (argmax sampling) for ones with a well-defined answer [default: 0]
      --frequency-penalty <FREQUENCY_PENALTY>    Number between -2.0 and 2.0. Positive values penalize new tokens based on their existing frequency in the text so far,
                                                 decreasing the model's likelihood to repeat the same line verbatim [default: 1.0]
      --system-prompt-file <SYSTEM_PROMPT_FILE>  A path to a file containing system prompt to use for the translation. If not provided or failed to read a provided
                                                 path, the default system prompt will be used. The system prompt is a fixed message that will be used for all
                                                 translations
      --user-prompt-file <USER_PROMPT_FILE>      A path to a file containing user prompt to use for the translation. If not provided or failed to read a provided path,
                                                 the default prompt will be used. The prompt can contain `{source}` and `{target}` placeholders which will be replaced
                                                 with the source and target language options, respectively
      --system-prompt-text <SYSTEM_PROMPT_TEXT>  System prompt text to use for the translation. If provided, it will override the system prompt file
      --user-prompt-text <USER_PROMPT_TEXT>      User prompt text to use for the translation. If provided, it will override the user prompt file
  -s, --source-language <SOURCE_LANGUAGE>        Original language of the text to translate [default: Japanese]
  -t, --target-language <TARGET_LANGUAGE>        Target language of the text to translate [default: English]
  -v, --verbose...                               Increase logging verbosity
  -q, --quiet...                                 Decrease logging verbosity
  -g, --two-column                               Translate input → two-column HTML table → system clipboard
  -h, --help                                     Print help
  -V, --version                                  Print version
```

i.e.

```console
$ echo "**こんにちは**、_世界_" | ct --source-language Japanese --target-language English
**Hello**, _World_
```

Notable option:

- `--two-column` which will generate a two-column HTML table with the original text on the left and the translated text on the right. The output will be copied to the system clipboard. It'll be useful for translating Markdown documents, then pasting it to Google Docs or other word processors.

i.e.

```console
$ ct --source-language English --target-language Japanese --two-column
```

will generate the following result:

![screenshot](assets/screenshot.png)

See [`assets/default-system-prompt.txt`](assets/default-system-prompt.txt) and [`assets/default-user-prompt.txt`](assets/default-user-prompt.txt) for prompts that can be used for the translation. You can override them by providing your own prompt files or text.

## Contributing

There should be similar and/or more capable tools available in every programming language, so if you have a better option, feel free to keep using it. I wrote this one for fun and quick access. While I don't expect any issues or pull requests, you're welcome to fork it and modify it as you see fit.

## Privacy

Be careful with sensitive information since the text that needs to be translated will be sent to [OpenAI](https://openai.com), obviously. Other than that, the tool doesn't collect any data nor send any data to any other third-party services. See OpenAI's [Privacy policy](https://openai.com/policies/privacy-policy/) for detail.

## License

MIT. See [LICENSE](LICENSE) for more details.
