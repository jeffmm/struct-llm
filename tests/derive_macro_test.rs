use serde::{Deserialize, Serialize};
use struct_llm::StructuredOutput;

#[derive(Debug, Serialize, Deserialize, StructuredOutput)]
#[structured_output(
    name = "test_output",
    description = "Test structured output"
)]
struct TestOutput {
    message: String,
    count: u32,
    score: f64,
    is_valid: bool,
}

#[derive(Debug, Serialize, Deserialize, StructuredOutput)]
#[structured_output(
    name = "npc_data",
    description = "Creates an NPC character with details"
)]
struct NPCData {
    name: String,
    age: u32,
    backstory: String,
    strength: u32,
    intelligence: u32,
}

#[test]
fn test_derive_basic() {
    assert_eq!(TestOutput::tool_name(), "test_output");
    assert_eq!(TestOutput::tool_description(), "Test structured output");

    let schema = TestOutput::json_schema();
    assert_eq!(schema["type"], "object");
    assert!(schema["properties"].is_object());
    assert!(schema["required"].is_array());
}

#[test]
fn test_derive_tool_definition() {
    let tool = TestOutput::tool_definition();

    assert_eq!(tool.name, "test_output");
    assert_eq!(tool.description, "Test structured output");
    assert_eq!(tool.parameters["type"], "object");
}

#[test]
fn test_npc_schema() {
    let schema = NPCData::json_schema();

    // Verify schema structure
    assert_eq!(schema["type"], "object");

    // Check properties exist
    let properties = &schema["properties"];
    assert!(properties["name"].is_object());
    assert!(properties["age"].is_object());
    assert!(properties["backstory"].is_object());
    assert!(properties["strength"].is_object());
    assert!(properties["intelligence"].is_object());

    // Check property types
    assert_eq!(properties["name"]["type"], "string");
    assert_eq!(properties["age"]["type"], "integer");
    assert_eq!(properties["backstory"]["type"], "string");
    assert_eq!(properties["strength"]["type"], "integer");
    assert_eq!(properties["intelligence"]["type"], "integer");

    // Check required fields
    let required = schema["required"].as_array().unwrap();
    assert_eq!(required.len(), 5);
}

#[test]
fn test_type_inference() {
    let schema = TestOutput::json_schema();
    let properties = &schema["properties"];

    assert_eq!(properties["message"]["type"], "string");
    assert_eq!(properties["count"]["type"], "integer");
    assert_eq!(properties["score"]["type"], "number");
    assert_eq!(properties["is_valid"]["type"], "boolean");
}
