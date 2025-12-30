/// Streaming parser for incremental tool call parsing from SSE responses
///
/// This module enables real-time processing of tool calls as they stream
/// from LLM APIs, without waiting for the complete response.
use crate::{error::Result, Provider, ToolCall};
use serde::Deserialize;

/// Delta updates during streaming tool call construction
#[derive(Debug, Clone)]
pub enum ToolDelta {
    /// Tool call has started
    Start { id: String, name: String },
    /// New arguments data (may be partial JSON)
    Arguments { delta: String },
    /// Tool call is complete
    End,
}

/// Parser for streaming SSE responses
///
/// Handles incremental parsing of tool calls from Server-Sent Events (SSE)
/// streams. Accumulates partial data and emits deltas as they arrive.
///
/// # Example
///
/// ```ignore
/// let mut parser = StreamParser::new(Provider::Anthropic);
///
/// // Process each SSE chunk
/// for chunk in sse_stream {
///     if let Some(delta) = parser.parse_chunk(&chunk)? {
///         match delta {
///             ToolDelta::Start { name, .. } => println!("Starting: {}", name),
///             ToolDelta::Arguments { delta } => print!("{}", delta),
///             ToolDelta::End => println!("\nDone!"),
///         }
///     }
/// }
///
/// // Get the complete tool call
/// let tool_call = parser.finalize()?;
/// ```
pub struct StreamParser {
    provider: Provider,
    state: ParserState,
}

#[derive(Debug, Default)]
struct ParserState {
    current_tool_id: Option<String>,
    current_tool_name: Option<String>,
    accumulated_args: String,
    is_complete: bool,
}

impl StreamParser {
    /// Create a new streaming parser for the specified provider
    pub fn new(provider: Provider) -> Self {
        Self {
            provider,
            state: ParserState::default(),
        }
    }

    /// Parse a single SSE chunk and return any deltas
    ///
    /// Returns `None` if the chunk doesn't contain relevant tool call data.
    /// Returns `Some(ToolDelta)` when tool call events occur.
    pub fn parse_chunk(&mut self, chunk: &str) -> Result<Option<ToolDelta>> {
        match self.provider {
            Provider::OpenAI => self.parse_openai_chunk(chunk),
            Provider::Anthropic => self.parse_anthropic_chunk(chunk),
            Provider::Local => self.parse_local_chunk(chunk),
        }
    }

    /// Get the final complete tool call after streaming is done
    ///
    /// This should be called after all chunks have been processed.
    pub fn finalize(self) -> Result<ToolCall> {
        if !self.state.is_complete {
            return Err(crate::error::Error::InvalidResponseFormat(
                "Tool call stream not completed".to_string(),
            ));
        }

        let id = self
            .state
            .current_tool_id
            .ok_or(crate::error::Error::NoToolCalls)?;
        let name = self
            .state
            .current_tool_name
            .ok_or(crate::error::Error::NoToolCalls)?;

        // Parse accumulated arguments as JSON
        let arguments: serde_json::Value = if self.state.accumulated_args.is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_str(&self.state.accumulated_args)?
        };

