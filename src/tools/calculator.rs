use serde_json::json;

use crate::core::tool::{Tool, ToolError};

/// Reference calculator tool capable of performing arithmetic operations.
#[derive(Debug, Default, Clone, Copy)]
pub struct CalculatorTool;

impl CalculatorTool {
    /// Creates a new `CalculatorTool` instance.
    pub fn new() -> Self {
        Self
    }

    fn calculate(a: f64, b: f64, op: &str) -> Result<f64, ToolError> {
        match op.trim().to_lowercase().as_str() {
            "add" | "+" => Ok(a + b),
            "subtract" | "sub" | "-" => Ok(a - b),
            "multiply" | "mul" | "*" => Ok(a * b),
            "divide" | "div" | "/" => {
                if b == 0.0 {
                    Err(ToolError::ExecutionFailed(
                        "Division by zero is undefined".to_string(),
                    ))
                } else {
                    Ok(a / b)
                }
            }
            unknown => Err(ToolError::InvalidArguments(format!(
                "Unsupported operation '{unknown}'. Supported: add (+), subtract (-), multiply (*), divide (/)"
            ))),
        }
    }
}

impl Tool for CalculatorTool {
    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "Performs basic arithmetic operations (add, subtract, multiply, divide) on two numbers."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "a": {
                    "type": "number",
                    "description": "The first number (left operand)."
                },
                "b": {
                    "type": "number",
                    "description": "The second number (right operand)."
                },
                "op": {
                    "type": "string",
                    "enum": ["add", "subtract", "multiply", "divide", "+", "-", "*", "/"],
                    "description": "The arithmetic operation to perform."
                }
            },
            "required": ["a", "b", "op"]
        })
    }

    fn execute(&self, args: serde_json::Value) -> Result<String, ToolError> {
        let a = args.get("a").and_then(|v| v.as_f64()).ok_or_else(|| {
            ToolError::InvalidArguments("Missing or invalid numeric parameter 'a'".to_string())
        })?;

        let b = args.get("b").and_then(|v| v.as_f64()).ok_or_else(|| {
            ToolError::InvalidArguments("Missing or invalid numeric parameter 'b'".to_string())
        })?;

        let op = args.get("op").and_then(|v| v.as_str()).ok_or_else(|| {
            ToolError::InvalidArguments("Missing or invalid string parameter 'op'".to_string())
        })?;

        let result = Self::calculate(a, b, op)?;
        Ok(result.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculator_operations() {
        let calc = CalculatorTool::new();

        // Addition
        assert_eq!(
            calc.execute(json!({"a": 12.5, "b": 7.5, "op": "add"}))
                .unwrap(),
            "20"
        );
        assert_eq!(
            calc.execute(json!({"a": 10, "b": 5, "op": "+"})).unwrap(),
            "15"
        );

        // Subtraction
        assert_eq!(
            calc.execute(json!({"a": 10, "b": 4, "op": "subtract"}))
                .unwrap(),
            "6"
        );
        assert_eq!(
            calc.execute(json!({"a": 10, "b": 4, "op": "-"})).unwrap(),
            "6"
        );

        // Multiplication
        assert_eq!(
            calc.execute(json!({"a": 6, "b": 7, "op": "multiply"}))
                .unwrap(),
            "42"
        );
        assert_eq!(
            calc.execute(json!({"a": 6, "b": 7, "op": "*"})).unwrap(),
            "42"
        );

        // Division
        assert_eq!(
            calc.execute(json!({"a": 20, "b": 4, "op": "divide"}))
                .unwrap(),
            "5"
        );
        assert_eq!(
            calc.execute(json!({"a": 20, "b": 4, "op": "/"})).unwrap(),
            "5"
        );

        // Division by zero
        let err_div = calc
            .execute(json!({"a": 10, "b": 0, "op": "divide"}))
            .unwrap_err();
        match err_div {
            ToolError::ExecutionFailed(msg) => assert!(msg.contains("Division by zero")),
            other => panic!("Expected ExecutionFailed, got {:?}", other),
        }

        // Invalid operation
        let err_op = calc
            .execute(json!({"a": 10, "b": 5, "op": "modulo"}))
            .unwrap_err();
        match err_op {
            ToolError::InvalidArguments(msg) => assert!(msg.contains("Unsupported operation")),
            other => panic!("Expected InvalidArguments, got {:?}", other),
        }
    }
}
