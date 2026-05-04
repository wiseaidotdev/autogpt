// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::common::utils::ClientType;
use crate::common::utils::Message;
use anyhow::Result;
use pinecone_sdk::models::{Kind, Value, Vector};
use pinecone_sdk::pinecone::PineconeClientConfig;
use std::borrow::Cow;
use std::collections::BTreeMap;
use tracing::{error, warn};

async fn embed_text(client: &mut ClientType, content: Cow<'static, str>) -> Vec<f64> {
    match client {
        #[cfg(feature = "gem")]
        ClientType::Gemini(gem_client) => {
            use gems::embed::EmbeddingBuilder;
            use gems::messages::Content;
            use gems::messages::Message;
            use gems::models::Model;
            use gems::traits::CTrait;

            let params = EmbeddingBuilder::default()
                .model(Model::Embedding001)
                .input(Message::User {
                    content: Content::Text(content.into()),
                    name: None,
                })
                .build()
                .unwrap_or_default();
            gem_client.set_model(Model::Embedding001);
            let response = gem_client.embeddings().create(params).await;
            gem_client.set_model(Model::Flash3Preview);
            match response {
                Ok(embed_response) => {
                    if let Some(embedding) = embed_response.embedding {
                        embedding.values
                    } else {
                        error!("Gemini: No embedding returned.");
                        vec![]
                    }
                }
                Err(err) => {
                    error!("Gemini: Failed to embed content: {}", err);
                    vec![]
                }
            }
        }
        #[cfg(feature = "oai")]
        ClientType::OpenAI(oai_client) => {
            use openai_dive::v1::models::EmbeddingModel;
            use openai_dive::v1::resources::embedding::{
                EmbeddingEncodingFormat, EmbeddingInput, EmbeddingOutput,
                EmbeddingParametersBuilder,
            };

            let parameters = EmbeddingParametersBuilder::default()
                .model(EmbeddingModel::TextEmbedding3Small.to_string())
                .input(EmbeddingInput::String(content.to_string()))
                .encoding_format(EmbeddingEncodingFormat::Float)
                .build()
                .unwrap();

            match oai_client.embeddings().create(parameters).await {
                Ok(response) => {
                    if let Some(embedding) = response.data.first() {
                        match &embedding.embedding {
                            EmbeddingOutput::Float(vec) => vec.clone(),
                            EmbeddingOutput::Base64(_) => {
                                error!("OpenAI: Expected embedding as Float, found Base64.");
                                vec![]
                            }
                        }
                    } else {
                        error!("OpenAI: No embedding returned.");
                        vec![]
                    }
                }
                Err(err) => {
                    error!("OpenAI: Failed to embed content: {}", err);
                    vec![]
                }
            }
        }
        #[cfg(feature = "co")]
        ClientType::Cohere(co_client) => {
            use cohere_rust::api::embed::EmbedRequest;
            use cohere_rust::api::{EmbedModel, Truncate};

            let parameters = EmbedRequest {
                model: Some(EmbedModel::EnglishV3),
                texts: &[content.to_string()],
                truncate: Truncate::None,
            };

            match co_client.embed(&parameters).await {
                Ok(response) => {
                    if let Some(embedding) = response.first() {
                        embedding.clone()
                    } else {
                        error!("Cohere: No embedding returned.");
                        vec![]
                    }
                }
                Err(err) => {
                    error!("Cohere: Failed to embed content: {:?}", err);
                    vec![]
                }
            }
        }

        // TODO: Add embeddings for claude and xai
        #[allow(unreachable_patterns)]
        _ => {
            error!("Unsupported AI client for embedding.");
            vec![]
        }
    }
}

