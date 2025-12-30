# struct-llm

A lightweight, WASM-compatible Rust library for generating structured outputs from LLMs using a tool-based approach. Inspired by [Pydantic AI](https://ai.pydantic.dev) and [luagent](https://github.com/yourusername/luagent).

## Features

- **🎯 Structured Outputs**: Type-safe, validated LLM responses using JSON schema and tool calling
- **🌐 Provider-Independent**: Works with any API supporting tool/function calling (OpenAI, Anthropic, local models)
- **📡 Streaming Compatible**: Tool-based approach works seamlessly with streaming responses
- **🦀 Type-Safe**: Leverages Rust's type system with serde integration
- **🕸️ WASM-Ready**: Synchronous API, no async/await required in the library itself
- **🪶 Lightweight**: Minimal dependencies, you bring your own HTTP client
- **🔧 Flexible**: Use derive macros for convenience or implement traits manually

## Why Tool-Based Structured Outputs?

Instead of relying on provider-specific features like OpenAI's `response_format`, this library uses a universal **tool calling** approach:

1. Your output schema is registered as a special `final_answer` tool
2. The LLM calls this tool when ready to return structured data
3. The library validates and deserializes the tool call arguments

**Benefits:**
- ✅ Works with streaming (tool calls can be streamed)
- ✅ Provider-independent (any model supporting tool calling)
- ✅ Mix structured output with regular tools
- ✅ More reliable than parsing raw JSON from text

## Quick Start

**See the [examples](./examples) directory for complete working examples!**

```rust
use struct_llm::{build_enforced_tool_request, extract_tool_calls, parse_tool_response,
                 Message, Provider, StructuredOutput};
use serde::{Deserialize, Serialize};

// Define your output structure
#[derive(Debug, Serialize, Deserialize, StructuredOutput)]
#[structured_output(
    name = "sentiment_analysis",
    description = "Analyzes the sentiment of the given text"
)]
struct SentimentAnalysis {
    sentiment: String,
    confidence: f32,
    reasoning: String,
}

// Get tool definition and build request that ENFORCES the tool call
let tool = SentimentAnalysis::tool_definition();
let messages = vec![Message::user("Analyze: 'This library is amazing!'")];
let mut request = build_enforced_tool_request(&messages, &tool, Provider::OpenAI);
request["model"] = "gpt-4o-mini".into();

// Your async code makes the HTTP request
let response = your_api_client
    .post("https://api.openai.com/v1/chat/completions")
    .json(&request)
    .send()
    .await?;

// Extract and validate the structured response (sync)
let tool_calls = extract_tool_calls(&response.text(), Provider::OpenAI)?;
let result: SentimentAnalysis = parse_tool_response(&tool_calls[0])?;

println!("Sentiment: {}", result.sentiment);
println!("Confidence: {}", result.confidence);
```

**Key insight:** `build_enforced_tool_request()` ensures the LLM *must* call your tool (like pydantic AI / luagent), guaranteeing you always get structured output back.

## Architecture

This library is designed to be a **utility layer** that you integrate into your existing async code:

```
┌─────────────────────────────────────────────────────────────┐
│ Your Application (async/await)                              │
│ - Makes HTTP requests (reqwest, ureq, etc.)                 │
│ - Handles API keys, retries, rate limiting                  │
│ - Manages conversation state                                │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ struct-llm (sync utilities)                                 │
│ - Converts Rust types to JSON Schema                        │
│ - Builds tool definitions for API requests                  │
│ - Parses tool calls from responses                          │
│ - Validates and deserializes tool arguments                 │
└─────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. StructuredOutput Trait

Defines how a type can be used as a structured LLM output:

```rust
pub trait StructuredOutput: Serialize + DeserializeOwned {
    /// Tool name (e.g., "final_answer", "create_character")
    fn tool_name() -> &'static str;

    /// Human-readable description of what this output represents
    fn tool_description() -> &'static str;

    /// JSON Schema for this type's structure
    fn json_schema() -> serde_json::Value;

    /// Complete tool definition for API requests
    fn tool_definition() -> ToolDefinition {
        ToolDefinition {
            name: Self::tool_name().to_string(),
            description: Self::tool_description().to_string(),
            parameters: Self::json_schema(),
        }
    }
}
```

### 2. Derive Macro (Ergonomic API)

```rust
use struct_llm::StructuredOutput;

#[derive(Serialize, Deserialize, StructuredOutput)]
#[structured_output(
    name = "create_npc",
    description = "Creates a character with structured data"
)]
struct NPCData {
    name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,

