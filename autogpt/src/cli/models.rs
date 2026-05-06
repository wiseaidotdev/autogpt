// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/// A normalised model entry built entirely from crate-native type information.
///
/// - `id`           → derived from the provider crate's `Display` or `serde(rename)`.
/// - `display_name` → derived from the provider crate's `Debug` variant name.
/// - `description`  → always empty; crate types carry none.
#[cfg(feature = "cli")]
#[derive(Debug, Clone)]
pub struct ProviderModel {
    pub id: String,
    pub display_name: String,
    pub description: String,
}

/// Returns models for `provider` sourced from the compiled-in provider crate.
///
/// Respects the following environment variables (in order of priority):
/// - `MODEL`: Global model override.
/// - `{PROVIDER}_MODEL`: Specific override for a provider (e.g., `GEMINI_MODEL`).
#[cfg(feature = "cli")]
pub fn provider_models(provider: &str) -> Vec<ProviderModel> {
    let mut models = crate_models(provider);

    let env_model = get_env_model(provider);
    if let Some(override_id) = env_model {
        let override_id = override_id.trim().to_string();
        if !override_id.is_empty() {
            if let Some(pos) = models.iter().position(|m| m.id == override_id) {
                let entry = models.remove(pos);
                models.insert(0, entry);
            } else {
                models.insert(
                    0,
                    ProviderModel {
                        display_name: override_id.clone(),
                        description: String::new(),
                        id: override_id,
                    },
                );
            }
        }
    }

    models
}

/// Helper to get the model ID from environment variables.
/// Checks `{PROVIDER}_MODEL` first, then `MODEL`.
#[cfg(feature = "cli")]
fn get_env_model(provider: &str) -> Option<String> {
    let provider_env = format!("{}_MODEL", provider.to_uppercase());
    if let Ok(m) = std::env::var(&provider_env) {
        return Some(m);
    }
    if let Ok(m) = std::env::var("MODEL") {
        return Some(m);
    }
    None
}

/// Returns the default model ID for `provider`, respecting standard env vars.
#[cfg(feature = "cli")]
pub fn default_model(provider: &str) -> String {
    if let Some(m) = get_env_model(provider) {
        let m = m.trim().to_string();
        if !m.is_empty() {
            return m;
        }
    }
    provider_models(provider)
        .into_iter()
        .next()
        .map(|m| m.id)
        .unwrap_or_else(|| match provider {
            "openai" => "gpt-5".to_string(),
            "anthropic" => "claude-opus-4-6".to_string(),
            "xai" => "grok-4".to_string(),
            "cohere" => "command-a-03-2025".to_string(),
            _ => "gemini-3.0-flash".to_string(),
        })
}

/// Returns the active provider from `AI_PROVIDER`, defaulting to `"gemini"`.
///
/// Accepted values: `gemini`, `openai`, `anthropic`, `xai`, `cohere`.
#[cfg(feature = "cli")]
pub fn default_provider() -> String {
    if let Ok(p) = std::env::var("AI_PROVIDER") {
        let p = p.trim().to_lowercase();
        if matches!(
            p.as_str(),
            "gemini" | "openai" | "anthropic" | "xai" | "cohere"
        ) {
            return p;
        }
    }
    "gemini".to_string()
}

/// Returns the zero-based index of the model matching `model_id`, or 0.
#[cfg(feature = "cli")]
pub fn model_index(models: &[ProviderModel], model_id: &str) -> usize {
    models.iter().position(|m| m.id == model_id).unwrap_or(0)
}

/// Converts an enum variant to a `ProviderModel`.
#[cfg(feature = "cli")]
fn make_model(id: String, display_name: String) -> ProviderModel {
    ProviderModel {
        id,
        display_name,
        description: String::new(),
    }
}

/// Dispatches to the feature-gated provider implementation.
#[cfg(feature = "cli")]
fn crate_models(provider: &str) -> Vec<ProviderModel> {
    match provider {
        #[cfg(feature = "gem")]
        "gemini" => gemini_models(),

        #[cfg(feature = "oai")]
        "openai" => openai_models(),

        #[cfg(feature = "cld")]
        "anthropic" => anthropic_models(),

        #[cfg(feature = "xai")]
        "xai" => xai_models(),

        #[cfg(feature = "co")]
        "cohere" => cohere_models(),

        _ => vec![],
    }
}

#[cfg(all(feature = "cli", feature = "gem"))]
fn gemini_models() -> Vec<ProviderModel> {
    use gems::models::Model;

    let variants = [
        Model::Flash3Preview,
        Model::Pro31Preview,
        Model::Flash31LitePreview,
        Model::Flash31LivePreview,
        Model::Flash31ImagePreview,
        Model::Tts31Preview,
        Model::Embedding001,
        Model::Imagen4,
        Model::Veo31Preview,
    ];

    variants
        .iter()
        .map(|m| make_model(m.to_string(), format!("{m:?}")))
        .collect()
}

