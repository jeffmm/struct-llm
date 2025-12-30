// Example showing NPC generation for game development (like Strange Aeons)
//
// This demonstrates how to use struct-llm to generate structured game data
// without manual JSON parsing.
//
// Run with: cargo run --example npc_generation --features derive

use serde::{Deserialize, Serialize};
use struct_llm::StructuredOutput;

#[derive(Debug, Serialize, Deserialize, StructuredOutput)]
#[structured_output(
    name = "create_npc",
    description = "Creates a detailed NPC character for a cosmic horror game"
)]
struct NPCData {
    name: String,
    age: u32,
    occupation: String,
    personality_traits: Vec<String>,
    backstory: String,
    sanity: u32,
    physical_description: String,
    secrets: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, StructuredOutput)]
#[structured_output(
    name = "generate_location",
    description = "Generates a mysterious location for exploration"
)]
struct LocationData {
    name: String,
    description: String,
    atmosphere: String,
    hidden_details: Vec<String>,
    danger_level: u32,
}

fn main() {
    println!("=== NPC Generation Example ===\n");

    // Get the tool definition to send to LLM
    let npc_tool = NPCData::tool_definition();
    println!("Tool Definition (send this to LLM API):");
    println!("{}\n", serde_json::to_string_pretty(&npc_tool).unwrap());

    // Example: Simulating what the LLM would return
    println!("=== Simulated LLM Response ===");
    let simulated_tool_call = serde_json::json!({
        "name": "create_npc",
        "arguments": {
            "name": "Dr. Eliza Winters",
            "age": 42,
            "occupation": "Archaeologist",
            "personality_traits": ["Curious", "Meticulous", "Haunted by past discoveries"],
            "backstory": "Dr. Winters spent years studying ancient civilizations, but her last expedition uncovered something that defied all explanation. She's been searching for answers ever since.",
            "sanity": 65,
            "physical_description": "Tall and thin, with prematurely graying hair. Dark circles under her eyes suggest many sleepless nights. Always carries a worn leather journal.",
            "secrets": [
                "Witnessed an entity beyond comprehension in Sumatra",
                "Has been having recurring nightmares of a sunken city"
            ]
        }
    });

    println!("{}\n", serde_json::to_string_pretty(&simulated_tool_call).unwrap());

    // In real usage, you would:
    // 1. Make API request with tool definition
    // 2. Extract tool calls using extract_tool_calls()
    // 3. Parse into struct using parse_tool_response()
    //
    // For example:
    // let tool_calls = extract_tool_calls(&response, Provider::Anthropic)?;
    // let npc: NPCData = parse_tool_response(&tool_calls[0])?;

    println!("=== Location Generation Example ===\n");
    let location_tool = LocationData::tool_definition();
    println!("{}", serde_json::to_string_pretty(&location_tool).unwrap());
}