    #[schema(description = "Physical appearance and first impression")]
    description: String,

    backstory: String,
    personality: String,

    #[schema(min_items = 1, max_items = 5)]
    dialogue_hints: Vec<String>,
}
```

### 3. Provider Adapters

Handle API-specific formatting differences:

```rust
pub enum Provider {
    OpenAI,
    Anthropic,
    Local,
}

pub fn build_request_with_tools(
    messages: &[Message],
    tools: &[ToolDefinition],
    provider: Provider,
) -> serde_json::Value {
    match provider {
        Provider::OpenAI => /* OpenAI format */,
        Provider::Anthropic => /* Anthropic format */,
        Provider::Local => /* Generic format */,
    }
}
```

### 4. Tool Call Parsing

Extract tool calls from various response formats:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Extract tool calls from API response text
pub fn extract_tool_calls(
    response: &str,
    provider: Provider,
) -> Result<Vec<ToolCall>, Error> {
    // Parse response based on provider format
    // Return structured tool calls
}

/// Parse and validate a specific tool call
pub fn parse_tool_response<T: StructuredOutput>(
    tool_call: &ToolCall,
) -> Result<T, Error> {
    // Validate against schema
    // Deserialize to T
}
```

## Usage Examples

### Basic Structured Output

```rust
use struct_llm::{StructuredOutput, Provider, extract_tool_calls, parse_tool_response};

#[derive(Serialize, Deserialize, StructuredOutput)]
#[structured_output(name = "final_answer", description = "Final response")]
struct Answer {
    response: String,
    confidence: f32,
}

async fn generate_answer(prompt: &str) -> Result<Answer, Error> {
    // 1. Build request with tool definition
    let tool = Answer::tool_definition();
    let request = build_request_with_tools(
        &[Message::user(prompt)],
        &[tool],
        Provider::OpenAI,
    );

    // 2. Make HTTP request (your async code)
    let client = reqwest::Client::new();
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request)
        .send()
        .await?
        .text()
        .await?;

    // 3. Parse tool calls (sync)
    let tool_calls = extract_tool_calls(&response, Provider::OpenAI)?;

    // 4. Validate and deserialize (sync)
    let answer: Answer = parse_tool_response(&tool_calls[0])?;

    Ok(answer)
}
```

### Streaming Support

```rust
use struct_llm::{StreamParser, ToolCall};

async fn generate_with_streaming(prompt: &str) -> Result<Answer, Error> {
    let mut parser = StreamParser::new(Provider::OpenAI);
    let mut accumulated_args = String::new();

    // Stream response chunks
    let mut stream = make_streaming_request(prompt).await?;

    while let Some(chunk) = stream.next().await {
        let chunk_text = chunk?;

        // Parse incremental SSE data
        if let Some(tool_delta) = parser.parse_chunk(&chunk_text)? {
            match tool_delta {
                ToolDelta::Start { name, id } => {
                    println!("Tool call started: {}", name);
                }
                ToolDelta::Arguments { delta } => {
                    accumulated_args.push_str(&delta);
                    print!("{}", delta); // Show progress
                }
                ToolDelta::End => {
                    println!("\nTool call complete");
                }
            }
        }
    }

    // Get final tool call
    let tool_call = parser.finalize()?;
    let answer: Answer = parse_tool_response(&tool_call)?;

    Ok(answer)
}
```

### Custom Schema (No Derive Macro)

```rust
impl StructuredOutput for CustomType {
    fn tool_name() -> &'static str {
        "custom_output"
    }

    fn tool_description() -> &'static str {
        "Custom structured output"
    }

    fn json_schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "field1": { "type": "string" },
                "field2": { "type": "number" }
            },
            "required": ["field1", "field2"]
        })
    }
}
```

### Mixing Regular Tools with Structured Output

