use serde::{Deserialize, Serialize};
use struct_llm::{parse_tool_response, StructuredOutput, ToolCall};

#[derive(Debug, Serialize, Deserialize, StructuredOutput)]
#[structured_output(
    name = "test_validation",
    description = "Test validation"
)]
struct ValidatedOutput {
    name: String,
    age: u32,
    email: String,
}

#[test]
fn test_validation_success() {
    let tool_call = ToolCall {
        id: "test_123".to_string(),
        name: "test_validation".to_string(),
        arguments: serde_json::json!({
            "name": "Alice",
            "age": 30,
            "email": "alice@example.com"
        }),
    };

    let result: ValidatedOutput = parse_tool_response(&tool_call).unwrap();
    assert_eq!(result.name, "Alice");
    assert_eq!(result.age, 30);
    assert_eq!(result.email, "alice@example.com");
}

#[test]
fn test_validation_missing_required() {
    let tool_call = ToolCall {
        id: "test_123".to_string(),
        name: "test_validation".to_string(),
        arguments: serde_json::json!({
            "name": "Alice",
            // Missing age
            "email": "alice@example.com"
        }),
    };

    let result: Result<ValidatedOutput, _> = parse_tool_response(&tool_call);
    assert!(result.is_err());

    // Should fail during validation (missing required field)
    let err = result.unwrap_err();
    assert!(matches!(err, struct_llm::Error::ValidationFailed(_)));
}

#[test]
fn test_validation_wrong_type() {
    let tool_call = ToolCall {
        id: "test_123".to_string(),
        name: "test_validation".to_string(),
        arguments: serde_json::json!({
            "name": "Alice",
            "age": "thirty", // Wrong type - should be number
            "email": "alice@example.com"
        }),
    };

    let result: Result<ValidatedOutput, _> = parse_tool_response(&tool_call);
    assert!(result.is_err());

    // Should fail during validation (wrong type)
    let err = result.unwrap_err();
    assert!(matches!(err, struct_llm::Error::ValidationFailed(_)));
}

#[test]
fn test_validation_tool_mismatch() {
    let tool_call = ToolCall {
        id: "test_123".to_string(),
        name: "wrong_tool_name".to_string(), // Wrong tool name
        arguments: serde_json::json!({
            "name": "Alice",
            "age": 30,
            "email": "alice@example.com"
        }),
    };

    let result: Result<ValidatedOutput, _> = parse_tool_response(&tool_call);
    assert!(result.is_err());

    // Should fail with tool mismatch
    let err = result.unwrap_err();
    assert!(matches!(err, struct_llm::Error::ToolMismatch(_, _)));
}
