//! Error types for struct-llm

use thiserror::Error;

/// Errors that can occur when working with structured LLM outputs
#[derive(Debug, Error)]
pub enum Error {
    /// JSON parsing or serialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Schema validation failed
    #[error("Schema validation failed: {0}")]
    ValidationFailed(String),

    /// No tool calls found in the response
    #[error("No tool calls found in response")]
    NoToolCalls,

    /// Tool call name doesn't match expected tool
    #[error("Tool call '{0}' does not match expected tool '{1}'")]
    ToolMismatch(String, String),

    /// Invalid response format from provider
    #[error("Invalid response format from provider: {0}")]
    InvalidResponseFormat(String),

    /// Missing required field in response
    #[error("Missing required field: {0}")]
    MissingField(String),
}

/// Result type alias for struct-llm operations
pub type Result<T> = std::result::Result<T, Error>;
