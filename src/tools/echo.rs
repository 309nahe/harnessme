use serde_json::json;

use crate::core::tool::{Tool, ToolError};

/// A simple reference tool that echoes back the input message.
#[derive(Debug, Default, Clone, Copy)]
pub struct EchoTool;

impl EchoTool {
    /// Creates a new `EchoTool` instance.
    pub fn new() -> Self {
        Self
    }
}

impl Tool for EchoTool {
    fn name(&self) -> &str {
        "echo"
    }

    fn description(&self) -> &str {
        "Echoes back the provided input string message."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "The message to echo back."
                }
            },
            "required": ["message"]
        })
    }

    fn execute(&self, args: serde_json::Value) -> Result<String, ToolError> {
        let message = args
            .get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ToolError::InvalidArguments("Missing required parameter 'message'".to_string())
            })?;

        Ok(message.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_echo_tool_execution() {
        let echo = EchoTool::new();
        assert_eq!(echo.name(), "echo");
        assert!(!echo.description().is_empty());

        let res = echo.execute(json!({ "message": "Hello, World!" })).unwrap();
        assert_eq!(res, "Hello, World!");

        let err = echo.execute(json!({})).unwrap_err();
        match err {
            ToolError::InvalidArguments(msg) => {
                assert!(msg.contains("Missing required parameter 'message'"))
            }
            other => panic!("Expected InvalidArguments, got {:?}", other),
        }

        // Invalid type (e.g. number instead of string)
        let err_type = echo.execute(json!({"message": 12345})).unwrap_err();
        match err_type {
            ToolError::InvalidArguments(msg) => {
                assert!(msg.contains("Missing required parameter 'message'"))
            }
            other => panic!("Expected InvalidArguments, got {:?}", other),
        }
    }
}
