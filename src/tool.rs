//! Tool definition and parsing utilities

use crate::{Error, Provider, Result, StructuredOutput};
use serde::{Deserialize, Serialize};

/// Tool definition for LLM function calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Name of the tool
    pub name: String,

    /// Description of what the tool does
    pub description: String,

    /// JSON Schema for the tool's parameters
    pub parameters: serde_json::Value,
}

/// A tool call made by an LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Unique identifier for this tool call
    pub id: String,

    /// Name of the tool being called
    pub name: String,

    /// Arguments passed to the tool (JSON)
    pub arguments: serde_json::Value,
}

/// Extract tool calls from an API response
///
/// This function parses the raw response text from an LLM API and extracts
/// any tool calls that were made. The format varies by provider.
pub fn extract_tool_calls(response: &str, provider: Provider) -> Result<Vec<ToolCall>> {
    match provider {
        Provider::OpenAI | Provider::Local => extract_openai_tool_calls(response),
        Provider::Anthropic => extract_anthropic_tool_calls(response),
    }
}

fn extract_openai_tool_calls(response: &str) -> Result<Vec<ToolCall>> {
    #[derive(Deserialize)]
    struct OpenAIResponse {
        choices: Vec<OpenAIChoice>,
    }

    #[derive(Deserialize)]
    struct OpenAIChoice {
        message: OpenAIMessage,
    }

    #[derive(Deserialize)]
    struct OpenAIMessage {
        tool_calls: Option<Vec<OpenAIToolCall>>,
    }

    #[derive(Deserialize)]
    struct OpenAIToolCall {
        id: String,
        function: OpenAIFunction,
    }

    #[derive(Deserialize)]
    struct OpenAIFunction {
        name: String,
        arguments: String,
    }

    let parsed: OpenAIResponse = serde_json::from_str(response)?;

    let choice = parsed
        .choices
        .first()
        .ok_or_else(|| Error::InvalidResponseFormat("No choices in response".to_string()))?;

    let tool_calls = match &choice.message.tool_calls {
        Some(calls) => calls
            .iter()
            .map(|tc| {
                let arguments: serde_json::Value =
                    serde_json::from_str(&tc.function.arguments).unwrap_or(serde_json::json!({}));

                ToolCall {
                    id: tc.id.clone(),
                    name: tc.function.name.clone(),
                    arguments,
                }
            })
            .collect(),
        None => return Err(Error::NoToolCalls),
    };

    Ok(tool_calls)
}

fn extract_anthropic_tool_calls(response: &str) -> Result<Vec<ToolCall>> {
    #[derive(Deserialize)]
    struct AnthropicResponse {
        content: Vec<AnthropicContent>,
    }

    #[derive(Deserialize)]
    #[serde(tag = "type")]
    enum AnthropicContent {
        #[serde(rename = "tool_use")]
        ToolUse {
            id: String,
            name: String,
            input: serde_json::Value,
        },
        #[serde(other)]
        Other,
    }

    let parsed: AnthropicResponse = serde_json::from_str(response)?;

    let tool_calls: Vec<ToolCall> = parsed
        .content
        .into_iter()
        .filter_map(|content| match content {
            AnthropicContent::ToolUse { id, name, input } => Some(ToolCall {
                id,
                name,
                arguments: input,
            }),
            AnthropicContent::Other => None,
        })
        .collect();

    if tool_calls.is_empty() {
        return Err(Error::NoToolCalls);
    }

    Ok(tool_calls)
}

/// Parse a tool call response into a structured type
///
/// This validates the tool call arguments against the expected schema
/// and deserializes them into the target type.
pub fn parse_tool_response<T: StructuredOutput>(tool_call: &ToolCall) -> Result<T> {
    // Validate that the tool name matches
    if tool_call.name != T::tool_name() {
        return Err(Error::ToolMismatch(
            tool_call.name.clone(),
            T::tool_name().to_string(),
        ));
    }

    // Validate arguments against the schema
    let schema = T::json_schema();
    crate::schema::validate(&tool_call.arguments, &schema)?;

    // Deserialize the arguments
    let result: T = serde_json::from_value(tool_call.arguments.clone())?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_openai_tool_calls() {
        let response = r#"{
            "choices": [{
                "message": {
                    "tool_calls": [{
                        "id": "call_123",
                        "function": {
                            "name": "final_answer",
                            "arguments": "{\"response\": \"Hello\", \"confidence\": 0.95}"
                        }
                    }]
                }
            }]
        }"#;

        let tool_calls = extract_openai_tool_calls(response).unwrap();
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].id, "call_123");
        assert_eq!(tool_calls[0].name, "final_answer");
        assert_eq!(tool_calls[0].arguments["response"], "Hello");
    }

    #[test]
    fn test_extract_anthropic_tool_calls() {
        let response = r#"{
            "content": [{
                "type": "tool_use",
                "id": "call_123",
                "name": "final_answer",
                "input": {
                    "response": "Hello",
                    "confidence": 0.95
                }
            }]
        }"#;

        let tool_calls = extract_anthropic_tool_calls(response).unwrap();
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].id, "call_123");
        assert_eq!(tool_calls[0].name, "final_answer");
        assert_eq!(tool_calls[0].arguments["response"], "Hello");
    }

    #[test]
    fn test_no_tool_calls() {
        let response = r#"{
            "choices": [{
                "message": {
                    "content": "Just a regular response"
                }
            }]
        }"#;

        let result = extract_openai_tool_calls(response);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::NoToolCalls));
    }
}
