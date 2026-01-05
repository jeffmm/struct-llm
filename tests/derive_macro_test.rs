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

// Test for Vec<String> (primitive array)
#[derive(Debug, Serialize, Deserialize, StructuredOutput)]
#[structured_output(
    name = "list_output",
    description = "Output with primitive arrays"
)]
struct ListOutput {
    tags: Vec<String>,
    scores: Vec<i32>,
}

#[test]
fn test_primitive_vec() {
    let schema = ListOutput::json_schema();
    let properties = &schema["properties"];

    // Check tags is array of strings
    assert_eq!(properties["tags"]["type"], "array");
    assert_eq!(properties["tags"]["items"]["type"], "string");

    // Check scores is array of integers
    assert_eq!(properties["scores"]["type"], "array");
    assert_eq!(properties["scores"]["items"]["type"], "integer");
}

// Test for Vec<CustomStruct> (nested struct array)
// Inner struct must also derive StructuredOutput
#[derive(Debug, Clone, Serialize, Deserialize, StructuredOutput)]
#[structured_output(
    name = "room_item",
    description = "A single room with description and lore"
)]
struct RoomItem {
    room_id: String,
    description: String,
    lore_entries: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, StructuredOutput)]
#[structured_output(
    name = "room_batch",
    description = "A batch of rooms"
)]
struct RoomBatch {
    rooms: Vec<RoomItem>,
}

#[test]
fn test_nested_struct_vec() {
    let schema = RoomBatch::json_schema();
    let properties = &schema["properties"];

    // Check rooms is array
    assert_eq!(properties["rooms"]["type"], "array");

    // Check items schema is an object (not string!)
    let items_schema = &properties["rooms"]["items"];
    assert_eq!(items_schema["type"], "object", "Vec<CustomStruct> should generate object schema, not string");

    // Check nested properties exist
    let nested_props = &items_schema["properties"];
    assert_eq!(nested_props["room_id"]["type"], "string");
    assert_eq!(nested_props["description"]["type"], "string");
    assert_eq!(nested_props["lore_entries"]["type"], "array");
    assert_eq!(nested_props["lore_entries"]["items"]["type"], "string");

    // Check required fields in nested struct
    let nested_required = items_schema["required"].as_array().unwrap();
    assert_eq!(nested_required.len(), 3);
}

// Test matching the exact pattern from the issues document
#[derive(Debug, Clone, Serialize, Deserialize, StructuredOutput)]
#[structured_output(
    name = "loot_placement",
    description = "Loot placement in a room"
)]
struct LootPlacement {
    room_id: String,
    loot_type: String,
    rarity: String,
    theme_hints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, StructuredOutput)]
#[structured_output(
    name = "population_plan",
    description = "Population plan for an area"
)]
struct PopulationPlan {
    loot_placements: Vec<LootPlacement>,
    boss_room_id: Option<String>,
}

#[test]
fn test_complex_nested_struct() {
    let schema = PopulationPlan::json_schema();
    let properties = &schema["properties"];

    // Check loot_placements is properly structured
    assert_eq!(properties["loot_placements"]["type"], "array");

    let items = &properties["loot_placements"]["items"];
    assert_eq!(items["type"], "object", "Nested LootPlacement should be object");

    let loot_props = &items["properties"];
    assert_eq!(loot_props["room_id"]["type"], "string");
    assert_eq!(loot_props["loot_type"]["type"], "string");
    assert_eq!(loot_props["rarity"]["type"], "string");
    assert_eq!(loot_props["theme_hints"]["type"], "array");
    assert_eq!(loot_props["theme_hints"]["items"]["type"], "string");

    // Print schema for debugging
    println!("Generated schema: {}", serde_json::to_string_pretty(&schema).unwrap());
}

// Test for optional types to verify they are not added to required fields
#[derive(Debug, Serialize, Deserialize, StructuredOutput)]
#[structured_output(
    name = "optional_types",
    description = "Test various optional field types"
)]
struct OptionalTypes {
    required_field: String,
    optional_string: Option<String>,
    optional_integer: Option<i32>,
    optional_number: Option<f64>,
    optional_boolean: Option<bool>,
    optional_vec: Option<Vec<String>>,
}

#[test]
fn test_optional_types() {
    let schema = OptionalTypes::json_schema();
    let properties = &schema["properties"];
    let required = schema["required"].as_array().unwrap();

    // Verify only required_field is in the required list
    assert_eq!(
        required.len(),
        1,
        "Only one field should be required, but found: {:?}",
        required
    );
    assert_eq!(required[0], "required_field");

    // Verify optional fields are NOT in the required list
    let required_names: Vec<String> = required.iter().map(|v| v.as_str().unwrap().to_string()).collect();
    assert!(!required_names.contains(&"optional_string".to_string()));
    assert!(!required_names.contains(&"optional_integer".to_string()));
    assert!(!required_names.contains(&"optional_number".to_string()));
    assert!(!required_names.contains(&"optional_boolean".to_string()));
    assert!(!required_names.contains(&"optional_vec".to_string()));

    // Verify optional fields have correct schema types (unwrapped from Option)
    assert_eq!(properties["optional_string"]["type"], "string");
    assert_eq!(properties["optional_integer"]["type"], "integer");
    assert_eq!(properties["optional_number"]["type"], "number");
    assert_eq!(properties["optional_boolean"]["type"], "boolean");
    assert_eq!(properties["optional_vec"]["type"], "array");
    assert_eq!(properties["optional_vec"]["items"]["type"], "string");

    // Verify all properties exist in the schema
    assert!(properties.is_object());
    assert!(properties["required_field"].is_object());
    assert!(properties["optional_string"].is_object());
    assert!(properties["optional_integer"].is_object());
    assert!(properties["optional_number"].is_object());
    assert!(properties["optional_boolean"].is_object());
    assert!(properties["optional_vec"].is_object());
}
