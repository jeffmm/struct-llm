// End-to-end example with real Anthropic API calls
//
// This example makes actual API requests to Anthropic (Claude) and demonstrates
// the complete workflow from request to validated structured output.
//
// Requires: ANTHROPIC_API_KEY environment variable
// Run with: ANTHROPIC_API_KEY=your_key cargo run --example anthropic_e2e

use serde::{Deserialize, Serialize};
use struct_llm::{
    build_enforced_tool_request, extract_tool_calls, parse_tool_response, Message, Provider,
    StructuredOutput,
};

#[derive(Debug, Serialize, Deserialize, StructuredOutput)]
#[structured_output(
    name = "create_character",
    description = "Creates a character for a story with detailed attributes"
)]
struct Character {
    name: String,
    age: u32,
    occupation: String,
    personality_traits: Vec<String>,
    backstory: String,
    motivation: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check for API key
    let api_key = match std::env::var("ANTHROPIC_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            eprintln!("Error: ANTHROPIC_API_KEY environment variable not set");
            eprintln!("Usage: ANTHROPIC_API_KEY=your_key cargo run --example anthropic_e2e");
            std::process::exit(1);
        }
    };

    println!("=== Anthropic (Claude) End-to-End Example ===\n");

    // Step 1: Get the tool definition
    let tool = Character::tool_definition();
    println!("Tool definition:");
    println!("{}\n", serde_json::to_string_pretty(&tool)?);

    // Step 2: Build the API request with enforced tool call (pydantic AI / luagent pattern)
    let messages = vec![Message::user(
        "Create a mysterious detective character for a noir story set in 1940s San Francisco."
    )];

    let mut request_body = build_enforced_tool_request(&messages, &tool, Provider::Anthropic);
    request_body["model"] = serde_json::json!("claude-3-haiku-20240307");
    request_body["max_tokens"] = serde_json::json!(1024);

    println!("Making API request to Anthropic...");

    // Step 3: Make the HTTP request
    let client = reqwest::Client::new();
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await?;
        eprintln!("API Error ({}): {}", status, error_text);
        std::process::exit(1);
    }

    let response_text = response.text().await?;
    println!("Received response\n");

    // Step 4: Extract tool calls
    println!("Extracting tool calls...");
    let tool_calls = extract_tool_calls(&response_text, Provider::Anthropic)?;
    println!("Found {} tool call(s)\n", tool_calls.len());

    // Step 5: Parse and validate the structured output
    println!("Parsing and validating response...");
    let character: Character = parse_tool_response(&tool_calls[0])?;

    println!("\n=== Character Created ===");
    println!("Name: {}", character.name);
    println!("Age: {}", character.age);
    println!("Occupation: {}", character.occupation);
    println!("\nPersonality Traits:");
    for trait_name in &character.personality_traits {
        println!("  - {}", trait_name);
    }
    println!("\nBackstory:\n{}", character.backstory);
    println!("\nMotivation:\n{}", character.motivation);

    println!("\n✅ Success! The response was validated against the JSON schema.");

    Ok(())
}
