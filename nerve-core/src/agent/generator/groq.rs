use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use groq_api_rs::completion::{
    client::Groq,
    request::{builder, Function, Tool},
    response::ErrorResponse,
};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::agent::{generator::Message, state::SharedState, Invocation};

use super::{ChatOptions, Client};

lazy_static! {
    static ref RETRY_TIME_PARSER: Regex =
        Regex::new(r"(?m)^.+try again in (.+)\. Visit.*").unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroqFunctionParameterProperty {
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub the_type: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroqFunctionParameters {
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub the_type: String,
    pub required: Vec<String>,
    pub properties: HashMap<String, GroqFunctionParameterProperty>,
}

pub struct GroqClient {
    model: String,
    api_key: String,
}

#[async_trait]
impl Client for GroqClient {
    fn new(_: &str, _: u16, model_name: &str, _: u32) -> Result<Self>
    where
        Self: Sized,
    {
        let api_key = std::env::var("GROQ_API_KEY")
            .map_err(|_| anyhow!("Missing GROQ_API_KEY".to_string()))?;

        let model = model_name.to_string();

        Ok(Self { model, api_key })
    }

    async fn check_native_tools_support(&self) -> Result<bool> {
        let chat_history = vec![
            groq_api_rs::completion::message::Message::SystemMessage {
                role: Some("system".to_string()),
                content: Some("You are an helpful assistant.".to_string()),
                name: None,
                tool_call_id: None,
            },
            groq_api_rs::completion::message::Message::UserMessage {
                role: Some("user".to_string()),
                content: Some("Call the test function.".to_string()),
                name: None,
                tool_call_id: None,
            },
        ];

        let mut properties = HashMap::new();
        properties.insert(
            "payload".to_string(),
            GroqFunctionParameterProperty {
                the_type: "string".to_string(),
                description: "Main function argument.".to_string(),
            },
        );

        let request = builder::RequestBuilder::new(self.model.clone())
            .with_stream(false)
            .with_tools(vec![Tool {
                tool_type: "function".to_string(),
                function: Function {
                    name: Some("test".to_string()),
                    description: Some("This is a test function.".to_string()),
                    parameters: Some(serde_json::json!(GroqFunctionParameters {
                        the_type: "object".to_string(),
                        required: vec!["payload".to_string()],
                        properties,
                    })),
                },
            }]);

        let mut client = Groq::new(&self.api_key);

        client.add_messages(chat_history);

        let resp = client.create(request).await;

        log::debug!("groq.check_tools_support.resp = {:?}", &resp);

        Ok(resp.is_ok())
    }

    async fn chat(
        &self,
        state: SharedState,
        options: &ChatOptions,
    ) -> anyhow::Result<(String, Vec<Invocation>)> {
        let mut chat_history = vec![
            groq_api_rs::completion::message::Message::SystemMessage {
                role: Some("system".to_string()),
                content: Some(options.system_prompt.trim().to_string()),
                name: None,
                tool_call_id: None,
            },
            groq_api_rs::completion::message::Message::UserMessage {
                role: Some("user".to_string()),
                content: Some(options.prompt.trim().to_string()),
                name: None,
                tool_call_id: None,
            },
        ];

        let mut call_idx = 0;

        for m in &options.history {
            chat_history.push(match m {
                Message::Agent(data, invocation) => {
                    let mut tool_call_id = None;
                    if let Some(inv) = invocation {
                        tool_call_id = Some(format!("{}-{}", inv.action, call_idx));
                        call_idx += 1;
                    }

                    groq_api_rs::completion::message::Message::AssistantMessage {
                        role: Some("assistant".to_string()),
                        content: Some(data.trim().to_string()),
                        name: None,
                        tool_call_id,
                        tool_calls: None,
                    }
                }
                Message::Feedback(data, invocation) => {
                    let mut tool_call_id: Option<String> = None;
                    if let Some(inv) = invocation {
                        tool_call_id = Some(format!("{}-{}", inv.action, call_idx));
                    }
                    if tool_call_id.is_some() {
                        groq_api_rs::completion::message::Message::ToolMessage {
                            role: Some("tool".to_string()),
                            content: Some(data.trim().to_string()),
                            name: None,
                            tool_call_id,
                        }
                    } else {
                        groq_api_rs::completion::message::Message::UserMessage {
                            role: Some("user".to_string()),
                            content: Some(data.trim().to_string()),
                            name: None,
                            tool_call_id,
                        }
                    }
                }
            });
        }

        let mut request = builder::RequestBuilder::new(self.model.clone()).with_stream(false);

        if state.lock().await.use_native_tools_format {
            let mut tools = vec![];

            for group in state.lock().await.get_namespaces() {
                for action in &group.actions {
                    let mut required = vec![];
                    let mut properties = HashMap::new();

                    if let Some(example) = action.example_payload() {
                        required.push("payload".to_string());
                        properties.insert(
                            "payload".to_string(),
                            GroqFunctionParameterProperty {
                                the_type: "string".to_string(),
                                description: format!(
                                    "The main function argument, use this as a template: {}",
                                    example
                                ),
                            },
                        );
                    }

                    if let Some(attrs) = action.example_attributes() {
                        for name in attrs.keys() {
                            required.push(name.to_string());
                            properties.insert(
                                name.to_string(),
                                GroqFunctionParameterProperty {
                                    the_type: "string".to_string(),
                                    description: name.to_string(),
                                },
                            );
                        }
                    }

                    let function = Function {
                        name: Some(action.name().to_string()),
                        description: Some(action.description().to_string()),
                        parameters: Some(serde_json::json!(GroqFunctionParameters {
                            the_type: "object".to_string(),
                            required,
                            properties,
                        })),
                    };

                    tools.push(Tool {
                        tool_type: "function".to_string(),
                        function,
                    });
                }
            }

            log::debug!("groq.tools={:?}", &tools);

            request = request.with_tools(tools);
        }

        let mut client = Groq::new(&self.api_key);

        client.add_messages(chat_history);

        let resp = client.create(request).await;
        if let Err(error) = resp {
            if let Some(err_resp) = error.downcast_ref::<ErrorResponse>() {
                // if rate limit exceeded, parse the retry time and retry
                if err_resp.code == 429 {
                    return if self.check_rate_limit(&err_resp.error.message).await {
                        self.chat(state, options).await
                    } else {
                        Err(anyhow!(error))
                    };
                }
            }

            return Err(error);
        }

        let choice = match resp.unwrap() {
            groq_api_rs::completion::client::CompletionOption::NonStream(resp) => {
                resp.choices.first().unwrap().to_owned()
            }
            groq_api_rs::completion::client::CompletionOption::Stream(_) => {
                return Err(anyhow!("Groq streaming is not supported yet, if this happens please open an issue on GitHub"));
            }
        };

        log::debug!("groq.choice.message={:?}", &choice.message);

        let content = choice.message.content.unwrap_or_default().to_string();
        let mut invocations = vec![];

        if let Some(calls) = choice.message.tool_calls {
            for call in calls {
                let mut attributes = HashMap::new();
                let mut payload = None;

                if let Some(args) = call.function.arguments.as_ref() {
                    let map: HashMap<String, serde_json::Value> = serde_json::from_str(args)?;

                    for (name, value) in map {
                        let mut content = value.to_string();
                        if let serde_json::Value::String(escaped_json) = &value {
                            content = escaped_json.to_string();
                        }

                        let str_val = content.trim_matches('"').to_string();
                        if name == "payload" {
                            payload = Some(str_val);
                        } else {
                            attributes.insert(name.to_string(), str_val);
                        }
                    }
                }

                let inv = Invocation {
                    action: call.function.name.unwrap_or_default().to_string(),
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
impl mini_rag::Embedder for GroqClient {
    async fn embed(&self, _text: &str) -> Result<mini_rag::Embeddings> {
        // TODO: extend the rust client to do this
        todo!("groq embeddings generation not yet implemented")
    }
}
