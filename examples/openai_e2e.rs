// End-to-end example with real OpenAI API calls
//
// This example makes actual API requests to OpenAI and demonstrates
// the complete workflow from request to validated structured output.
//
// Requires: OPENAI_API_KEY environment variable
// Run with: OPENAI_API_KEY=your_key cargo run --example openai_e2e

use serde::{Deserialize, Serialize};
use struct_llm::{
    build_enforced_tool_request, extract_tool_calls, parse_tool_response, Message, Provider,
    StructuredOutput,
};

#[derive(Debug, Serialize, Deserialize, StructuredOutput)]
#[structured_output(
    name = "analyze_sentiment",
    description = "Analyzes the sentiment of the given text with reasoning"
)]
struct SentimentAnalysis {
    sentiment: String,
    confidence: f32,
    reasoning: String,
    key_phrases: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check for API key
    let api_key = match std::env::var("OPENAI_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            eprintln!("Error: OPENAI_API_KEY environment variable not set");
            eprintln!("Usage: OPENAI_API_KEY=your_key cargo run --example openai_e2e");
            std::process::exit(1);
        }
    };

    println!("=== OpenAI End-to-End Example ===\n");

    // Step 1: Get the tool definition
    let tool = SentimentAnalysis::tool_definition();
    println!("Tool definition:");
    println!("{}\n", serde_json::to_string_pretty(&tool)?);

    // Step 2: Build the API request with enforced tool call (pydantic AI / luagent pattern)
    let messages = vec![Message::user(
        "Analyze the sentiment of this text: 'I absolutely love this library! \
         The API is intuitive and the documentation is excellent. Makes my work so much easier.'"
    )];

    let mut request_body = build_enforced_tool_request(&messages, &tool, Provider::OpenAI);
    request_body["model"] = serde_json::json!("gpt-4o-mini");

    println!("Making API request to OpenAI...");

    // Step 3: Make the HTTP request
    let client = reqwest::Client::new();
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
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
    let tool_calls = extract_tool_calls(&response_text, Provider::OpenAI)?;
    println!("Found {} tool call(s)\n", tool_calls.len());

    // Step 5: Parse and validate the structured output
    println!("Parsing and validating response...");
    let analysis: SentimentAnalysis = parse_tool_response(&tool_calls[0])?;

    println!("\n=== Sentiment Analysis Result ===");
    println!("{:#?}", analysis);

    println!("\n✅ Success! The response was validated against the JSON schema.");

    Ok(())
}
