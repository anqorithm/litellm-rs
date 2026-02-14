//! Built-in function executors

use super::executor::FunctionExecutor;
use super::types::FunctionDefinition;
use crate::utils::error::gateway_error::{GatewayError, Result};
use serde_json::{Value, json};

/// Weather function executor (example)
pub struct WeatherFunction;

#[async_trait::async_trait]
impl FunctionExecutor for WeatherFunction {
    async fn execute(&self, arguments: Value) -> Result<Value> {
        let location = arguments
            .get("location")
            .and_then(|l| l.as_str())
            .ok_or_else(|| GatewayError::Validation("Missing location parameter".to_string()))?;

        // Mock weather data
        let weather_data = json!({
            "location": location,
            "temperature": "22°C",
            "condition": "Sunny",
            "humidity": "65%",
            "wind": "10 km/h"
        });

        Ok(weather_data)
    }

    fn get_schema(&self) -> FunctionDefinition {
        FunctionDefinition {
            name: "get_weather".to_string(),
            description: Some("Get current weather information for a location".to_string()),
            parameters: json!({
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "The city and state, e.g. San Francisco, CA"
                    }
                },
                "required": ["location"]
            }),
            strict: Some(false),
        }
    }

    fn validate_arguments(&self, arguments: &Value) -> Result<()> {
        if !arguments.is_object() {
            return Err(GatewayError::Validation(
                "Arguments must be an object".to_string(),
            ));
        }

        if arguments.get("location").is_none() {
            return Err(GatewayError::Validation(
                "Missing required parameter: location".to_string(),
            ));
        }

        Ok(())
    }
}

/// Calculator function executor (example)
pub struct CalculatorFunction;

#[async_trait::async_trait]
impl FunctionExecutor for CalculatorFunction {
    async fn execute(&self, arguments: Value) -> Result<Value> {
        let expression = arguments
            .get("expression")
            .and_then(|e| e.as_str())
            .ok_or_else(|| GatewayError::Validation("Missing expression parameter".to_string()))?;

        // Simple calculator (in real implementation, use a proper expression parser)
        let result = match expression {
            expr if expr.contains("+") => {
                let parts: Vec<&str> = expr.split('+').collect();
                if parts.len() == 2 {
                    let a: f64 = parts[0]
                        .trim()
                        .parse()
                        .map_err(|_| GatewayError::Validation("Invalid number".to_string()))?;
                    let b: f64 = parts[1]
                        .trim()
                        .parse()
                        .map_err(|_| GatewayError::Validation("Invalid number".to_string()))?;
                    a + b
                } else {
                    return Err(GatewayError::Validation("Invalid expression".to_string()));
                }
            }
            _ => {
                return Err(GatewayError::Validation(
                    "Unsupported operation".to_string(),
                ));
            }
        };

        Ok(json!({
            "expression": expression,
            "result": result
        }))
    }

    fn get_schema(&self) -> FunctionDefinition {
        FunctionDefinition {
            name: "calculate".to_string(),
            description: Some("Perform basic mathematical calculations".to_string()),
            parameters: json!({
                "type": "object",
                "properties": {
                    "expression": {
                        "type": "string",
                        "description": "Mathematical expression to evaluate (e.g., '2 + 3')"
                    }
                },
                "required": ["expression"]
            }),
            strict: Some(false),
        }
    }
}
