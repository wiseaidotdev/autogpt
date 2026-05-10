// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate proc_macro;

use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(Auto)]
pub fn derive_agent(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl Agent for #name {
            fn new(persona: Cow<'static, str>, behavior: Cow<'static, str>) -> Self {
                let mut agent = Self::default();
                agent.agent.persona = persona;
                agent.agent.behavior = behavior;
                agent
            }

            fn update(&mut self, status: Status) {
                self.agent.update(status);
            }

            fn behavior(&self) -> &std::borrow::Cow<'static, str> {
                &self.agent.behavior
            }

            fn persona(&self) -> &std::borrow::Cow<'static, str> {
                &self.agent.persona
            }

            fn status(&self) -> &Status {
                &self.agent.status
            }

            fn memory(&self) -> &Vec<Message> {
                &self.agent.memory
            }

            fn tools(&self) -> &Vec<Tool> {
                &self.agent.tools
            }

            fn knowledge(&self) -> &Knowledge {
                &self.agent.knowledge
            }

            fn planner(&self) -> Option<&Planner> {
                self.agent.planner.as_ref()
            }

            fn profile(&self) -> &Persona {
                &self.agent.profile
            }

            #[cfg(feature = "net")]
            fn collaborators(&self) -> Vec<Collaborator> {
                let mut all = Vec::new();
                all.extend(self.agent.local_collaborators.values().cloned());
                all.extend(self.agent.remote_collaborators.values().cloned());
                all
            }

            fn reflection(&self) -> Option<&Reflection> {
                self.agent.reflection.as_ref()
            }

            fn scheduler(&self) -> Option<&TaskScheduler> {
                self.agent.scheduler.as_ref()
            }

            fn capabilities(&self) -> &std::collections::HashSet<Capability> {
                &self.agent.capabilities
            }

            fn context(&self) -> &ContextManager {
                &self.agent.context
            }

            fn tasks(&self) -> &Vec<Task> {
                &self.agent.tasks
            }

            fn memory_mut(&mut self) -> &mut Vec<Message> {
                &mut self.agent.memory
            }

            fn planner_mut(&mut self) -> Option<&mut Planner> {
                self.agent.planner.as_mut()
            }

            fn context_mut(&mut self) -> &mut ContextManager {
                &mut self.agent.context
            }
        }

        impl Functions for #name {
            fn get_agent(&self) -> &AgentGPT {
                &self.agent
            }
        }

        #[async_trait]
        impl AsyncFunctions for #name {
            async fn execute<'a>(
                &'a mut self,
                task: &'a mut Task,
                execute: bool,
                browse: bool,
                max_tries: u64,
            ) -> Result<()> {
                <#name as Executor>::execute(self, task, execute, browse, max_tries).await
            }

            /// Saves a communication to long-term memory for the agent.
            ///
            /// # Arguments
            ///
            /// * `communication` - The communication to save, which contains the role and content.
            ///
            /// # Returns
            ///
            /// (`Result<()>`): Result indicating the success or failure of saving the communication.
            ///
            /// # Business Logic
            ///
            /// - This method uses the `save_long_term_memory` util function to save the communication into the agent's long-term memory.
            /// - The communication is embedded and stored using the agent's unique ID as the namespace.
            /// - It handles the embedding and metadata for the communication, ensuring it's stored correctly.
            #[cfg(feature = "mem")]
            async fn save_ltm(&mut self, message: Message) -> Result<()> {
                save_long_term_memory(&mut self.client, self.agent.id.clone(), message).await
            }

            /// Retrieves all communications stored in the agent's long-term memory.
            ///
            /// # Returns
            ///
            /// (`Result<Vec<Message>>`): A result containing a vector of communications retrieved from the agent's long-term memory.
            ///
            /// # Business Logic
            ///
            /// - This method fetches the stored communications for the agent by interacting with the `load_long_term_memory` function.
            /// - The function will return a list of communications that are indexed by the agent's unique ID.
            /// - It handles the retrieval of the stored metadata and content for each communication.
            #[cfg(feature = "mem")]
            async fn get_ltm(&self) -> Result<Vec<Message>> {
                load_long_term_memory(self.agent.id.clone()).await
            }

            /// Retrieves the concatenated context of all communications in the agent's long-term memory.
            ///
            /// # Returns
            ///
            /// (`String`): A string containing the concatenated role and content of all communications stored in the agent's long-term memory.
            ///
            /// # Business Logic
            ///
            /// - This method calls the `long_term_memory_context` function to generate a string representation of the agent's entire long-term memory.
            /// - The context string is composed of each communication's role and content, joined by new lines.
            /// - It provides a quick overview of the agent's memory in a human-readable format.
            #[cfg(feature = "mem")]
            async fn ltm_context(&self) -> String {
                long_term_memory_context(self.agent.id.clone()).await
            }

            async fn generate(&mut self, request: &str) -> Result<String> {
                #[cfg(feature = "gem")]
                use gems::{chat::ChatBuilder, messages::Content, traits::CTrait};

                #[cfg(feature = "oai")]
                use openai_dive::v1::{models::Gpt4Model, resources::chat::{ChatMessage, ChatMessageContent, ChatCompletionResponseFormat}};

                #[cfg(feature = "cld")]
                use anthropic_ai_sdk::types::message::{
                    ContentBlock, CreateMessageParams, Message as AnthMessage,
                    MessageClient, RequiredMessageParams, Role,
                };

                #[cfg(feature = "xai")]
                use x_ai::chat_compl::{ChatCompletionsRequestBuilder, Message as XaiMessage};

                #[cfg(feature = "co")]
                use cohere_rust::api::chat::ChatRequest;

                #[cfg(feature = "hf")]
                use ::autogpt::prelude::serde_json::{Value as JsonValue, json};

                match &mut self.client {
                    #[cfg(feature = "gem")]
                    ClientType::Gemini(gem_client) => {
                        let parameters = ChatBuilder::default()
                            .messages(vec![gems::messages::Message::User {
                                content: Content::Text(request.to_string()),
                                name: None,
                            }])
                            .build()?;

                        let result = gem_client.chat().generate(parameters).await;
                        Ok(result.unwrap_or_default())
                    }

                    #[cfg(feature = "oai")]
                    ClientType::OpenAI(oai_client) => {
                        use openai_dive::v1::resources::chat::ChatCompletionParametersBuilder;

                        let parameters = ChatCompletionParametersBuilder::default()
                            .model(Gpt4Model::Gpt4O.to_string())
                            .messages(vec![ChatMessage::User {
                                content: ChatMessageContent::Text(request.to_string()),
                                name: None,
                            }])
                            .response_format(ChatCompletionResponseFormat::Text)
                            .build()?;

                        let result = oai_client.chat().create(parameters).await?;
                        let message = &result.choices[0].message;

                        Ok(match message {
                            ChatMessage::Assistant {
                                content: Some(chat_content),
                                ..
                            } => chat_content.to_string(),
                            ChatMessage::User { content, .. } => content.to_string(),
                            ChatMessage::System { content, .. } => content.to_string(),
                            ChatMessage::Developer { content, .. } => content.to_string(),
                            ChatMessage::Tool { content, .. } => content.to_string(),
                            _ => String::new(),
                        })
                    }

                    #[cfg(feature = "cld")]
                    ClientType::Anthropic(client) => {
                        let body = CreateMessageParams::new(RequiredMessageParams {
                            model: "claude-opus-4-6".to_string(),
                            messages: vec![AnthMessage::new_text(Role::User, request.to_string())],
                            max_tokens: 1024,
                        });

                        let chat_response = client.create_message(Some(&body)).await?;
                        Ok(chat_response
                            .content
                            .iter()
                            .filter_map(|block| match block {
                                ContentBlock::Text { text, .. } => Some(text.as_str()),
                                _ => None,
                            })
                            .collect::<Vec<_>>()
                            .join("\n"))
                    }

                    #[cfg(feature = "xai")]
                    ClientType::Xai(xai_client) => {
                        use x_ai::traits::ChatCompletionsFetcher;

                        let messages = vec![XaiMessage::text("user", request)];

                        let rb = ChatCompletionsRequestBuilder::new(
                            xai_client.clone(),
                            "grok-4".into(),
                            messages,
                        )
                        .temperature(0.0)
                        .stream(false);

                        let req = rb.clone().build()?;
                        let chat = rb.create_chat_completion(req).await?;
                        Ok(chat.choices[0].message.content.to_string())
                    }

                    #[cfg(feature = "co")]
                    ClientType::Cohere(co_client) => {
                        let chat_request = ChatRequest {
                            message: request,
                            ..Default::default()
                        };

                        let mut receiver = match co_client.chat(&chat_request).await {
                            Ok(rx) => rx,
                            Err(e) => return Err(::autogpt::prelude::anyhow!("Cohere API initialization failed: {}", e)),
                        };
                        let mut full_text = String::new();
                        while let Some(res) = receiver.recv().await {
                            match res {
                                Ok(cohere_rust::api::chat::ChatStreamResponse::ChatTextGeneration { text, .. }) => {
                                    full_text.push_str(&text);
                                }
                                Ok(_) => {}
                                // Err(e) => return Err(::autogpt::prelude::anyhow!("Cohere chat error: {:?}", e)),
                                Err(_) => {},
                            }
                        }
                        Ok(full_text)
                    }

                    #[cfg(feature = "hf")]
                    ClientType::HuggingFace(hf_client) => {
                        let model_id = hf_model_from_str(&hf_client.model);
                        let result = hf_client
                            .client
                            .inference()
                            .create(request, model_id)
                            .await
                            .map_err(|e| ::autogpt::prelude::anyhow!("HuggingFace inference failed: {}", e))?;
                        let text = match result {
                            api_huggingface::components::inference_shared::InferenceResponse::Single(output) => output.generated_text,
                            api_huggingface::components::inference_shared::InferenceResponse::Batch(mut batch) => {
                                batch.pop().map(|o| o.generated_text).unwrap_or_default()
                            }
                            _ => String::new(),
                        };
                        Ok(text)
                    }

                    #[allow(unreachable_patterns)]
                    _ => {
                        return Err(::autogpt::prelude::anyhow!(
                            "No valid AI client configured. Enable `hf`, `co`, `gem`, `oai`, `cld`, or `xai` feature."
                        ));
                    }
                }
            }

            async fn imagen(&mut self, request: &str) -> Result<Vec<u8>> {
                #[cfg(feature = "gem")]
                use gems::{imagen::ImageGenBuilder, messages::Content, models::Model, traits::CTrait};

                #[cfg(feature = "hf")]
                use ::autogpt::prelude::serde_json::json;

                match &mut self.client {
                    #[cfg(feature = "gem")]
                    ClientType::Gemini(gem_client) => {
                        gem_client.set_model(Model::Imagen4);

                        let input = gems::messages::Message::User {
                            content: Content::Text(request.into()),
                            name: None,
                        };

                        let params = ImageGenBuilder::default()
                            .model(Model::Imagen4)
                            .input(input)
                            .build()?;

                        let image_bytes = gem_client.images().generate(params).await;
                        Ok(image_bytes.unwrap_or_default())
                    }

                    #[cfg(feature = "oai")]
                    ClientType::OpenAI(oai_client) => {
                        // TODO: Implement this
                        Ok(Default::default())
                    }

                    #[cfg(feature = "cld")]
                    ClientType::Anthropic(client) => {
                        // TODO: Implement this
                        Ok(Default::default())
                    }

                    #[cfg(feature = "xai")]
                    ClientType::Xai(xai_client) => {
                        // TODO: Implement this
                        Ok(Default::default())
                    }

                    #[cfg(feature = "co")]
                    ClientType::Cohere(_co_client) => {
                        // Cohere does not support image generation
                        Ok(Default::default())
                    }

                    #[cfg(feature = "hf")]
                    ClientType::HuggingFace(hf_client) => {
                        let model_id = hf_model_from_str(&hf_client.model);
                        let result = hf_client
                            .client
                            .inference()
                            .create(request, model_id)
                            .await
                            .map_err(|e| ::autogpt::prelude::anyhow!("HuggingFace imagen failed: {}", e))?;
                        let text = match result {
                            api_huggingface::components::inference_shared::InferenceResponse::Single(output) => output.generated_text,
                            api_huggingface::components::inference_shared::InferenceResponse::Batch(mut batch) => {
                                batch.pop().map(|o| o.generated_text).unwrap_or_default()
                            }
                            _ => String::new(),
                        };
                        Ok(text.into_bytes())
                    }

                    #[allow(unreachable_patterns)]
                    _ => {
                        return Err(::autogpt::prelude::anyhow!(
                            "No valid AI client configured. Enable `hf`, `co`, `gem`, `oai`, `cld`, or `xai` feature."
                        ));
                    }
                }
            }

            async fn stream(&mut self, request: &str) -> Result<ReqResponse> {
                #[cfg(feature = "gem")]
                use gems::{messages::Content, models::Model, stream::StreamBuilder, traits::CTrait};

                #[cfg(any(feature = "oai", feature = "cld", feature = "hf"))]
                use futures::StreamExt;

                #[cfg(feature = "oai")]
                use {
                    openai_dive::v1::resources::chat::{
                        ChatCompletionParametersBuilder, ChatMessage, ChatMessageContent,
                    },
                };

                #[cfg(feature = "cld")]
                use {
                    anthropic_ai_sdk::types::message::{
                        ContentBlockDelta, CreateMessageParams, Message as AnthMessage,
                        MessageClient, RequiredMessageParams, Role, StreamEvent,
                    },
                };

                #[cfg(feature = "xai")]
                use {
                    x_ai::chat_compl::{ChatCompletionsRequestBuilder, Message as XaiMessage},
                    x_ai::traits::ClientConfig,
                };

                #[cfg(feature = "hf")]
                use {
                    ::autogpt::prelude::serde_json::{Value as JsonValue, json},
                };

                let request_owned = request.to_string();
                match &mut self.client {
                    #[cfg(feature = "gem")]
                    ClientType::Gemini(gem_client) => {
                        let parameters = StreamBuilder::default()
                            .model(Model::Flash3Preview)
                            .input(gems::messages::Message::User {
                                content: Content::Text(request_owned.clone()),
                                name: None,
                            })
                            .build()?;

                        let resp = gem_client.stream().generate(parameters).await?;
                        let (tx, rx) = tokio::sync::mpsc::channel::<String>(100);

                        tokio::spawn(async move {
                            let mut resp = resp;
                            let mut buffer = String::new();

                            while let Ok(Some(chunk)) = resp.chunk().await {
                                if let Ok(text) = std::str::from_utf8(&chunk) {
                                    buffer.push_str(text);
                                    let mut parts: Vec<&str> =
                                        buffer.split("\n\n").collect();
                                    let new_buffer = if !buffer.ends_with("\n\n") {
                                        parts.pop().unwrap_or("").to_string()
                                    } else {
                                        String::new()
                                    };

                                    for part in parts {
                                        for line in part.lines() {
                                            if let Some(data) =
                                                line.strip_prefix("data: ")
                                            {
                                                let data = data.trim();
                                                if data == "[DONE]" {
                                                    continue;
                                                }
                                                if let Ok(json) =
                                                    ::autogpt::prelude::serde_json::from_str::<::autogpt::prelude::serde_json::Value>(data)
                                                {
                                                    if let Some(text) = json
                                                        .get("candidates")
                                                        .and_then(|c| c.get(0))
                                                        .and_then(|c| c.get("content"))
                                                        .and_then(|c| c.get("parts"))
                                                        .and_then(|p| p.get(0))
                                                        .and_then(|p| p.get("text"))
                                                        .and_then(|t| t.as_str())
                                                    {
                                                        let _ = tx
                                                            .send(text.to_string())
                                                            .await;
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    buffer = new_buffer;
                                }
                            }
                        });

                        Ok(ReqResponse(Some(rx)))
                    }

                    #[cfg(feature = "oai")]
                    ClientType::OpenAI(oai_client) => {
                        let oai_client = oai_client.clone();
                        let request_owned = request_owned.clone();
                        let (tx, rx) = tokio::sync::mpsc::channel::<String>(100);

                        tokio::spawn(async move {
                            let parameters =
                                ChatCompletionParametersBuilder::default()
                                    .model("gpt-5")
                                    .messages(vec![ChatMessage::User {
                                        content: ChatMessageContent::Text(
                                            request_owned,
                                        ),
                                        name: None,
                                    }])
                                    .build()
                                    .unwrap();

                            if let Ok(mut stream) =
                                oai_client.chat().create_stream(parameters).await
                            {
                                while let Some(response) = stream.next().await {
                                    match response {
                                        Ok(chat_response) => {
                                            for choice in chat_response.choices {
                                                let text_opt = match &choice.delta {
                                                    openai_dive::v1::resources::chat::DeltaChatMessage::Assistant {
                                                        content: Some(
                                                            openai_dive::v1::resources::chat::ChatMessageContent::Text(text),
                                                        ),
                                                        ..
                                                    } => Some(text.clone()),
                                                    openai_dive::v1::resources::chat::DeltaChatMessage::Untagged {
                                                        content: Some(
                                                            openai_dive::v1::resources::chat::ChatMessageContent::Text(text),
                                                        ),
                                                        ..
                                                    } => Some(text.clone()),
                                                    _ => None,
                                                };
                                                if let Some(t) = text_opt {
                                                    let _ = tx.send(t).await;
                                                }
                                            }
                                        }
                                        Err(_) => break,
                                    }
                                }
                            }
                        });

                        Ok(ReqResponse(Some(rx)))
                    }

                    #[cfg(feature = "cld")]
                    ClientType::Anthropic(client) => {
                        let client = client.clone();
                        let request_owned = request_owned.clone();
                        let (tx, rx) = tokio::sync::mpsc::channel::<String>(100);

                        tokio::spawn(async move {
                            let body = CreateMessageParams::new(
                                RequiredMessageParams {
                                    model: "claude-opus-4-6".to_string(),
                                    messages: vec![AnthMessage::new_text(
                                        Role::User,
                                        request_owned,
                                    )],
                                    max_tokens: 1024,
                                },
                            )
                            .with_stream(true);

                            if let Ok(mut stream) =
                                client.create_message_streaming(&body).await
                            {
                                while let Some(event_result) = stream.next().await {
                                    if let Ok(StreamEvent::ContentBlockDelta { delta, .. }) = event_result {
                                        if let ContentBlockDelta::TextDelta { text } = delta {
                                            let _ = tx.send(text).await;
                                        }
                                    }
                                }
                            }
                        });

                        Ok(ReqResponse(Some(rx)))
                    }

                    #[cfg(feature = "xai")]
                    ClientType::Xai(xai_client) => {
                        let messages = vec![XaiMessage::text("user", request_owned)];
                        let req = ChatCompletionsRequestBuilder::new(
                            xai_client.clone(),
                            "grok-4".into(),
                            messages,
                        )
                        .stream(true)
                        .build()?;

                        let resp = ClientConfig::request(
                            &*xai_client,
                            reqwest::Method::POST,
                            "chat/completions",
                        )
                        .map_err(|e| {
                            ::autogpt::prelude::anyhow!("Failed to build xAI request: {}", e)
                        })?
                        .json(&req)
                        .send()
                        .await?;

                        let (tx, rx) = tokio::sync::mpsc::channel::<String>(100);

                        tokio::spawn(async move {
                            let mut resp = resp;
                            let mut buffer = String::new();

                            while let Ok(Some(chunk)) = resp.chunk().await {
                                if let Ok(text) = std::str::from_utf8(&chunk) {
                                    buffer.push_str(text);
                                    let mut parts: Vec<&str> =
                                        buffer.split("\n\n").collect();
                                    let new_buffer = if !buffer.ends_with("\n\n") {
                                        parts.pop().unwrap_or("").to_string()
                                    } else {
                                        String::new()
                                    };

                                    for part in parts {
                                        for line in part.lines() {
                                            if let Some(data) =
                                                line.strip_prefix("data: ")
                                            {
                                                let data = data.trim();
                                                if data == "[DONE]" {
                                                    continue;
                                                }
                                                if let Ok(json) =
                                                    ::autogpt::prelude::serde_json::from_str::<::autogpt::prelude::serde_json::Value>(data)
                                                {
                                                    if let Some(content) = json
                                                        .get("choices")
                                                        .and_then(|c| c.get(0))
                                                        .and_then(|c| c.get("delta"))
                                                        .and_then(|d| d.get("content"))
                                                        .and_then(|c| c.as_str())
                                                    {
                                                        let _ = tx
                                                            .send(content.to_string())
                                                            .await;
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    buffer = new_buffer;
                                }
                            }
                        });

                        Ok(ReqResponse(Some(rx)))
                    }

                    #[cfg(feature = "co")]
                    ClientType::Cohere(co_client) => {
                        let chat_request = cohere_rust::api::chat::ChatRequest {
                            message: request,
                            ..Default::default()
                        };
                        let co_result = co_client.chat(&chat_request).await;
                        let (tx, rx) = tokio::sync::mpsc::channel::<String>(100);

                        if let Ok(mut receiver) = co_result {
                            tokio::spawn(async move {
                                while let Some(res) = receiver.recv().await {
                                    if let Ok(resp) = res {
                                        if let cohere_rust::api::chat::ChatStreamResponse::ChatTextGeneration {
                                            text, ..
                                        } = resp
                                        {
                                            let _ = tx.send(text).await;
                                        }
                                    }
                                }
                            });
                        }

                        Ok(ReqResponse(Some(rx)))
                    }

                    #[cfg(feature = "hf")]
                    ClientType::HuggingFace(hf_client) => {
                        let model_id = hf_model_from_str(&hf_client.model);
                        let (tx, rx) = tokio::sync::mpsc::channel::<String>(256);
                        let client = hf_client.client.clone();
                        let model_id_owned = model_id.to_string();
                        let request_owned_clone = request_owned.clone();

                        tokio::spawn(async move {
                            match client
                                .inference()
                                .create(&request_owned_clone, &model_id_owned)
                                .await
                            {
                                Ok(result) => {
                                    use api_huggingface::components::inference_shared::InferenceResponse;
                                    let text = match result {
                                        InferenceResponse::Single(o) => o.generated_text,
                                        InferenceResponse::Batch(mut v) => {
                                            v.pop().map(|o| o.generated_text).unwrap_or_default()
                                        }
                                        _ => String::new(),
                                    };
                                    for word in text.split_whitespace() {
                                        let _ = tx.send(format!("{word} ")).await;
                                    }
                                }
                                Err(e) => {
                                    let _ = tx
                                        .send(format!("[HuggingFace error: {}]", e))
                                        .await;
                                }
                            }
                        });

                        Ok(ReqResponse(Some(rx)))
                    }

                    #[allow(unreachable_patterns)]
                    _ => {
                        return Err(::autogpt::prelude::anyhow!(
                            "No valid AI client configured. \
                             Enable `hf`, `co`, `gem`, `oai`, `cld`, or `xai` feature."
                        ));
                    }
                }
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
