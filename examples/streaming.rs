// Example showing streaming parser for real-time tool call parsing
//
// This demonstrates how to process streaming SSE responses from LLM APIs
// and get incremental updates as the tool call is being generated.
//
// Run with: cargo run --example streaming --features derive

use serde::{Deserialize, Serialize};
use struct_llm::{Provider, StreamParser, StructuredOutput, ToolDelta};

#[derive(Debug, Serialize, Deserialize, StructuredOutput)]
#[structured_output(
    name = "generate_story",
    description = "Generates a short story with structured elements"
)]
struct Story {
    title: String,
    setting: String,
    protagonist: String,
    plot_twist: String,
}

fn main() {
    println!("=== Streaming Parser Example ===\n");

    // Example 1: OpenAI streaming format
    println!("--- OpenAI Streaming Format ---\n");
    demo_openai_streaming();

    println!("\n--- Anthropic Streaming Format ---\n");
    demo_anthropic_streaming();
}

fn demo_openai_streaming() {
    let mut parser = StreamParser::new(Provider::OpenAI);

    // Simulate OpenAI SSE chunks (these would come from the API stream)
    let chunks = vec![
        // First chunk: tool call starts
        r#"data: {"choices":[{"delta":{"tool_calls":[{"id":"call_abc123","function":{"name":"generate_story"}}]}}]}"#,
        // Subsequent chunks: arguments streamed incrementally
        r#"data: {"choices":[{"delta":{"tool_calls":[{"function":{"arguments":"{"}}]}}]}"#,
        r#"data: {"choices":[{"delta":{"tool_calls":[{"function":{"arguments":"\"title\": \""}}]}}]}"#,
        r#"data: {"choices":[{"delta":{"tool_calls":[{"function":{"arguments":"The Midnight"}}]}}]}"#,
        r#"data: {"choices":[{"delta":{"tool_calls":[{"function":{"arguments":" Garden\","}}]}}]}"#,
        r#"data: {"choices":[{"delta":{"tool_calls":[{"function":{"arguments":"\"setting\": \"A"}}]}}]}"#,
        r#"data: {"choices":[{"delta":{"tool_calls":[{"function":{"arguments":" mysterious garden\","}}]}}]}"#,
        r#"data: {"choices":[{"delta":{"tool_calls":[{"function":{"arguments":"\"protagonist\": \"Luna\","}}]}}]}"#,
        r#"data: {"choices":[{"delta":{"tool_calls":[{"function":{"arguments":"\"plot_twist\": \"The garden exists only in dreams\""}}]}}]}"#,
        r#"data: {"choices":[{"delta":{"tool_calls":[{"function":{"arguments":"}"}}]}}]}"#,
        // Final chunk
        "data: [DONE]",
    ];

    println!("Processing SSE stream...\n");

    let mut accumulated_text = String::new();

    for chunk in chunks {
        if let Ok(Some(delta)) = parser.parse_chunk(chunk) {
            match delta {
                ToolDelta::Start { id, name } => {
                    println!("🚀 Tool call started:");
                    println!("   ID: {}", id);
                    println!("   Name: {}\n", name);
                }
                ToolDelta::Arguments { delta } => {
                    // In a real app, you could display this incrementally to the user
                    accumulated_text.push_str(&delta);
                    print!("{}", delta);
                    std::io::Write::flush(&mut std::io::stdout()).unwrap();
                }
                ToolDelta::End => {
                    println!("\n\n✅ Tool call complete!");
                }
            }
        }
    }

    // Get the final parsed result
    let tool_call = parser.finalize().expect("Failed to finalize");
    let story: Story = struct_llm::parse_tool_response(&tool_call).expect("Failed to parse story");

    println!("\n=== Parsed Story ===");
    println!("{:#?}", story);
}

fn demo_anthropic_streaming() {
    let mut parser = StreamParser::new(Provider::Anthropic);

    // Simulate Anthropic SSE chunks
    let chunks = vec![
        r#"data: {"type":"content_block_start","content_block":{"type":"tool_use","id":"toolu_xyz789","name":"generate_story"}}"#,
        r#"data: {"type":"content_block_delta","delta":{"type":"input_json_delta","partial_json":"{\"title\": \""}}"#,
        r#"data: {"type":"content_block_delta","delta":{"type":"input_json_delta","partial_json":"The Forgotten"}}"#,
        r#"data: {"type":"content_block_delta","delta":{"type":"input_json_delta","partial_json":" Library\", "}}"#,
        r#"data: {"type":"content_block_delta","delta":{"type":"input_json_delta","partial_json":"\"setting\": \"An ancient library\", "}}"#,
        r#"data: {"type":"content_block_delta","delta":{"type":"input_json_delta","partial_json":"\"protagonist\": \"Elara\", "}}"#,
        r#"data: {"type":"content_block_delta","delta":{"type":"input_json_delta","partial_json":"\"plot_twist\": \"She discovers she's a character in a book\"}"}}"#,
        r#"data: {"type":"content_block_stop"}"#,
    ];

    println!("Processing SSE stream...\n");

    for chunk in chunks {
        if let Ok(Some(delta)) = parser.parse_chunk(chunk) {
            match delta {
                ToolDelta::Start { id, name } => {
                    println!("🚀 Tool call started:");
                    println!("   ID: {}", id);
                    println!("   Name: {}\n", name);
                }
                ToolDelta::Arguments { delta } => {
                    print!("{}", delta);
                    std::io::Write::flush(&mut std::io::stdout()).unwrap();
                }
                ToolDelta::End => {
                    println!("\n\n✅ Tool call complete!");
                }
            }
        }
    }

    // Get the final parsed result
    let tool_call = parser.finalize().expect("Failed to finalize");
    let story: Story = struct_llm::parse_tool_response(&tool_call).expect("Failed to parse story");

    println!("\n=== Parsed Story ===");
    println!("{:#?}", story);
}