```rust
// Define regular tools
let tools = vec![
    ToolDefinition {
        name: "get_weather".to_string(),
        description: "Get current weather".to_string(),
        parameters: weather_schema(),
    },
    ToolDefinition {
        name: "search_web".to_string(),
        description: "Search the web".to_string(),
        parameters: search_schema(),
    },
    // Structured output as final tool
    WeatherReport::tool_definition(),
];

// The LLM can call regular tools first, then the final_answer tool
let response = call_api_with_tools(prompt, &tools).await?;
let tool_calls = extract_tool_calls(&response, Provider::OpenAI)?;

// Handle each tool call
for tool_call in tool_calls {
    if tool_call.name == WeatherReport::tool_name() {
        let report: WeatherReport = parse_tool_response(&tool_call)?;
        return Ok(report);
    } else {
        // Execute other tools
        handle_regular_tool(&tool_call)?;
    }
}
```

## WASM Compatibility

The library is designed to work in WASM environments:

```rust
// No async/await in the library itself
// No file system access
// No std-only dependencies

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn parse_llm_response(response_json: &str) -> Result<JsValue, JsValue> {
    let tool_calls = extract_tool_calls(response_json, Provider::OpenAI)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let result: MyOutput = parse_tool_response(&tool_calls[0])
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    Ok(serde_wasm_bindgen::to_value(&result)?)
}
```

## Comparison to Alternatives

| Feature | struct-llm | raw JSON parsing | provider-specific APIs |
|---------|------------|------------------|------------------------|
| Streaming | ✅ Yes | ❌ No | ⚠️ Sometimes |
| Provider-independent | ✅ Yes | ⚠️ Manual | ❌ No |
| Type-safe | ✅ Yes | ❌ No | ✅ Yes |
| WASM-compatible | ✅ Yes | ✅ Yes | ⚠️ Varies |
| Mix with regular tools | ✅ Yes | ❌ No | ⚠️ Sometimes |
| Validation | ✅ Automatic | ❌ Manual | ✅ Automatic |

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("Schema validation failed: {0}")]
    ValidationFailed(String),

    #[error("No tool calls found in response")]
    NoToolCalls,

    #[error("Tool call '{0}' does not match expected tool")]
    ToolMismatch(String),

    #[error("Invalid provider response format")]
    InvalidResponseFormat,
}
```

## Roadmap

### v0.1 - Core Functionality
- [x] `StructuredOutput` trait
- [ ] Derive macro for `StructuredOutput`
- [ ] Provider adapters (OpenAI, Anthropic, Local)
- [ ] Tool call extraction and parsing
- [ ] JSON Schema generation from Rust types
- [ ] Basic validation

### v0.2 - Streaming & Ergonomics
- [ ] Streaming parser for incremental responses
- [ ] Schema attributes (`#[schema(description = "...")]`)
- [ ] Helper functions for common patterns
- [ ] Better error messages

### v0.3 - Advanced Features
- [ ] Schema caching for performance
- [ ] Custom validators
- [ ] Tool execution framework (optional)
- [ ] Conversation state helpers

## Design Philosophy

1. **Utility Layer**: You handle HTTP, we handle schemas and parsing
2. **Type Safety**: Leverage Rust's type system, not runtime magic
3. **WASM First**: No async, no filesystem, pure data transforms
4. **Bring Your Own Client**: Works with reqwest, ureq, or fetch API
5. **Simple & Focused**: Does one thing well - structured outputs

## Contributing

This library is in early development. Contributions welcome!

- Keep the API synchronous (no async in the library)
- Maintain WASM compatibility
- Add tests for new features
- Document provider-specific quirks

## License

MIT OR Apache-2.0

## Inspiration

- [Pydantic AI](https://ai.pydantic.dev/) - Python agent framework with structured outputs
- [luagent](https://github.com/yourusername/luagent) - Lua agent library using tool-based outputs
- [instructor](https://github.com/jxnl/instructor) - Structured outputs for OpenAI

## See Also

- [Rig](https://github.com/0xPlaygrounds/rig) - Full-featured Rust agent framework
- [schemars](https://github.com/GREsau/schemars) - JSON Schema generation for Rust
