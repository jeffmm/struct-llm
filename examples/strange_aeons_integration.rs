// Example showing complete integration with a game like Strange Aeons
//
// This demonstrates the full workflow of using struct-llm in a WASM-compatible
// game engine where you need structured LLM outputs for procedural content.
//
// Run with: cargo run --example strange_aeons_integration --features derive

use serde::{Deserialize, Serialize};
use struct_llm::{extract_tool_calls, parse_tool_response, Provider, StructuredOutput, Message};

#[derive(Debug, Serialize, Deserialize, StructuredOutput)]
#[structured_output(
    name = "generate_character",
    description = "Generates a character for Strange Aeons with stats and background"
)]
struct CharacterData {
    name: String,
    age: u32,
    occupation: String,
    strength: u32,
    intelligence: u32,
    sanity: u32,
    backstory: String,
    motivation: String,
}

// This replaces the fragile JSON parsing in character.rs:218-231
// OLD (error-prone):
//   let start = response.find('{').ok_or(...)?;
//   let end = response.rfind('}').ok_or(...)?;
//   let json_str = &response[start..=end];
//   let data: CharacterData = serde_json::from_str(json_str)?;
//
// NEW (reliable):
//   let tool_calls = extract_tool_calls(&response, Provider::Anthropic)?;
//   let data: CharacterData = parse_tool_response(&tool_calls[0])?;

fn main() {
    println!("=== Strange Aeons Integration Example ===\n");

    // Step 1: Create the prompt
    let messages = vec![
        Message::system("You are a creative writer for a cosmic horror game."),
        Message::user(
            "Create a mysterious character who has just arrived in the cursed town of Innsmouth."
        ),
    ];

    // Step 2: Get tool definition to include in API request
    let tool = CharacterData::tool_definition();

    println!("Tool to send to LLM API:");
    println!("{}\n", serde_json::to_string_pretty(&tool).unwrap());

    // Step 3: In your actual code, make the API request
    // Example for Anthropic (Claude):
    println!("=== API Request Format (Anthropic) ===");
    let api_request = serde_json::json!({
        "model": "claude-3-5-sonnet-20241022",
        "max_tokens": 1024,
        "messages": messages,
        "tools": [tool]
    });
    println!("{}\n", serde_json::to_string_pretty(&api_request).unwrap());

    // Step 4: Simulate LLM response with tool call
    println!("=== Simulated LLM Response ===");
    let api_response = serde_json::json!({
        "id": "msg_123",
        "type": "message",
        "role": "assistant",
        "content": [
            {
                "type": "tool_use",
                "id": "toolu_456",
                "name": "generate_character",
                "input": {
                    "name": "Samuel Marsh",
                    "age": 34,
                    "occupation": "Marine Biologist",
                    "strength": 12,
                    "intelligence": 16,
                    "sanity": 70,
                    "backstory": "Samuel came to Innsmouth to study the peculiar fish mutations in the harbor, unaware of the town's dark secrets.",
                    "motivation": "Seeking to understand the strange biology of the coastal waters"
                }
            }
        ],
        "model": "claude-3-5-sonnet-20241022",
        "stop_reason": "tool_use"
    });
    println!("{}\n", serde_json::to_string_pretty(&api_response).unwrap());

    // Step 5: Extract and parse the structured response
    println!("=== Parsing Tool Response ===");

    // Extract tool calls from the API response
    let response_str = serde_json::to_string(&api_response).unwrap();
    let tool_calls = extract_tool_calls(&response_str, Provider::Anthropic)
        .expect("Failed to extract tool calls");

    println!("Extracted {} tool call(s)", tool_calls.len());

    // Parse the first tool call into our struct
    let character: CharacterData = parse_tool_response(&tool_calls[0])
        .expect("Failed to parse tool response");

    println!("\n=== Parsed Character Data ===");
    println!("{:#?}", character);

    println!("\n=== Benefits Over Manual JSON Parsing ===");
    println!("✓ No need to search for {{ and }} in the response");
    println!("✓ Handles provider-specific formats (OpenAI vs Anthropic)");
    println!("✓ Built-in validation via JSON Schema");
    println!("✓ Type-safe deserialization");
    println!("✓ Works with streaming responses");
    println!("✓ WASM-compatible (no async required)");

    println!("\n=== Integration Steps for Strange Aeons ===");
    println!("1. Add struct-llm to Cargo.toml dependencies");
    println!("2. Define your data structures with #[derive(StructuredOutput)]");
    println!("3. Replace manual JSON parsing with extract_tool_calls() + parse_tool_response()");
    println!("4. Include tool_definition() in your API requests");
}
