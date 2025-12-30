//! Provider-specific adapters for different LLM APIs

/// Supported LLM API providers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    /// OpenAI API (chat.openai.com)
    OpenAI,

    /// Anthropic API (claude.ai)
    Anthropic,

    /// Local or generic OpenAI-compatible API
    Local,
}

impl Provider {
    /// Returns the expected format for tool definitions
    pub fn tool_format(&self) -> ToolFormat {
        match self {
            Provider::OpenAI | Provider::Local => ToolFormat::OpenAI,
            Provider::Anthropic => ToolFormat::Anthropic,
        }
    }
}

/// Different tool definition formats used by providers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolFormat {
    /// OpenAI format: { "type": "function", "function": { "name": "...", "parameters": {...} } }
    OpenAI,

    /// Anthropic format: { "name": "...", "description": "...", "input_schema": {...} }
    Anthropic,
}

/// Helper to build request bodies with tools for different providers
pub fn build_request_with_tools(
    messages: &[crate::Message],
    tools: &[crate::ToolDefinition],
    provider: Provider,
) -> serde_json::Value {
    match provider {
        Provider::OpenAI | Provider::Local => build_openai_request(messages, tools),
        Provider::Anthropic => build_anthropic_request(messages, tools),
    }
}

/// Build a request that enforces a specific tool call (like pydantic AI / luagent pattern)
///
/// This is the recommended approach for structured outputs - it guarantees the LLM
/// will call the specified tool, ensuring you always get back your structured data.
///
/// # Example
///
/// ```ignore
/// use struct_llm::{build_enforced_tool_request, Provider, StructuredOutput};
///
/// let tool = MyOutput::tool_definition();
/// let request = build_enforced_tool_request(
///     &messages,
///     &tool,
///     Provider::OpenAI
/// );
/// // The LLM will be forced to call MyOutput's tool
/// ```
pub fn build_enforced_tool_request(
    messages: &[crate::Message],
    tool: &crate::ToolDefinition,
    provider: Provider,
) -> serde_json::Value {
    match provider {
        Provider::OpenAI | Provider::Local => {
            let mut request = build_openai_request(messages, &[tool.clone()]);
            // Force this specific tool to be called
            request["tool_choice"] = serde_json::json!({
                "type": "function",
                "function": {
                    "name": tool.name
                }
            });
            request
        }
        Provider::Anthropic => {
            let mut request = build_anthropic_request(messages, &[tool.clone()]);
            // Force this specific tool to be called
            request["tool_choice"] = serde_json::json!({
                "type": "tool",
                "name": tool.name
            });
            request
        }
    }
}

fn build_openai_request(
    messages: &[crate::Message],
    tools: &[crate::ToolDefinition],
) -> serde_json::Value {
    let mut request = serde_json::json!({
        "messages": messages,
    });

    if !tools.is_empty() {
        let formatted_tools: Vec<_> = tools
            .iter()
            .map(|tool| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": tool.name,
                        "description": tool.description,
                        "parameters": tool.parameters,
                    }
                })
            })
            .collect();

        request["tools"] = serde_json::json!(formatted_tools);
    }

    request
}

fn build_anthropic_request(
    messages: &[crate::Message],
    tools: &[crate::ToolDefinition],
) -> serde_json::Value {
    let mut request = serde_json::json!({
        "messages": messages,
    });

    if !tools.is_empty() {
        let formatted_tools: Vec<_> = tools
            .iter()
            .map(|tool| {
                serde_json::json!({
                    "name": tool.name,
                    "description": tool.description,
                    "input_schema": tool.parameters,
                })
            })
            .collect();

        request["tools"] = serde_json::json!(formatted_tools);
    }

    request
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_tool_format() {
        assert_eq!(Provider::OpenAI.tool_format(), ToolFormat::OpenAI);
        assert_eq!(Provider::Anthropic.tool_format(), ToolFormat::Anthropic);
        assert_eq!(Provider::Local.tool_format(), ToolFormat::OpenAI);
    }
}
