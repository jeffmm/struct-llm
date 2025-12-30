//! JSON Schema generation and validation utilities

use crate::error::{Error, Result};
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

/// Validate a JSON value against a JSON Schema
///
/// This is a basic validator that checks common schema constraints:
/// - Type checking (string, number, integer, boolean, array, object)
/// - Required properties
/// - Enum values
/// - Array items
///
/// Note: This is not a complete JSON Schema validator. For production use,
/// consider using a dedicated JSON Schema validation library.
pub fn validate(value: &Value, schema: &Value) -> Result<()> {
    // Check type
    if let Some(schema_type) = schema.get("type").and_then(|t| t.as_str()) {
        match schema_type {
            "string" => {
                if !value.is_string() {
                    return Err(Error::ValidationFailed(format!(
                        "Expected string, got {:?}",
                        value
                    )));
                }

                // Check enum if present
                if let Some(enum_values) = schema.get("enum") {
                    if let Some(enum_array) = enum_values.as_array() {
                        if !enum_array.contains(value) {
                            return Err(Error::ValidationFailed(format!(
                                "Value {:?} not in allowed enum values",
                                value
                            )));
                        }
                    }
                }
            }
            "number" => {
                if !value.is_f64() && !value.is_i64() && !value.is_u64() {
                    return Err(Error::ValidationFailed(format!(
                        "Expected number, got {:?}",
                        value
                    )));
                }
            }
            "integer" => {
                if !value.is_i64() && !value.is_u64() {
                    return Err(Error::ValidationFailed(format!(
                        "Expected integer, got {:?}",
                        value
                    )));
                }
            }
            "boolean" => {
                if !value.is_boolean() {
                    return Err(Error::ValidationFailed(format!(
                        "Expected boolean, got {:?}",
                        value
                    )));
                }
            }
            "array" => {
                if !value.is_array() {
                    return Err(Error::ValidationFailed(format!(
                        "Expected array, got {:?}",
                        value
                    )));
                }

                // Validate items if schema specifies
                if let Some(items_schema) = schema.get("items") {
                    if let Some(array) = value.as_array() {
                        for item in array {
                            validate(item, items_schema)?;
                        }
                    }
                }
            }
            "object" => {
                if !value.is_object() {
                    return Err(Error::ValidationFailed(format!(
                        "Expected object, got {:?}",
                        value
                    )));
                }

                // Check required properties
                if let Some(required) = schema.get("required") {
                    if let Some(required_array) = required.as_array() {
                        if let Some(obj) = value.as_object() {
                            for req_prop in required_array {
                                if let Some(prop_name) = req_prop.as_str() {
                                    if !obj.contains_key(prop_name) {
                                        return Err(Error::ValidationFailed(format!(
                                            "Missing required property: {}",
                                            prop_name
                                        )));
                                    }
                                }
                            }
                        }
                    }
                }

                // Validate properties if schema specifies
                if let Some(properties) = schema.get("properties") {
                    if let Some(props_obj) = properties.as_object() {
                        if let Some(value_obj) = value.as_object() {
                            for (prop_name, prop_value) in value_obj {
                                if let Some(prop_schema) = props_obj.get(prop_name) {
                                    validate(prop_value, prop_schema)?;
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                // Unknown type, skip validation
            }
        }
    }

    Ok(())
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

    #[test]
    fn test_validate_string() {
        let schema = SchemaBuilder::string();
        let value = json!("hello");

        assert!(validate(&value, &schema).is_ok());

        let invalid = json!(123);
        assert!(validate(&invalid, &schema).is_err());
    }

    #[test]
    fn test_validate_enum() {
        let schema = SchemaBuilder::string_enum(&["positive", "negative", "neutral"]);
        let value = json!("positive");

        assert!(validate(&value, &schema).is_ok());

        let invalid = json!("unknown");
        assert!(validate(&invalid, &schema).is_err());
    }

    #[test]
    fn test_validate_integer() {
        let schema = SchemaBuilder::integer();
        let value = json!(42);

        assert!(validate(&value, &schema).is_ok());

        let invalid = json!(3.14);
        assert!(validate(&invalid, &schema).is_err());
    }

    #[test]
    fn test_validate_array() {
        let schema = SchemaBuilder::array(SchemaBuilder::string());
        let value = json!(["a", "b", "c"]);

        assert!(validate(&value, &schema).is_ok());

        let invalid_type = json!("not an array");
        assert!(validate(&invalid_type, &schema).is_err());

        let invalid_items = json!([1, 2, 3]);
        assert!(validate(&invalid_items, &schema).is_err());
    }

    #[test]
    fn test_validate_object() {
        let schema = SchemaBuilder::object(
            json!({
                "name": SchemaBuilder::string(),
                "age": SchemaBuilder::integer()
            }),
            &["name"],
        );

        let value = json!({
            "name": "Alice",
            "age": 30
        });
        assert!(validate(&value, &schema).is_ok());

        // Missing required field
        let missing_required = json!({
            "age": 30
        });
        assert!(validate(&missing_required, &schema).is_err());

        // Wrong property type
        let wrong_type = json!({
            "name": "Alice",
            "age": "thirty"
        });
        assert!(validate(&wrong_type, &schema).is_err());
    }

    #[test]
    fn test_validate_nested() {
        let schema = SchemaBuilder::object(
            json!({
                "user": SchemaBuilder::object(
                    json!({
                        "name": SchemaBuilder::string(),
                        "tags": SchemaBuilder::array(SchemaBuilder::string())
                    }),
                    &["name"]
                )
            }),
            &["user"],
        );

        let value = json!({
            "user": {
                "name": "Bob",
                "tags": ["admin", "user"]
            }
        });
        assert!(validate(&value, &schema).is_ok());

        let invalid = json!({
            "user": {
                "name": "Bob",
                "tags": [1, 2, 3]
            }
        });
        assert!(validate(&invalid, &schema).is_err());
    }
}