#[cfg(all(feature = "cli", feature = "oai"))]
fn openai_models() -> Vec<ProviderModel> {
    use openai_dive::v1::models::{
        EmbeddingModel, Gpt4Model, Gpt5Model, ImageModel, ModerationModel, TTSModel, ToolModel,
        TranscriptionModel,
    };

    let mut models = Vec::new();

    macro_rules! push_serde {
        ($variant:expr) => {
            if let Some(id) = serde_json::to_value(&$variant)
                .ok()
                .and_then(|v| v.as_str().map(String::from))
            {
                models.push(make_model(id, format!("{:?}", $variant)));
            }
        };
    }

    push_serde!(Gpt5Model::Gpt54);
    push_serde!(Gpt5Model::Gpt54Mini);
    push_serde!(Gpt5Model::Gpt54Nano);
    push_serde!(Gpt4Model::Gpt4O);
    push_serde!(Gpt4Model::Gpt41);
    push_serde!(Gpt4Model::Gpt4OAudioPreview);
    push_serde!(ToolModel::Gpt4OSearchPreview);
    push_serde!(ToolModel::Gpt4OMiniSearchPreview);
    push_serde!(ToolModel::ComputerUsePreview);
    push_serde!(EmbeddingModel::TextEmbedding3Small);
    push_serde!(EmbeddingModel::TextEmbedding3Large);
    push_serde!(ImageModel::GptImage1);
    push_serde!(ImageModel::DallE3);
    push_serde!(ImageModel::DallE2);
    push_serde!(TTSModel::Gpt4OMiniTts);
    push_serde!(TTSModel::Tts1);
    push_serde!(TTSModel::Tts1HD);
    push_serde!(TranscriptionModel::Gpt4OTranscribe);
    push_serde!(TranscriptionModel::Whisper1);
    push_serde!(ModerationModel::OmniModerationLatest);

    models.dedup_by(|a, b| a.id == b.id);
    models
}

#[cfg(all(feature = "cli", feature = "cld"))]
fn anthropic_models() -> Vec<ProviderModel> {
    vec![
        make_model("claude-opus-4-6".into(), "Claude4_6Opus".into()),
        make_model("claude-haiku-4-5".into(), "Claude4_5Haiku".into()),
        make_model("claude-opus-4-7".into(), "Claude4_7Opus".into()),
    ]
}

#[cfg(all(feature = "cli", feature = "xai"))]
fn xai_models() -> Vec<ProviderModel> {
    vec![
        make_model("grok-4".into(), "Grok4".into()),
        make_model("grok-4-1-fast".into(), "Grok4_1_Fast".into()),
        make_model(
            "grok-4-1-fast-reasoning".into(),
            "Grok4_1_Fast_Reasoning".into(),
        ),
    ]
}

#[cfg(all(feature = "cli", feature = "co"))]
fn cohere_models() -> Vec<ProviderModel> {
    use cohere_rust::api::GenerateModel;

    let variants = [
        GenerateModel::CommandRPlus,
        GenerateModel::CommandR,
        GenerateModel::CommandRPlus082024,
        GenerateModel::CommandR082024,
        GenerateModel::Command,
        GenerateModel::CommandLight,
        GenerateModel::CommandNightly,
    ];

    variants
        .iter()
        .filter_map(|m| {
            serde_json::to_value(m)
                .ok()
                .and_then(|v| v.as_str().map(String::from))
                .map(|id| make_model(id, format!("{m:?}")))
        })
        .collect()
}

/// Fetches the live model list from Anthropic asynchronously.
#[cfg(all(feature = "cli", feature = "cld"))]
pub async fn anthropic_models_async(api_key: &str) -> Vec<ProviderModel> {
    use anthropic_ai_sdk::{
        client::AnthropicClient,
        types::model::{ListModelsParams, ModelClient},
    };

    let client = AnthropicClient::new::<anthropic_ai_sdk::types::message::MessageError>(
        api_key.to_string(),
        "2023-06-01",
    )
    .unwrap();
    match client.list_models(Some(&ListModelsParams::default())).await {
        Ok(resp) => resp
            .data
            .into_iter()
            .map(|m| make_model(m.id, m.display_name))
            .collect(),
        Err(_) => anthropic_models(),
    }
}

/// Fetches the live model list from xAI asynchronously.
#[cfg(all(feature = "cli", feature = "xai"))]
pub async fn xai_models_async(api_key: &str) -> Vec<ProviderModel> {
    use x_ai::{
        client::XaiClient,
        list_mod::ReducedModelListRequestBuilder,
        traits::{ClientConfig, ListModelFetcher},
    };

    let client = XaiClient::builder().build().unwrap();
    client.set_api_key(api_key.to_string());
    let builder = ReducedModelListRequestBuilder::new(client);
    match builder.fetch_model_info().await {
        Ok(resp) => resp
            .data
            .into_iter()
            .map(|m| make_model(m.id.clone(), m.id))
            .collect(),
        Err(_) => xai_models(),
    }
}
