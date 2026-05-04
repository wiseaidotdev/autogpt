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
                        let parameters = ChatCompletionParametersBuilder::default()
                            .model(FlagshipModel::Gpt4O.to_string())
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
                            ChatMessage::Tool { content, .. } => content.clone(),
                            _ => String::new(),
                        })
                    }

                    #[cfg(feature = "cld")]
                    ClientType::Anthropic(client) => {
                        let body = CreateMessageParams::new(RequiredMessageParams {
                            model: "claude-3-7-sonnet-latest".to_string(),
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
                        let messages = vec![XaiMessage {
                            role: "user".into(),
                            content: request.to_string(),
                        }];

                        let rb = ChatCompletionsRequestBuilder::new(
                            xai_client.clone(),
                            "grok-beta".into(),
                            messages,
                        )
                        .temperature(0.0)
                        .stream(false);

                        let req = rb.clone().build()?;
                        let chat = rb.create_chat_completion(req).await?;
                        Ok(chat.choices[0].message.content.clone())
                    }

                    #[cfg(feature = "co")]
                    ClientType::Cohere(co_client) => {
                        use cohere_rust::api::chat::ChatRequest;
                        use cohere_rust::api::GenerateModel;

                        let chat_request = ChatRequest {
                            message: request,
                            ..Default::default()
                        };

                        let mut receiver = match co_client.chat(&chat_request).await {
                            Ok(rx) => rx,
                            Err(e) => return Err(anyhow::anyhow!("Cohere API initialization failed: {}", e)),
                        };
                        let mut full_text = String::new();
                        while let Some(res) = receiver.recv().await {
                            match res {
                                Ok(cohere_rust::api::chat::ChatStreamResponse::ChatTextGeneration { text, .. }) => {
                                    full_text.push_str(&text);
                                }
                                Ok(_) => {}
                                // Err(e) => return Err(anyhow!("Cohere chat error: {:?}", e)),
                                Err(_) => {},
                            }
                        }
                        Ok(full_text)
                    }

                    #[allow(unreachable_patterns)]
                    _ => {
                        return Err(anyhow!(
                            "No valid AI client configured. Enable `co`, `gem`, `oai`, `cld`, or `xai` feature."
                        ));
                    }
                }
            }

            async fn imagen(&mut self, request: &str) -> Result<Vec<u8>> {
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

                    #[allow(unreachable_patterns)]
                    _ => {
                        return Err(anyhow!(
                            "No valid AI client configured. Enable `co`, `gem`, `oai`, `cld`, or `xai` feature."
                        ));
                    }
                }
            }

            async fn stream(&mut self, request: &str) -> Result<ReqResponse> {
                match &mut self.client {
                    #[cfg(feature = "gem")]
                    ClientType::Gemini(gem_client) => {
                        let parameters = StreamBuilder::default()
                            .model(Model::Flash3Preview)
                            .input(gems::messages::Message::User {
                                content: Content::Text(request.into()),
                                name: None,
                            })
                            .build()?;

                        Ok(ReqResponse(Some(gem_client.stream().generate(parameters).await?)))
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
                        // TODO: Implement this
                        Ok(Default::default())
                    }

                    #[allow(unreachable_patterns)]
                    _ => {
                        return Err(anyhow!(
                            "No valid AI client configured. Enable `co`, `gem`, `oai`, `cld`, or `xai` feature."
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
