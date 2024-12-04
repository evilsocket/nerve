use std::collections::HashMap;

use crate::api::ollama::{
    generation::{
        chat::{
            request::{
                ChatMessageRequest, Tool, ToolFunction, ToolFunctionParameterProperty,
                ToolFunctionParameters,
            },
            ChatMessage,
        },
        options::GenerationOptions,
    },
    Ollama,
};
use anyhow::Result;
use async_trait::async_trait;

use crate::agent::{state::SharedState, Invocation};

use super::{ChatOptions, ChatResponse, Client, Message};

pub struct OllamaClient {
    model: String,
    options: GenerationOptions,
    client: Ollama,
}

#[async_trait]
impl Client for OllamaClient {
    fn new(url: &str, port: u16, model_name: &str, context_window: u32) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut url = url.to_string();
        if !url.contains("://") {
            // ollama-rs is picky about this and wants the schema
            url = format!("http://{url}");
        }

        let client = Ollama::new(url.to_string(), port);
        let model = model_name.to_string();
        // Do not provide model options other than the context window size so that we'll use whatever was
        // specified in the modelfile.
        let options = GenerationOptions::default().num_ctx(context_window);

        Ok(Self {
            model,
            client,
            options,
        })
    }

    async fn check_native_tools_support(&self) -> Result<bool> {
        let chat_history = vec![
            ChatMessage::system("You are an helpful assistant.".to_string()),
            ChatMessage::user("Call the test function.".to_string()),
        ];

        let mut request = ChatMessageRequest::new(self.model.to_string(), chat_history)
            // Do not provide model options other than the context window size so that we'll use whatever was
            // specified in the modelfile.
            .options(self.options.clone())
            // Set tools ( https://ollama.com/blog/tool-support )
            .tools(vec![Tool {
                the_type: "function".to_string(),
                function: ToolFunction {
                    name: "test".to_string(),
                    description: "This is a test function.".to_string(),
                    parameters: ToolFunctionParameters {
                        the_type: "object".to_string(),
                        required: vec![],
                        properties: HashMap::new(),
                    },
                },
            }]);

        request.model_name.clone_from(&self.model);

        if let Err(err) = self.client.send_chat_messages(request).await {
            if err.to_string().contains("does not support tools") {
                Ok(false)
            } else {
                Err(anyhow!(err))
            }
        } else {
            Ok(true)
        }
    }

    async fn chat(
        &self,
        state: SharedState,
        options: &ChatOptions,
    ) -> anyhow::Result<ChatResponse> {
        // TODO: images for multimodal (see todo for screenshot action)

        // build chat history:
        //    - system prompt
        //    - user prompt
        //    - msg 0
        //    - msg 1
        //    - ...
        //    - msg n
        let mut chat_history = vec![
            ChatMessage::system(options.system_prompt.trim().to_string()),
            ChatMessage::user(options.prompt.to_string()),
        ];

        for m in options.history.iter() {
            chat_history.push(match m {
                Message::Agent(data, _) => ChatMessage::assistant(data.trim().to_string()),
                Message::Feedback(data, _) => ChatMessage::user(data.trim().to_string()),
            });
        }

        // Generate tools vector.
        let mut tools = vec![];

        if state.lock().await.use_native_tools_format {
            for group in state.lock().await.get_namespaces() {
                for action in &group.actions {
                    let mut required = vec![];
                    let mut properties = HashMap::new();

                    if let Some(example) = action.example_payload() {
                        required.push("payload".to_string());
                        properties.insert(
                            "payload".to_string(),
                            ToolFunctionParameterProperty {
                                the_type: "string".to_string(),
                                description: format!(
                                    "The main function argument, use this as a template: {}",
                                    example
                                ),
                                an_enum: None,
                            },
                        );
                    }

                    if let Some(attrs) = action.example_attributes() {
                        for name in attrs.keys() {
                            required.push(name.to_string());
                            properties.insert(
                                name.to_string(),
                                ToolFunctionParameterProperty {
                                    the_type: "string".to_string(),
                                    description: name.to_string(),
                                    an_enum: None,
                                },
                            );
                        }
                    }

                    let function = ToolFunction {
                        name: action.name().to_string(),
                        description: action.description().to_string(),
                        parameters: ToolFunctionParameters {
                            the_type: "object".to_string(),
                            required,
                            properties,
                        },
                    };

                    tools.push(Tool {
                        the_type: "function".to_string(),
                        function,
                    });
                }
            }

            log::trace!("ollama.tools={:?}", &tools);
        }

        let mut request = ChatMessageRequest::new(self.model.to_string(), chat_history)
            // Do not provide model options other than the context window size so that we'll use whatever was
            // specified in the modelfile.
            .options(self.options.clone())
            // Set tools ( https://ollama.com/blog/tool-support )
            .tools(tools);

        request.model_name.clone_from(&self.model);

        let res = self.client.send_chat_messages(request).await?;

        if let Some(msg) = res.message {
            let content = msg.content.to_owned();
            let mut invocations = vec![];

            if let Some(tool_calls) = msg.tool_calls.as_ref() {
                log::debug!("ollama.tool.calls = {:?}", tool_calls);

                for call in tool_calls {
                    let mut attributes = HashMap::new();
                    let mut payload = None;

                    if let Some(args) = call.function.arguments.as_ref() {
                        for (name, value) in args {
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

            log::debug!("ollama.invocations = {:?}", &invocations);
            Ok(ChatResponse {
                content,
                invocations,
                usage: res.final_data.map(|final_data| super::Usage {
                        input_tokens: final_data.prompt_eval_count as u32,
                        output_tokens: final_data.eval_count as u32,
                    }),
            })
        } else {
            log::warn!("model returned an empty message.");
            Ok(ChatResponse {
                content: "".to_string(),
                invocations: vec![],
                usage: res.final_data.map(|final_data| super::Usage {
                        input_tokens: final_data.prompt_eval_count as u32,
                        output_tokens: final_data.eval_count as u32,
                    }),
            })
        }
    }
}

#[async_trait]
impl mini_rag::Embedder for OllamaClient {
    async fn embed(&self, text: &str) -> Result<mini_rag::Embeddings> {
        let resp = self
            .client
            .generate_embeddings(self.model.to_string(), text.to_string(), None)
            .await?;

        Ok(mini_rag::Embeddings::from(resp.embeddings))
    }
}
