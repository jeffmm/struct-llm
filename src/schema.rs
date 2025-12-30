//! JSON Schema generation utilities

use serde_json::{json, Value};

/// Helper to build JSON schema for common Rust types
pub struct SchemaBuilder;

impl SchemaBuilder {
    /// Create a string schema
    pub fn string() -> Value {
        json!({ "type": "string" })
    }

    /// Create a string schema with enum values
    pub fn string_enum(values: &[&str]) -> Value {
        json!({
            "type": "string",
            "enum": values
        })
    }

    /// Create a number schema
    pub fn number() -> Value {
        json!({ "type": "number" })
    }

    /// Create an integer schema
    pub fn integer() -> Value {
        json!({ "type": "integer" })
    }

    /// Create a boolean schema
    pub fn boolean() -> Value {
        json!({ "type": "boolean" })
    }

    /// Create an array schema
    pub fn array(items: Value) -> Value {
        json!({
            "type": "array",
            "items": items
        })
    }

    /// Create an object schema
    pub fn object(properties: Value, required: &[&str]) -> Value {
        json!({
            "type": "object",
            "properties": properties,
            "required": required
        })
    }

    /// Add description to a schema
    pub fn with_description(mut schema: Value, description: &str) -> Value {
        if let Some(obj) = schema.as_object_mut() {
            obj.insert("description".to_string(), json!(description));
        }
        schema
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_builder() {
        let schema = SchemaBuilder::string();
        assert_eq!(schema["type"], "string");

        let schema = SchemaBuilder::string_enum(&["positive", "negative", "neutral"]);
        assert_eq!(schema["type"], "string");
        assert!(schema["enum"].is_array());
        assert_eq!(schema["enum"].as_array().unwrap().len(), 3);

        let schema = SchemaBuilder::object(
            json!({
                "name": SchemaBuilder::string(),
                "age": SchemaBuilder::integer()
            }),
            &["name"],
        );
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"].is_object());
        assert_eq!(schema["required"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_with_description() {
        let schema = SchemaBuilder::with_description(
            SchemaBuilder::string(),
            "A user's name",
        );
        assert_eq!(schema["description"], "A user's name");
    }
}
