# Configuration

AutoGPT is configured entirely through environment variables, no config files required. The table below lists every variable with its purpose, whether it is required, and which feature flag it belongs to.

## Core Variables

| Variable               | Required | Default        | Description                                                           |
| ---------------------- | -------- | -------------- | --------------------------------------------------------------------- |
| `AI_PROVIDER`          | No       | `gemini`       | Active LLM provider: `gemini`, `openai`, `anthropic`, `xai`, `cohere` |
| `AUTOGPT_WORKSPACE`    | No       | `workspace/`   | Directory where agents write generated files                          |
| `ORCHESTRATOR_ADDRESS` | CLI only | `0.0.0.0:8443` | TCP address the orchestrator listens on                               |

## LLM Provider API Keys

Configure the key for your chosen provider. Only the key matching your `AI_PROVIDER` value is required.

```sh
# Gemini (default), requires feature: gem
export AI_PROVIDER=gemini
export GEMINI_API_KEY=<your_gemini_api_key>
export GEMINI_MODEL=<your_gemini_model>

# OpenAI, requires feature: oai
export AI_PROVIDER=openai
export OPENAI_API_KEY=<your_openai_api_key>

# Anthropic Claude, requires feature: cld
export AI_PROVIDER=anthropic
export ANTHROPIC_API_KEY=<your_anthropic_api_key>

# XAI Grok, requires feature: xai
export AI_PROVIDER=xai
export XAI_API_KEY=<your_xai_api_key>

# Cohere, requires feature: co
export AI_PROVIDER=cohere
export COHERE_API_KEY=<your_cohere_api_key>
```

Obtain a Gemini API key from [Google AI Studio](https://aistudio.google.com/app/api-keys). Keys for other providers are available on their respective developer portals.

## Workspace

The workspace variable controls where agents write code artifacts. By default all agents write to subdirectories of `workspace/`:

```
workspace/
├── architect/   # ArchitectGPT: diagram.py and generated PNGs
├── backend/     # BackendGPT: main.py, template.py, etc.
├── frontend/    # FrontendGPT: HTML/CSS/JS files
└── designer/    # DesignerGPT: image assets
```

Override the workspace root:

```sh
export AUTOGPT_WORKSPACE=/my/project/workspace/
```

## Orchestrator Address

When running in [Orchestrated Mode](../modes/orchestrated.md), the orchestrator (`orchgpt`) needs a bind address and the agent (`autogpt --net`) needs to know where to connect:

```sh
# On the orchestrator host
export ORCHESTRATOR_ADDRESS=0.0.0.0:8443

# On the agent host (must point to the orchestrator)
export ORCHESTRATOR_ADDRESS=192.168.1.10:8443
```

## Optional: DesignerGPT (`img`)

Enables AI-generated image creation via [GetImg](https://getimg.ai/):

```sh
export GETIMG_API_KEY=<your_getimg_api_key>
```

## Optional: MailerGPT (`mail`)

Enables email reading and sending via [Nylas](https://developer.nylas.com/):

```sh
export NYLAS_SYSTEM_TOKEN=<your_nylas_system_token>
export NYLAS_CLIENT_ID=<your_nylas_client_id>
export NYLAS_CLIENT_SECRET=<your_nylas_client_secret>
```

## Optional: Long-Term Memory (`mem`)

Persists agent memory in [Pinecone](https://www.pinecone.io/) vector database:

```sh
export PINECONE_API_KEY=<your_pinecone_api_key>
export PINECONE_INDEX_URL=<your_pinecone_index_url>
```

The index URL looks like: `https://my-index-abcdef.svc.us-east-1-aws.pinecone.io`

<div class="callout callout-tip">
<strong>💡 Tip</strong>
Store secrets in a <code>.env</code> file and load them with <code>source .env</code> or a tool like <a href="https://direnv.net/">direnv</a>. Never commit API keys to version control.
</div>
