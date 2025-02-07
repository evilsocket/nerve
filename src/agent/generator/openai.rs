use std::collections::HashMap;

use crate::api::openai::*;
use crate::{agent::namespaces::ToolOutput, api::openai::chat::*};
use anyhow::Result;
use async_trait::async_trait;
use embeddings::EmbeddingsApi;
use serde::{Deserialize, Serialize};

use crate::agent::{state::SharedState, ToolCall};

use super::{ChatOptions, ChatResponse, Client, Message, SupportedFeatures};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiToolFunctionParameterProperty {
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub the_type: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiToolFunctionParameters {
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub the_type: String,
    pub required: Vec<String>,
    pub properties: HashMap<String, OpenAiToolFunctionParameterProperty>,
}

pub struct OpenAIClient {
    ident: String,
    model: String,
    client: OpenAI,
}

impl OpenAIClient {
    pub fn custom(model: &str, api_key_env: &str, endpoint: &str) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let model = model.to_string();
        let api_key = std::env::var(api_key_env).map_err(|_| anyhow!("Missing {api_key_env}"))?;
        let ident = api_key_env
            .split('_')
            .next()
            .unwrap_or("openai")
            .to_string();
        let auth = Auth::new(&api_key);
        let client = OpenAI::new(auth, endpoint);

        Ok(Self {
            ident,
            model,
            client,
        })
    }

    pub fn custom_no_auth(model: &str, endpoint: &str) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let ident = "http".to_string();
        let model = model.to_string();
        let auth = Auth::new("");
        let client = OpenAI::new(auth, endpoint);

        Ok(Self {
            ident,
            model,
            client,
        })
    }

    async fn get_tools_if_supported(&self, state: &SharedState) -> Vec<FunctionTool> {
        let mut tools = vec![];

        // if native tool calls are supported (and XML was not forced)
        if state.lock().await.use_native_tools_format {
            // for every namespace available to the model
            for group in state.lock().await.get_namespaces() {
                // for every tool of the namespace
                for tool in &group.tools {
                    let mut required = vec![];
                    let mut properties = HashMap::new();

                    if let Some(example) = tool.example_payload() {
                        required.push("payload".to_string());
                        properties.insert(
                            "payload".to_string(),
                            OpenAiToolFunctionParameterProperty {
                                the_type: "string".to_string(),
                                description: format!(
                                    "The main function argument, use this as a template: {}",
                                    example
                                ),
                            },
                        );
                    }

                    if let Some(attrs) = tool.example_attributes() {
                        for name in attrs.keys() {
                            required.push(name.to_string());
                            properties.insert(
                                name.to_string(),
                                OpenAiToolFunctionParameterProperty {
                                    the_type: "string".to_string(),
                                    description: name.to_string(),
                                },
                            );
                        }
                    }

                    let function = FunctionDefinition {
                        name: tool.name().to_string(),
                        description: Some(tool.description().to_string()),
                        parameters: Some(serde_json::json!(OpenAiToolFunctionParameters {
                            the_type: "object".to_string(),
                            required,
                            properties,
                        })),
                    };

                    tools.push(FunctionTool {
                        the_type: "function".to_string(),
                        function,
                    });
                }
            }

            log::trace!("openai.tools={:?}", &tools);

            // let j = serde_json::to_string_pretty(&tools).unwrap();
            // log::info!("{j}");
        }

        tools
    }
}

