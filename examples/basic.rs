// Basic example showing how to use struct-llm with derive macro
//
// Run with: cargo run --example basic --features derive

use serde::{Deserialize, Serialize};
use struct_llm::StructuredOutput;

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

fn main() {
    // Generate tool definition
    let tool = SentimentAnalysis::tool_definition();

    println!("=== Tool Definition for LLM API ===");
    println!("{}", serde_json::to_string_pretty(&tool).unwrap());

    println!("\n=== Tool Name ===");
    println!("{}", SentimentAnalysis::tool_name());

    println!("\n=== Tool Description ===");
    println!("{}", SentimentAnalysis::tool_description());

    println!("\n=== JSON Schema ===");
    let schema = SentimentAnalysis::json_schema();
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());

    println!("\n=== Example Usage ===");
    println!("1. Include this tool in your API request to OpenAI/Anthropic");
    println!("2. The LLM will call the tool with structured arguments");
    println!("3. Use parse_tool_response() to deserialize into SentimentAnalysis");
}