        Ok(ToolCall {
            id,
            name,
            arguments,
        })
    }

    fn parse_openai_chunk(&mut self, chunk: &str) -> Result<Option<ToolDelta>> {
        // OpenAI SSE format: "data: {json}\n\n"
        let chunk = chunk.trim();

        if chunk.starts_with("data: [DONE]") {
            if self.state.current_tool_id.is_some() {
                self.state.is_complete = true;
                return Ok(Some(ToolDelta::End));
            }
            return Ok(None);
        }

        if !chunk.starts_with("data: ") {
            return Ok(None);
        }

        let json_str = chunk.strip_prefix("data: ").unwrap_or(chunk);
        let chunk_data: serde_json::Value = serde_json::from_str(json_str)?;

        // Look for tool_calls in delta
        if let Some(choices) = chunk_data["choices"].as_array() {
            if let Some(choice) = choices.first() {
                if let Some(delta) = choice["delta"].as_object() {
                    if let Some(tool_calls) = delta.get("tool_calls") {
                        if let Some(tool_call) = tool_calls.as_array().and_then(|arr| arr.first()) {
                            // Check for tool call start
                            if let Some(id) = tool_call["id"].as_str() {
                                if let Some(name) = tool_call["function"]["name"].as_str() {
                                    self.state.current_tool_id = Some(id.to_string());
                                    self.state.current_tool_name = Some(name.to_string());
                                    return Ok(Some(ToolDelta::Start {
                                        id: id.to_string(),
                                        name: name.to_string(),
                                    }));
                                }
                            }

                            // Check for arguments delta
                            if let Some(args_delta) = tool_call["function"]["arguments"].as_str() {
                                if !args_delta.is_empty() {
                                    self.state.accumulated_args.push_str(args_delta);
                                    return Ok(Some(ToolDelta::Arguments {
                                        delta: args_delta.to_string(),
                                    }));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    fn parse_anthropic_chunk(&mut self, chunk: &str) -> Result<Option<ToolDelta>> {
        // Anthropic SSE format
        let chunk = chunk.trim();

        if !chunk.starts_with("data: ") {
            return Ok(None);
        }

        let json_str = chunk.strip_prefix("data: ").unwrap_or(chunk);
        let event: AnthropicEvent = serde_json::from_str(json_str)?;

        match event.event_type.as_str() {
            "content_block_start" => {
                if let Some(content) = event.content_block {
                    if content.block_type == "tool_use" {
                        let id = content.id.unwrap_or_default();
                        let name = content.name.unwrap_or_default();

                        self.state.current_tool_id = Some(id.clone());
                        self.state.current_tool_name = Some(name.clone());

                        return Ok(Some(ToolDelta::Start { id, name }));
                    }
                }
            }
            "content_block_delta" => {
                if let Some(delta) = event.delta {
                    if delta.delta_type == "input_json_delta" {
                        if let Some(partial_json) = delta.partial_json {
                            self.state.accumulated_args.push_str(&partial_json);
                            return Ok(Some(ToolDelta::Arguments {
                                delta: partial_json,
                            }));
                        }
                    }
                }
            }
            "content_block_stop" => {
                if self.state.current_tool_id.is_some() {
                    self.state.is_complete = true;
                    return Ok(Some(ToolDelta::End));
                }
            }
            "message_stop" => {
                if self.state.current_tool_id.is_some() && !self.state.is_complete {
                    self.state.is_complete = true;
                    return Ok(Some(ToolDelta::End));
                }
            }
            _ => {}
        }

        Ok(None)
    }

    fn parse_local_chunk(&mut self, chunk: &str) -> Result<Option<ToolDelta>> {
        // Local/generic format (similar to OpenAI)
        self.parse_openai_chunk(chunk)
    }
}

#[derive(Debug, Deserialize)]
struct AnthropicEvent {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(default)]
    content_block: Option<ContentBlock>,
    #[serde(default)]
    delta: Option<Delta>,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Delta {
    #[serde(rename = "type")]
    delta_type: String,
    #[serde(default)]
    partial_json: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_streaming() {
        let mut parser = StreamParser::new(Provider::OpenAI);

        // Simulate OpenAI SSE chunks
        let chunks = vec![
            r#"data: {"choices":[{"delta":{"tool_calls":[{"id":"call_123","function":{"name":"test_tool"}}]}}]}"#,
            r#"data: {"choices":[{"delta":{"tool_calls":[{"function":{"arguments":"{"}}]}}]}"#,
            r#"data: {"choices":[{"delta":{"tool_calls":[{"function":{"arguments":"\"key\": \"value\""}}]}}]}"#,
            r#"data: {"choices":[{"delta":{"tool_calls":[{"function":{"arguments":"}"}}]}}]}"#,
            "data: [DONE]",
        ];

        let mut deltas = Vec::new();
        for chunk in chunks {
            if let Some(delta) = parser.parse_chunk(chunk).unwrap() {
                deltas.push(delta);
            }
        }

        // Check we got the expected deltas
        assert!(matches!(deltas[0], ToolDelta::Start { .. }));
        assert!(matches!(deltas[1], ToolDelta::Arguments { .. }));
        assert!(matches!(deltas.last(), Some(ToolDelta::End)));

        // Finalize and check result
        let tool_call = parser.finalize().unwrap();
        assert_eq!(tool_call.name, "test_tool");
        assert_eq!(tool_call.id, "call_123");
    }

    #[test]
    fn test_anthropic_streaming() {
        let mut parser = StreamParser::new(Provider::Anthropic);

        // Simulate Anthropic SSE chunks
        let chunks = vec![
            r#"data: {"type":"content_block_start","content_block":{"type":"tool_use","id":"toolu_123","name":"test_tool"}}"#,
            r#"data: {"type":"content_block_delta","delta":{"type":"input_json_delta","partial_json":"{\"key\": "}}"#,
            r#"data: {"type":"content_block_delta","delta":{"type":"input_json_delta","partial_json":"\"value\"}"}}"#,
            r#"data: {"type":"content_block_stop"}"#,
        ];

        let mut deltas = Vec::new();
        for chunk in chunks {
            if let Some(delta) = parser.parse_chunk(chunk).unwrap() {
                deltas.push(delta);
            }
        }

        // Check we got the expected deltas
        assert!(matches!(deltas[0], ToolDelta::Start { .. }));
        assert!(matches!(deltas[1], ToolDelta::Arguments { .. }));
        assert!(matches!(deltas.last(), Some(ToolDelta::End)));

        // Finalize and check result
        let tool_call = parser.finalize().unwrap();
        assert_eq!(tool_call.name, "test_tool");
        assert_eq!(tool_call.id, "toolu_123");
        assert_eq!(tool_call.arguments["key"], "value");
    }
}
