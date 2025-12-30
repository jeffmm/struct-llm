//! # struct-llm
//!
//! A lightweight, WASM-compatible Rust library for generating structured outputs from LLMs
//! using a tool-based approach.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use struct_llm::StructuredOutput;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Serialize, Deserialize, StructuredOutput)]
//! #[structured_output(
//!     name = "sentiment_analysis",
//!     description = "Analyzes the sentiment of the given text"
//! )]
//! struct SentimentAnalysis {
//!     sentiment: String,
//!     confidence: f32,
//!     reasoning: String,
//! }
//!
//! // Generate tool definition
//! let tool = SentimentAnalysis::tool_definition();
//!
//! // Make API request with your HTTP client
//! let response = make_api_request_with_tools(&prompt, &[tool]).await?;
//!
//! // Extract and validate structured response
//! let tool_calls = extract_tool_calls(&response, Provider::OpenAI)?;
//! let result: SentimentAnalysis = parse_tool_response(&tool_calls[0])?;
//! ```

use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub mod error;
pub mod provider;
pub mod schema;
pub mod tool;

pub use error::{Error, Result};
pub use provider::Provider;
pub use tool::{extract_tool_calls, parse_tool_response, ToolCall, ToolDefinition};

/// Core trait for types that can be used as structured LLM outputs.
///
/// This trait enables a type to be used as a structured output from an LLM by:
/// 1. Generating a JSON Schema describing the type's structure
/// 2. Creating a tool definition that the LLM can call
/// 3. Validating and deserializing tool call arguments
///
/// The derive macro provides automatic implementation for most use cases:
///
/// ```rust,ignore
/// #[derive(Serialize, Deserialize, StructuredOutput)]
/// #[structured_output(
///     name = "final_answer",
///     description = "Final response with structured data"
/// )]
/// struct Answer {
///     response: String,
///     confidence: f32,
/// }
/// ```
pub trait StructuredOutput: Serialize + DeserializeOwned {
    /// Tool name used in API requests (e.g., "final_answer", "create_character")
    fn tool_name() -> &'static str;

    /// Human-readable description of what this output represents
    fn tool_description() -> &'static str;

    /// JSON Schema describing this type's structure
    ///
    /// The schema should follow the JSON Schema specification and will be used
    /// to validate the LLM's output before deserialization.
    fn json_schema() -> serde_json::Value;

    /// Complete tool definition ready for API requests
    ///
    /// This combines the tool name, description, and schema into the format
    /// expected by LLM APIs (OpenAI, Anthropic, etc.)
    fn tool_definition() -> ToolDefinition {
        ToolDefinition {
            name: Self::tool_name().to_string(),
            description: Self::tool_description().to_string(),
            parameters: Self::json_schema(),
        }
    }
}

/// Message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message::user("Hello");
        assert_eq!(msg.role, "user");
        assert_eq!(msg.content, "Hello");
    }
}
