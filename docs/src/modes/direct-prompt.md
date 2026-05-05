# Direct Prompt Mode

Direct Prompt Mode lets you send a single prompt to the active LLM provider and receive an immediate response, no agents, no configuration, no interactive shell. It is the fastest way to query your AI provider from the terminal.

```sh
autogpt -p "<your prompt>"
```

## Examples

Ask a factual question:

```sh
autogpt -p "Explain the Rust borrow checker in simple terms"
```

Generate a code snippet:

```sh
autogpt -p "Write a Rust function that reads a file and returns its contents as a String"
```

Summarize text (pipe from stdin via shell substitution):

```sh
autogpt -p "Summarize the following text: $(cat /path/to/document.txt)"
```

## When to Use Direct Prompt Mode

Direct Prompt Mode is ideal when you want:

- A quick one-shot answer without starting the interactive shell.
- To script AutoGPT into shell pipelines or CI workflows.
- To verify your API key and provider are working correctly.

## Provider Selection

Direct Prompt Mode respects the `AI_PROVIDER` environment variable:

```sh
AI_PROVIDER=openai autogpt -p "What is the capital of France?"
```

The `-p` flag uses the same client initialization as the interactive shell, so all provider API keys and model environment variables apply.

## Output

The response is printed to stdout with no extra formatting. This makes it easy to pipe into other tools:

```sh
autogpt -p "Write a haiku about Rust" | cowsay
```

<div class="callout callout-info">
<strong>ℹ️ Note</strong>
Direct Prompt Mode does not save sessions or use long-term memory. For persistent conversations with memory, use <a href="./interactive.md">Interactive Mode</a>.
</div>
