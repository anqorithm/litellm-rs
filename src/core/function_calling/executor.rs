//! Function executor trait and handler implementation

use super::types::*;
use crate::utils::error::error::Result;
use serde_json::Value;
use std::collections::HashMap;

/// Trait for executing functions
#[async_trait::async_trait]
pub trait FunctionExecutor: Send + Sync {
    /// Execute the function with given arguments
    async fn execute(&self, arguments: Value) -> Result<Value>;

    /// Get function schema
    fn get_schema(&self) -> FunctionDefinition;

    /// Validate function arguments
    fn validate_arguments(&self, _arguments: &Value) -> Result<()> {
        // Default implementation - can be overridden
        Ok(())
    }
}

/// Function calling handler
pub struct FunctionCallingHandler {
    /// Available functions
    pub(crate) functions: HashMap<String, FunctionDefinition>,
    /// Function execution handlers
    pub(crate) executors: HashMap<String, Box<dyn FunctionExecutor>>,
}

impl FunctionCallingHandler {
    /// Create a new function calling handler
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            executors: HashMap::new(),
        }
    }

    /// Register a function
    pub fn register_function<F>(&mut self, name: String, executor: F) -> Result<()>
    where
        F: FunctionExecutor + 'static,
    {
        let schema = executor.get_schema();
        self.functions.insert(name.clone(), schema);
        self.executors.insert(name, Box::new(executor));
        Ok(())
    }

    /// Get available functions as tool definitions
    pub fn get_tool_definitions(&self) -> Vec<ToolDefinition> {
        self.functions
            .values()
            .map(|function| ToolDefinition {
                tool_type: "function".to_string(),
                function: function.clone(),
            })
            .collect()
    }
}

impl Default for FunctionCallingHandler {
    fn default() -> Self {
        Self::new()
    }
}
