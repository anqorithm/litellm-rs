//! Tests for function calling module

#[cfg(test)]
use super::builtin::{CalculatorFunction, WeatherFunction};
use super::executor::{FunctionCallingHandler, FunctionExecutor};
use serde_json::json;

#[test]
fn test_function_calling_handler_creation() {
    let handler = FunctionCallingHandler::new();
    assert!(handler.functions.is_empty());
    assert!(handler.executors.is_empty());
}

#[tokio::test]
async fn test_weather_function() {
    let weather_fn = WeatherFunction;
    let args = json!({"location": "San Francisco, CA"});

    let result = weather_fn.execute(args).await.unwrap();
    assert!(result.get("location").is_some());
    assert!(result.get("temperature").is_some());
}

#[tokio::test]
async fn test_calculator_function() {
    let calc_fn = CalculatorFunction;
    let args = json!({"expression": "2 + 3"});

    let result = calc_fn.execute(args).await.unwrap();
    assert_eq!(result.get("result").unwrap().as_f64().unwrap(), 5.0);
}

#[test]
fn test_function_registration() {
    let mut handler = FunctionCallingHandler::new();
    let weather_fn = WeatherFunction;

    handler
        .register_function("get_weather".to_string(), weather_fn)
        .unwrap();
    assert_eq!(handler.functions.len(), 1);
    assert_eq!(handler.executors.len(), 1);
}

#[test]
fn test_tool_definitions() {
    let mut handler = FunctionCallingHandler::new();
    let weather_fn = WeatherFunction;

    handler
        .register_function("get_weather".to_string(), weather_fn)
        .unwrap();
    let tools = handler.get_tool_definitions();

    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].function.name, "get_weather");
    assert_eq!(tools[0].tool_type, "function");
}
