use crate::error::OllamaError;
use crate::generation::chat::{ChatMessage, ChatMessageResponse};
use crate::generation::functions::pipelines::openai::DEFAULT_SYSTEM_TEMPLATE;
use crate::generation::functions::pipelines::RequestParserBase;
use crate::generation::functions::tools::Tool;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::sync::Arc;

pub fn convert_to_openai_tool(tool: &Arc<dyn Tool>) -> Value {
    let mut function = HashMap::new();
    function.insert("name".to_string(), Value::String(tool.name()));
    function.insert("description".to_string(), Value::String(tool.description()));
    function.insert("parameters".to_string(), tool.parameters());

    let mut result = HashMap::new();
    result.insert("type".to_string(), Value::String("function".to_string()));

    let mapp: Map<String, Value> = function.into_iter().collect();
    result.insert("function".to_string(), Value::Object(mapp));

    json!(result)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OpenAIFunctionCallSignature {
    pub name: String, //name of the tool
    pub arguments: Value,
}

pub struct OpenAIFunctionCall {}

impl OpenAIFunctionCall {
    pub async fn function_call_with_history(
        &self,
        model_name: String,
        tool_params: Value,
        tool: Arc<dyn Tool>,
    ) -> Result<ChatMessageResponse, ChatMessageResponse> {
        let result = tool.run(tool_params).await;
        match result {
            Ok(result) => Ok(ChatMessageResponse {
                model: model_name.clone(),
                created_at: "".to_string(),
                message: Some(ChatMessage::assistant(result.to_string())),
                done: true,
                final_data: None,
            }),
            Err(e) => Err(self.error_handler(OllamaError::from(e))),
        }
    }

    fn clean_tool_call(&self, json_str: &str) -> String {
        json_str
            .trim()
            .trim_start_matches("```json")
            .trim_end_matches("```")
            .trim()
            .to_string()
    }
}

#[async_trait]
impl RequestParserBase for OpenAIFunctionCall {
    async fn parse(
        &self,
        input: &str,
        model_name: String,
        tools: Vec<Arc<dyn Tool>>,
    ) -> Result<ChatMessageResponse, ChatMessageResponse> {
        let response_value: Result<OpenAIFunctionCallSignature, serde_json::Error> =
            serde_json::from_str(&self.clean_tool_call(input));
        match response_value {
            Ok(response) => {
                if let Some(tool) = tools.iter().find(|t| t.name() == response.name) {
                    let tool_params = response.arguments;
                    let result = self
                        .function_call_with_history(
                            model_name.clone(),
                            tool_params.clone(),
                            tool.clone(),
                        )
                        .await?;
                    return Ok(result);
                } else {
                    return Err(self.error_handler(OllamaError::from("Tool not found".to_string())));
                }
            }
            Err(e) => {
                return Err(self.error_handler(OllamaError::from(e)));
            }
        }
    }

    async fn get_system_message(&self, tools: &[Arc<dyn Tool>]) -> ChatMessage {
        let tools_info: Vec<Value> = tools.iter().map(convert_to_openai_tool).collect();
        let tools_json = serde_json::to_string(&tools_info).unwrap();
        let system_message_content = DEFAULT_SYSTEM_TEMPLATE.replace("{tools}", &tools_json);
        ChatMessage::system(system_message_content)
    }

    fn error_handler(&self, error: OllamaError) -> ChatMessageResponse {
        ChatMessageResponse {
            model: "".to_string(),
            created_at: "".to_string(),
            message: Some(ChatMessage::assistant(error.to_string())),
            done: true,
            final_data: None,
        }
    }
}