#[async_trait]
impl Client for OpenAIClient {
    fn new(_: &str, _: u16, model_name: &str, _: u32) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Self::custom(model_name, "OPENAI_API_KEY", "https://api.openai.com/v1/")
    }

    async fn check_supported_features(&self) -> Result<SupportedFeatures> {
        let chat_history = vec![
            crate::api::openai::Message::text("You are an helpful assistant.", Role::System),
            crate::api::openai::Message::text("Execute the test function.", Role::User),
        ];

        let tools = Some(vec![FunctionTool {
            the_type: "function".to_string(),
            function: FunctionDefinition {
                name: "test".to_string(),
                description: Some("This is a test function.".to_string()),
                parameters: Some(serde_json::json!(HashMap::<String, String>::new())),
            },
        }]);

        let body = ChatBody {
            model: self.model.to_string(),
            max_tokens: None,
            temperature: None,
            top_p: None,
            n: None,
            stream: Some(false),
            stop: None,
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            user: None,
            messages: chat_history,
            tools,
        };
        let resp = self.client.chat_completion_create(&body);

        log::debug!("openai.check_tools_support.resp = {:?}", &resp);

        let mut system_prompt_support = true;

        if let Ok(comp) = resp {
            if !comp.choices.is_empty() {
                let first = comp.choices.first().unwrap();
                if let Some(m) = first.message.as_ref() {
                    if let Some(calls) = m.tool_calls.as_ref() {
                        if !calls.is_empty() {
                            log::debug!("found tool_calls: {:?}", calls);
                            return Ok(SupportedFeatures {
                                system_prompt: true,
                                tools: true,
                            });
                        }
                    }
                }
            }
        } else {
            let api_error = resp.unwrap_err().to_string();
            if api_error.contains("unsupported_value")
                && api_error.contains("does not support 'system' with this model")
            {
                system_prompt_support = false;
            } else if self.ident == "GEMINI" && api_error.contains("INVALID_ARGUMENT") {
                // Gemini openai endpoint breaks with multiple tools:
                //
                //  https://discuss.ai.google.dev/t/invalid-argument-error-using-openai-compatible/51788
                //  https://discuss.ai.google.dev/t/gemini-openai-compatibility-multiple-functions-support-in-function-calling-error-400/49431
                //
                // Never can overcome this bug by providing its own xml based tooling prompt.
                log::warn!("this is a documented bug of Google Gemini OpenAI endpoint: https://discuss.ai.google.dev/t/invalid-argument-error-using-openai-compatible/51788");
            } else {
                log::error!("openai.check_tools_support.error = {}", api_error);
            }
        }

        Ok(SupportedFeatures {
            system_prompt: system_prompt_support,
            tools: false,
        })
    }

    async fn chat(
        &self,
        state: SharedState,
        options: &ChatOptions,
    ) -> anyhow::Result<ChatResponse> {
        let mut chat_history = match &options.system_prompt {
            Some(sp) => vec![
                crate::api::openai::Message::text(sp.trim(), Role::System),
                crate::api::openai::Message::text(options.prompt.trim(), Role::User),
            ],
            None => vec![crate::api::openai::Message::text(
                options.prompt.trim(),
                Role::User,
            )],
        };

        for m in options.history.iter() {
            chat_history.push(match m {
                Message::Agent {
                    content,
                    tool_call: _,
                } => crate::api::openai::Message::text(content.trim(), Role::Assistant),
                Message::Feedback {
                    result,
                    tool_call: _,
                } => match result {
                    ToolOutput::Text(text) => {
                        // handles string_too_short cases (NIM)
                        let mut content = text.trim().to_string();
                        if content.is_empty() {
                            content = "<no output>".to_string();
                        }
                        crate::api::openai::Message::text(&content, Role::User)
                    }
                    ToolOutput::Image { data, mime_type } => {
                        crate::api::openai::Message::image(data, mime_type, Role::User)
                    }
                },
            });
        }

        let tools = self.get_tools_if_supported(&state).await;

        let body = ChatBody {
            model: self.model.to_string(),
            max_tokens: None,
            temperature: None,
            top_p: None,
            n: None,
            stream: Some(false),
            stop: None,
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            user: None,
            messages: chat_history,
            tools: if tools.is_empty() { None } else { Some(tools) },
        };
        let resp = self.client.chat_completion_create(&body);

        if let Err(error) = resp {
            return if self.check_rate_limit(&error.to_string()).await {
                self.chat(state, options).await
            } else {
                Err(anyhow!(error))
            };
        }

        let resp = resp.unwrap();
        let choice = resp.choices.first().unwrap();
        let (content, tool_calls) = if let Some(m) = &choice.message {
            (
                m.content.clone().unwrap_or_default().to_string(),
                m.tool_calls.clone(),
            )
        } else {
            ("".to_string(), None)
        };

        let mut resolved_tool_calls = vec![];

        log::debug!("openai.tool_calls={:?}", &tool_calls);

        if let Some(calls) = tool_calls {
            for call in calls {
                let mut attributes = HashMap::new();
                let mut argument = None;

                let map: HashMap<String, serde_json::Value> =
                    serde_json::from_str(&call.function.arguments).map_err(|e| {
                        log::error!(
                            "failed to parse tool call arguments: {e} - {}",
                            call.function.arguments
                        );
                        anyhow!(e)
                    })?;
                for (name, value) in map {
                    log::debug!("openai.tool_call.arg={} = {:?}", name, value);

                    let mut content = value.to_string();
                    if let serde_json::Value::String(escaped_json) = &value {
                        content = escaped_json.to_string();
                    }

                    let str_val = content.trim_matches('"').to_string();
                    if name == "payload" {
                        argument = Some(str_val);
                    } else {
                        attributes.insert(name.to_string(), str_val);
                    }
                }

                resolved_tool_calls.push(ToolCall {
                    tool_name: call.function.name.to_string(),
                    named_arguments: if attributes.is_empty() {
                        None
                    } else {
                        Some(attributes)
                    },
                    argument,
                });
            }
        }

        Ok(ChatResponse {
            content: content.to_string(),
            tool_calls: resolved_tool_calls,
            usage: match resp.usage.prompt_tokens {
                Some(prompt_tokens) => Some(super::Usage {
                    input_tokens: prompt_tokens,
                    output_tokens: resp.usage.completion_tokens.unwrap_or(0),
                }),
                None => None,
            },
        })
    }
}

#[async_trait]
impl mini_rag::Embedder for OpenAIClient {
    async fn embed(&self, text: &str) -> Result<mini_rag::Embeddings> {
        let body = embeddings::EmbeddingsBody {
            model: self.model.to_string(),
            input: vec![text.to_string()],
            user: None,
        };
        let resp = self.client.embeddings_create(&body);
        if let Err(error) = resp {
            return if self.check_rate_limit(&error.to_string()).await {
                self.embed(text).await
            } else {
                Err(anyhow!(error))
            };
        }

        let embeddings = resp.unwrap().data;
        let embedding = embeddings.as_ref().unwrap().first().unwrap();

        Ok(mini_rag::Embeddings::from(
            embedding.embedding.as_ref().unwrap_or(&vec![]).clone(),
        ))
    }
}
