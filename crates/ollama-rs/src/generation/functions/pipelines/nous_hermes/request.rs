use crate::error::OllamaError;
use crate::generation::chat::{ChatMessage, ChatMessageResponse};
use crate::generation::functions::pipelines::nous_hermes::DEFAULT_SYSTEM_TEMPLATE;
use crate::generation::functions::pipelines::RequestParserBase;
use crate::generation::functions::tools::Tool;
use async_trait::async_trait;
use regex::Regex;
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
pub struct NousFunctionCallSignature {
    pub name: String,
    pub arguments: Value,
}

pub struct NousFunctionCall {}

impl Default for NousFunctionCall {
    fn default() -> Self {
        Self::new()
    }
}

impl NousFunctionCall {
    pub fn new() -> Self {
        Self {}
    }

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
                message: Some(ChatMessage::assistant(self.format_tool_response(&result))),
                done: true,
                final_data: None,
            }),
            Err(e) => Err(self.error_handler(OllamaError::from(e))),
        }
    }

    pub fn format_tool_response(&self, function_response: &str) -> String {
        format!("<tool_response>\n{}\n</tool_response>\n", function_response)
    }

    pub fn extract_tool_call(&self, content: &str) -> Option<String> {
        let re = Regex::new(r"(?s)<tool_call>(.*?)</tool_call>").unwrap();
        if let Some(captures) = re.captures(content) {
            if let Some(matched) = captures.get(1) {
                let result = matched
                    .as_str()
                    .replace('\n', "")
                    .replace("{{", "{")
                    .replace("}}", "}");
                return Some(result);
            }
        }
        None
    }
}

#[async_trait]
impl RequestParserBase for NousFunctionCall {
    async fn parse(
        &self,
        input: &str,
        model_name: String,
        tools: Vec<Arc<dyn Tool>>,
    ) -> Result<ChatMessageResponse, ChatMessageResponse> {
        //Extract between <tool_call> and </tool_call>
        let tool_response = self.extract_tool_call(input);
        match tool_response {
            Some(tool_response_str) => {
                let response_value: Result<NousFunctionCallSignature, serde_json::Error> =
                    serde_json::from_str(&tool_response_str);
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
                                .await?; //Error is also returned as String for LLM feedback
                            return Ok(result);
                        } else {
                            return Err(self.error_handler(OllamaError::from(
                                "Tool name not found".to_string(),
                            )));
                        }
                    }
                    Err(e) => return Err(self.error_handler(OllamaError::from(e))),
                }
            }
            None => {
                return Err(self.error_handler(OllamaError::from(
                    "Error while extracting <tool_call> tags.".to_string(),
                )))
            }
        }
    }

    fn format_query(&self, input: &str) -> String {
        format!(
            "{}\nThis is the first turn and you don't have <tool_results> to analyze yet",
            input
        )
    }

    fn format_response(&self, response: &str) -> String {
        format!("Agent iteration to assist with user query: {}", response)
    }

    async fn get_system_message(&self, tools: &[Arc<dyn Tool>]) -> ChatMessage {
        let tools_info: Vec<Value> = tools.iter().map(convert_to_openai_tool).collect();
        let tools_json = serde_json::to_string(&tools_info).unwrap();
        let system_message_content = DEFAULT_SYSTEM_TEMPLATE.replace("{tools}", &tools_json);
        ChatMessage::system(system_message_content)
    }

    fn error_handler(&self, error: OllamaError) -> ChatMessageResponse {
        let error_message = format!(
            "<tool_response>\nThere was an error parsing function calls\n Here's the error stack trace: {}\nPlease call the function again with correct syntax</tool_response>",
            error
        );

        ChatMessageResponse {
            model: "".to_string(),
            created_at: "".to_string(),
            message: Some(ChatMessage::assistant(error_message)),
            done: true,
            final_data: None,
        }
    }
}
