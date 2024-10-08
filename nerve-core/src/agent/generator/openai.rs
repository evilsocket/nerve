use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use openai_api_rust::chat::*;
use openai_api_rust::embeddings::EmbeddingsApi;
use openai_api_rust::*;
use serde::{Deserialize, Serialize};

use crate::agent::{state::SharedState, Invocation};

use super::{ChatOptions, Client, Message};

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
        let auth = Auth::new(&api_key);
        let client = OpenAI::new(auth, endpoint);

        Ok(Self { model, client })
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

    async fn check_tools_support(&self) -> Result<bool> {
        let chat_history = vec![
            openai_api_rust::Message {
                role: Role::System,
                content: Some("You are an helpful assistant.".to_string()),
                tool_calls: None,
            },
            openai_api_rust::Message {
                role: Role::User,
                content: Some("Call the test function.".to_string()),
                tool_calls: None,
            },
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

        if let Ok(comp) = resp {
            if !comp.choices.is_empty() {
                let first = comp.choices.first().unwrap();
                if let Some(m) = first.message.as_ref() {
                    if m.tool_calls.is_some() {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    async fn chat(
        &self,
        state: SharedState,
        options: &ChatOptions,
    ) -> anyhow::Result<(String, Vec<Invocation>)> {
        let mut chat_history = vec![
            openai_api_rust::Message {
                role: Role::System,
                content: Some(options.system_prompt.trim().to_string()),
                tool_calls: None,
            },
            openai_api_rust::Message {
                role: Role::User,
                content: Some(options.prompt.trim().to_string()),
                tool_calls: None,
            },
        ];

        for m in &options.history {
            chat_history.push(match m {
                Message::Agent(data, _) => openai_api_rust::Message {
                    role: Role::Assistant,
                    content: Some(data.trim().to_string()),
                    tool_calls: None,
                },
                Message::Feedback(data, _) => openai_api_rust::Message {
                    role: Role::User,
                    content: Some(data.trim().to_string()),
                    tool_calls: None,
                },
            });
        }

        let mut tools = vec![];
        if state.lock().await.tools_support {
            for group in state.lock().await.get_namespaces() {
                for action in &group.actions {
                    let mut required = vec![];
                    let mut properties = HashMap::new();

                    if action.example_payload().is_some() {
                        required.push("payload".to_string());
                        properties.insert(
                            "payload".to_string(),
                            OpenAiToolFunctionParameterProperty {
                                the_type: "string".to_string(),
                                description: "Main function argument.".to_string(),
                            },
                        );
                    }

                    if let Some(attrs) = action.example_attributes() {
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
                        name: action.name().to_string(),
                        description: Some(action.description().to_string()),
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

            log::debug!("openai.tools={:?}", &tools);

            // let j = serde_json::to_string_pretty(&tools).unwrap();
            // log::info!("{j}");
        }

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

        let mut invocations = vec![];

        if let Some(calls) = tool_calls {
            for call in calls {
                let mut attributes = HashMap::new();
                let mut payload = None;

                let map: HashMap<String, String> = serde_json::from_str(&call.function.arguments)?;
                for (name, value) in map {
                    let str_val = value.to_string().trim_matches('"').to_string();
                    if name == "payload" {
                        payload = Some(str_val);
                    } else {
                        attributes.insert(name.to_string(), str_val);
                    }
                }

                let inv = Invocation {
                    action: call.function.name.to_string(),
                    attributes: if attributes.is_empty() {
                        None
                    } else {
                        Some(attributes)
                    },
                    payload,
                };

                invocations.push(inv);
            }
        }

        Ok((content, invocations))
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
