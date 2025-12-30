# struct-llm Examples

This directory contains examples demonstrating how to use struct-llm in various scenarios.

## Running Examples

All examples require the `derive` feature (enabled by default):

```bash
cargo run --example <example_name>
```

## Available Examples

### 1. Basic Usage (`basic.rs`)

The simplest example showing how to define a structured output and generate tool definitions.

```bash
cargo run --example basic
```

**What it demonstrates:**
- Basic `#[derive(StructuredOutput)]` usage
- Generating tool definitions
- Inspecting JSON schemas

### 2. NPC Generation (`npc_generation.rs`)

Shows how to use struct-llm for game development, generating structured game data like NPCs and locations.

```bash
cargo run --example npc_generation
```

**What it demonstrates:**
- Multiple structured output types
- Complex nested data (Vec fields)
- Game development use cases

### 3. Streaming (`streaming.rs`)

Demonstrates real-time parsing of streaming SSE responses from LLM APIs.

```bash
cargo run --example streaming
```

**What it demonstrates:**
- Using `StreamParser` for incremental parsing
- Handling OpenAI and Anthropic streaming formats
- Processing `ToolDelta` events in real-time
- Displaying partial results as they arrive

### 4. OpenAI End-to-End (`openai_e2e.rs`)

**Real API calls to OpenAI** - Complete workflow with actual HTTP requests and responses.

```bash
OPENAI_API_KEY=your_key cargo run --example openai_e2e
```

**What it demonstrates:**
- Making real API requests to OpenAI
- Sentiment analysis with structured output
- Full request/response cycle
- Error handling for missing API keys
- JSON schema validation in practice

**Requirements:** Set `OPENAI_API_KEY` environment variable

### 5. Anthropic End-to-End (`anthropic_e2e.rs`)

**Real API calls to Anthropic Claude** - Complete workflow with actual HTTP requests and responses.

```bash
ANTHROPIC_API_KEY=your_key cargo run --example anthropic_e2e
```

**What it demonstrates:**
- Making real API requests to Anthropic (Claude)
- Character generation with structured output
- Full request/response cycle
- Error handling for missing API keys
- JSON schema validation in practice

**Requirements:** Set `ANTHROPIC_API_KEY` environment variable

### 6. Strange Aeons Integration (`strange_aeons_integration.rs`)

Complete end-to-end example showing integration with a WASM game like Strange Aeons (simulated, no API calls).

```bash
cargo run --example strange_aeons_integration
```

**What it demonstrates:**
- Full API request/response workflow
- Parsing Anthropic Claude responses
- Replacing manual JSON extraction with tool-based approach
- WASM-compatible synchronous API

**Key benefit:** Replaces fragile JSON parsing like:
```rust
// OLD (error-prone)
let start = response.find('{').ok_or(...)?;
let end = response.rfind('}').ok_or(...)?;
let json_str = &response[start..=end];
let data: CharacterData = serde_json::from_str(json_str)?;

// NEW (reliable)
let tool_calls = extract_tool_calls(&response, Provider::Anthropic)?;
let data: CharacterData = parse_tool_response(&tool_calls[0])?;
```

## Enforcing Tool Calls (Recommended Pattern)

Following the **pydantic AI** and **luagent** approach, these examples use **enforced tool calls** to guarantee structured outputs. This ensures the LLM always returns your data structure rather than deciding whether to use the tool.

**Key function:** `build_enforced_tool_request()`

```rust
// Force the LLM to call this specific tool
let request = build_enforced_tool_request(&messages, &tool, Provider::OpenAI);
```

This is more reliable than `tool_choice: "auto"` which lets the LLM decide whether to call the tool.

## Integration Pattern

All examples follow the same pattern:

1. **Define your structure** with `#[derive(StructuredOutput)]`:
   ```rust
   #[derive(Serialize, Deserialize, StructuredOutput)]
   #[structured_output(
       name = "tool_name",
       description = "What this tool does"
   )]
   struct MyData {
       field1: String,
       field2: u32,
   }
   ```

2. **Generate tool definition** to send to LLM API:
   ```rust
   let tool = MyData::tool_definition();
   // Include in your API request
   ```

3. **Parse the response** when LLM calls the tool:
   ```rust
   let tool_calls = extract_tool_calls(&response, Provider::Anthropic)?;
   let data: MyData = parse_tool_response(&tool_calls[0])?;
   ```

## Provider Support

The examples show Anthropic format, but the library supports:
- OpenAI (ChatGPT, GPT-4)
- Anthropic (Claude)
- Local models (LLaMA, Mistral, etc.)

Switch providers by changing the `Provider` enum:
```rust
extract_tool_calls(&response, Provider::OpenAI)      // for OpenAI
extract_tool_calls(&response, Provider::Anthropic)    // for Claude
extract_tool_calls(&response, Provider::Local)        // for local models
```