pub async fn save_long_term_memory(
    client: &mut ClientType,
    agent_id: Cow<'static, str>,
    message: Message,
) -> Result<()> {
    let config = PineconeClientConfig {
        api_key: Some(std::env::var("PINECONE_API_KEY").unwrap_or_default()),
        ..Default::default()
    };

    let pinecone_result = config.client();
    let pinecone = match pinecone_result {
        Ok(client) => client,
        Err(e) => {
            error!("Error creating Pinecone client: {:?}", e);
            return Err(std::io::Error::other("Failed to create Pinecone client").into());
        }
    };

    let index_result = pinecone
        .index(&std::env::var("PINECONE_INDEX_URL").unwrap_or_default())
        .await;
    let mut index = match index_result {
        Ok(index) => index,
        Err(e) => {
            error!("Error connecting to Pinecone index: {:?}", e);
            return Err(std::io::Error::other("Failed to connect to Pinecone index").into());
        }
    };

    let namespace = format!("agent-{agent_id}");
    let values_f32: Vec<f32> = embed_text(client, message.content.clone())
        .await
        .into_iter()
        .map(|v| v as f32)
        .collect();
    let mut padded_values: Vec<f32> = values_f32;
    if padded_values.len() > 1024 {
        padded_values.truncate(1024);
    } else {
        padded_values.resize(1024, 0.0);
    }

    let content = message.content.clone();
    let role = message.role.clone();

    let vector = Vector {
        id: uuid::Uuid::new_v4().to_string(),
        values: padded_values,
        sparse_values: None,
        metadata: Some(pinecone_sdk::models::Metadata {
            fields: BTreeMap::from([
                (
                    "role".to_string(),
                    Value {
                        kind: Some(Kind::StringValue(role.to_string())),
                    },
                ),
                (
                    "content".to_string(),
                    Value {
                        kind: Some(Kind::StringValue(content.to_string())),
                    },
                ),
            ]),
        }),
    };
    if let Err(_e) = index.upsert(&[vector], &namespace.into()).await {
        warn!("Upsert failed -> check `PINECONE_INDEX_URL` and trial limits.");
    }
    Ok(())
}

pub async fn load_long_term_memory(agent_id: Cow<'static, str>) -> Result<Vec<Message>> {
    let config = PineconeClientConfig {
        api_key: Some(std::env::var("PINECONE_API_KEY").unwrap()),
        ..Default::default()
    };

    let pinecone_result = config.client();
    let pinecone = match pinecone_result {
        Ok(client) => client,
        Err(e) => {
            error!("Error creating Pinecone client: {:?}", e);
            return Err(std::io::Error::other("Failed to create Pinecone client").into());
        }
    };

    let index_result = pinecone
        .index(&std::env::var("PINECONE_INDEX_URL").unwrap_or_default())
        .await;
    let mut index = match index_result {
        Ok(index) => index,
        Err(e) => {
            error!("Error connecting to Pinecone index: {:?}", e);
            return Err(std::io::Error::other("Failed to connect to Pinecone index").into());
        }
    };

    let namespace = format!("agent-{agent_id}");
    let list = index
        .list(&namespace.clone().into(), None, None, None)
        .await
        .unwrap();

    let ids: Vec<&str> = list.vectors.iter().map(|v| v.id.as_str()).collect();

    let fetched_result = index.fetch(&ids, &namespace.into()).await;

    let messages = if let Ok(fetched) = fetched_result {
        fetched
            .vectors
            .values()
            .map(|v| {
                let metadata_opt = v.metadata.as_ref();

                let role = metadata_opt
                    .and_then(|meta| meta.fields.get("role"))
                    .and_then(|v| v.kind.as_ref())
                    .and_then(|kind| match kind {
                        Kind::StringValue(val) => Some(Cow::Owned(val.clone())),
                        _ => None,
                    })
                    .unwrap_or(Cow::Borrowed("unknown"));

                let content = metadata_opt
                    .and_then(|meta| meta.fields.get("content"))
                    .and_then(|v| v.kind.as_ref())
                    .and_then(|kind| match kind {
                        Kind::StringValue(val) => Some(Cow::Owned(val.clone())),
                        _ => None,
                    })
                    .unwrap_or(Cow::Borrowed(""));

                Message { role, content }
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    Ok(messages)
}

pub async fn long_term_memory_context(agent_id: Cow<'static, str>) -> String {
    match load_long_term_memory(agent_id).await {
        Ok(messages) => messages
            .iter()
            .map(|c| format!("{}: {}", c.role, c.content))
            .collect::<Vec<_>>()
            .join("\n"),
        Err(_) => String::from(""),
    }
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
