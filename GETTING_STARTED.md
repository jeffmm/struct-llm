# Getting Started with struct-llm

## What We've Built

`struct-llm` is a lightweight Rust library that solves the problem of getting **reliable, type-safe structured outputs from LLMs**. Instead of asking the LLM to output raw JSON (which is error-prone), we use a **tool-based approach** where the LLM calls a function with your desired structure.

### Key Innovation: Tool-Based Structured Output

Inspired by [Pydantic AI](https://ai.pydantic.dev) and [luagent](https://github.com/yourusername/luagent), this library treats structured output as a special tool call:

1. Your desired output structure → JSON Schema → Special "final_answer" tool
2. LLM calls the tool with structured arguments
3. Library validates and deserializes the arguments

**Why this is better than raw JSON:**
- ✅ Works with streaming (tool calls can be streamed)
- ✅ Provider-independent (any API supporting tool calling)
- ✅ More reliable (validated at API level before you see it)
- ✅ Can mix with regular tools

## Current Status

✅ **Phase 1 Complete** - Core library working:
- `StructuredOutput` trait
- Provider adapters (OpenAI, Anthropic, Local)
- Tool call extraction & parsing
- JSON Schema utilities
- Full error handling
- 7/7 tests passing
- WASM-compatible

🚧 **Phase 2 Next** - Ergonomics:
- Derive macro for `StructuredOutput`
- Usage examples
- Schema validation
- Streaming parser

## Quick Example (Manual Implementation)

Until the derive macro is ready, you can manually implement `StructuredOutput`:

```rust
use struct_llm::{StructuredOutput, ToolDefinition};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
struct NPCData {
    name: String,
    title: Option<String>,
    description: String,
    backstory: String,
    personality: String,
    dialogue_hints: Vec<String>,
}

impl StructuredOutput for NPCData {
    fn tool_name() -> &'static str {
        "create_npc"
    }

    fn tool_description() -> &'static str {
        "Creates an NPC character with structured data"
    }

    fn json_schema() -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "title": { "type": "string" },
                "description": {
                    "type": "string",
                    "description": "Physical appearance and first impression"
                },
                "backstory": { "type": "string" },
                "personality": { "type": "string" },
                "dialogue_hints": {
                    "type": "array",
                    "items": { "type": "string" },
                    "minItems": 1,
                    "maxItems": 5
                }
            },
            "required": ["name", "description", "backstory", "personality", "dialogue_hints"]
        })
    }
}
```

## Usage in Your Async Code

```rust
use struct_llm::{Message, Provider, extract_tool_calls, parse_tool_response};

async fn generate_npc(prompt: &str, api_key: &str) -> Result<NPCData, Box<dyn std::error::Error>> {
    // 1. Get the tool definition
    let tool = NPCData::tool_definition();

    // 2. Build request with your HTTP client
    let client = reqwest::Client::new();
    let request_body = serde_json::json!({
        "model": "gpt-4",
        "messages": [
            Message::user(prompt)
        ],
        "tools": [{
            "type": "function",
            "function": {
                "name": tool.name,
                "description": tool.description,
                "parameters": tool.parameters
            }
        }]
    });

    // 3. Make the API call (your async code)
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request_body)
        .send()
        .await?
        .text()
        .await?;

    // 4. Extract tool calls (sync - library code)
    let tool_calls = extract_tool_calls(&response, Provider::OpenAI)?;

    // 5. Parse and validate (sync - library code)
    let npc: NPCData = parse_tool_response(&tool_calls[0])?;

    Ok(npc)
}
```

## Integration with Strange Aeons

This library can replace the manual JSON parsing in `src/agents/character.rs`:

**Before (error-prone):**
```rust
// Lines 218-231 in character.rs
let json_text = if let Some(start) = response_text.find('{') {
    if let Some(end) = response_text.rfind('}') {
        &response_text[start..=end]
    } else {
        &response_text
    }
} else {
    &response_text
};

let generated: NPCGenerationResponse = serde_json::from_str(json_text)
    .map_err(|e| format!("Failed to parse NPC data: {}", e))?;
```

**After (reliable):**
```rust
// Implement StructuredOutput for NPCGenerationResponse
// Then use:
let tool_calls = extract_tool_calls(&response_text, Provider::Anthropic)?;
let npc: NPCGenerationResponse = parse_tool_response(&tool_calls[0])?;
```

## Next Steps

### For struct-llm Development:

1. **Create derive macro** - Make implementation automatic
2. **Add examples** - Show real-world usage patterns
3. **Streaming parser** - Handle incremental SSE responses
4. **Schema validation** - Validate against JSON Schema before deserialization
5. **Publish to crates.io** - Make it available to everyone

### For Strange Aeons Integration:

1. **Add struct-llm as dependency** in `Cargo.toml`
2. **Implement StructuredOutput** for `NPCGenerationResponse` (and future `LocationData`, `ItemData`, etc.)
3. **Refactor CharacterAgent** to use tool-based approach
4. **Test with both Anthropic and local models**

## Testing

```bash
cd ~/Projects/struct-llm

# Check compilation
cargo check

# Run tests
cargo test

# Test with WASM
cargo check --target wasm32-unknown-unknown
```

## Design Philosophy

- **Sync utilities** - You handle async, we handle schemas
- **Provider agnostic** - Works with any tool-calling API
- **WASM first** - No filesystem, no async in library
- **Type safe** - Leverages Rust's type system
- **Minimal deps** - Just serde, serde_json, thiserror

## Questions?

This is a new library designed specifically for your use case. The patterns are proven (Pydantic AI, luagent), but the Rust implementation is fresh. Feel free to modify anything!
