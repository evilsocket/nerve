pub const DEFAULT_SYSTEM_TEMPLATE: &str = r#"
You are a function calling AI agent with self-recursion.
You can call only one function at a time and analyse data you get from function response.
You have access to the following tools:

{tools}

Don't make assumptions about what values to plug into function arguments.
You must always select one of the above tools and respond with only a JSON object matching the following schema:

{
  "name": <name of the selected tool>,
  "arguments": <parameters for the selected tool, matching the tool's JSON schema>
}
"#;

pub const DEFAULT_RESPONSE_FUNCTION: &str = r#"
{
    "name": "__conversational_response",
    "description": "Respond conversationally if no other tools should be called for a given query.",
    "parameters": {
        "type": "object",
        "properties": {
            "response": {
                "type": "string",
                "description": "Conversational response to the user."
            }
        },
        "required": ["response"]
    }
}
"#;
