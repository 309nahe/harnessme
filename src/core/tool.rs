use std::collections::HashMap;
use std::fmt;

use crate::core::types::{FunctionDefinition, ToolDefinition};

/// Domain-specific errors that can occur during tool lookup, argument parsing, or execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolError {
    /// Arguments provided by the LLM were malformed or failed schema validation.
    InvalidArguments(String),
    /// Tool execution encountered a runtime failure.
    ExecutionFailed(String),
    /// Requested tool name was not found in the registry.
    ToolNotFound(String),
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArguments(msg) => write!(f, "Invalid tool arguments: {msg}"),
            Self::ExecutionFailed(msg) => write!(f, "Tool execution failed: {msg}"),
            Self::ToolNotFound(name) => write!(f, "Tool not found: '{name}'"),
        }
    }
}

impl std::error::Error for ToolError {}

/// Trait implemented by capabilities exposed to the LLM.
pub trait Tool: Send + Sync {
    /// Unique identifier for the tool (e.g., `"calculator"`, `"echo"`).
    fn name(&self) -> &str;

    /// Human/LLM-readable description explaining what the tool does and when to use it.
    fn description(&self) -> &str;

    /// JSON Schema describing the accepted input parameter object.
    fn parameters_schema(&self) -> serde_json::Value;

    /// Executes the tool given parsed JSON arguments.
    fn execute(&self, args: serde_json::Value) -> Result<String, ToolError>;
}

/// In-memory registry for managing, introspecting, and executing tools.
#[derive(Default)]
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    /// Creates a new, empty tool registry.
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Registers a tool in the registry.
    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        self.tools.insert(tool.name().to_string(), Box::new(tool));
    }

    /// Retrieves a reference to a registered tool by its name.
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|b| b.as_ref())
    }

    /// Returns the number of registered tools.
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Returns true if no tools are registered.
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// Exports all registered tools as provider-ready `ToolDefinition` schemas.
    pub fn definitions(&self) -> Vec<ToolDefinition> {
        let mut defs: Vec<ToolDefinition> = self
            .tools
            .values()
            .map(|tool| ToolDefinition {
                r#type: "function".to_string(),
                function: FunctionDefinition {
                    name: tool.name().to_string(),
                    description: tool.description().to_string(),
                    parameters: tool.parameters_schema(),
                },
            })
            .collect();
        // Sort deterministically by name
        defs.sort_by(|a, b| a.function.name.cmp(&b.function.name));
        defs
    }

    /// Parses raw JSON argument string and executes the named tool.
    pub fn execute(&self, name: &str, raw_args: &str) -> Result<String, ToolError> {
        let tool = self
            .get(name)
            .ok_or_else(|| ToolError::ToolNotFound(name.to_string()))?;

        let parsed_args: serde_json::Value = if raw_args.trim().is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_str(raw_args).map_err(|err| {
                ToolError::InvalidArguments(format!("Failed to parse JSON arguments: {err}"))
            })?
        };

        tool.execute(parsed_args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    struct DummyTool;
    impl Tool for DummyTool {
        fn name(&self) -> &str {
            "dummy"
        }
        fn description(&self) -> &str {
            "A test dummy tool"
        }
        fn parameters_schema(&self) -> serde_json::Value {
            json!({
                "type": "object",
                "properties": {
                    "input": { "type": "string" }
                },
                "required": ["input"]
            })
        }
        fn execute(&self, args: serde_json::Value) -> Result<String, ToolError> {
            let input = args
                .get("input")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::InvalidArguments("Missing 'input' field".to_string()))?;
            Ok(format!("dummy output: {input}"))
        }
    }

    #[test]
    fn test_registry_registration_and_execution() {
        let mut registry = ToolRegistry::new();
        assert!(registry.is_empty());

        registry.register(DummyTool);
        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());
        assert!(registry.get("dummy").is_some());
        assert!(registry.get("nonexistent").is_none());

        // Successful execution
        let res = registry.execute("dummy", "{\"input\": \"hello\"}").unwrap();
        assert_eq!(res, "dummy output: hello");

        // Missing argument error
        let err_args = registry.execute("dummy", "{}").unwrap_err();
        match err_args {
            ToolError::InvalidArguments(msg) => assert!(msg.contains("Missing 'input' field")),
            other => panic!("Expected InvalidArguments, got {:?}", other),
        }

        // Malformed JSON error
        let err_json = registry.execute("dummy", "{invalid_json}").unwrap_err();
        match err_json {
            ToolError::InvalidArguments(msg) => assert!(msg.contains("Failed to parse JSON")),
            other => panic!("Expected InvalidArguments, got {:?}", other),
        }

        // Unknown tool error
        let err_not_found = registry.execute("unknown", "{}").unwrap_err();
        match err_not_found {
            ToolError::ToolNotFound(name) => assert_eq!(name, "unknown"),
            other => panic!("Expected ToolNotFound, got {:?}", other),
        }
    }

    #[test]
    fn test_definitions_export() {
        let mut registry = ToolRegistry::new();
        registry.register(DummyTool);

        let defs = registry.definitions();
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].r#type, "function");
        assert_eq!(defs[0].function.name, "dummy");
        assert_eq!(defs[0].function.description, "A test dummy tool");
    }

    #[test]
    fn test_tool_error_display() {
        let err1 = ToolError::InvalidArguments("bad type".to_string());
        assert_eq!(err1.to_string(), "Invalid tool arguments: bad type");

        let err2 = ToolError::ExecutionFailed("division by zero".to_string());
        assert_eq!(err2.to_string(), "Tool execution failed: division by zero");

        let err3 = ToolError::ToolNotFound("search".to_string());
        assert_eq!(err3.to_string(), "Tool not found: 'search'");
    }
}
