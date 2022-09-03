//! Provides a basic call stack implementation.

use humphrey_json::Value;

/// Represents a stack frame.
///
/// When the stack frame is popped, the output of the frame is appended to the output of the frame below it.
#[derive(Debug)]
pub struct StackFrame {
    /// The name of the stack frame, used for identification.
    pub name: String,
    /// Variables in the stack frame.
    pub variables: Vec<(String, Value)>,
    /// The output of the stack frame.
    pub output: Vec<u8>,
}

impl StackFrame {
    /// Creates a new stack frame with the given name.
    pub fn new(name: impl AsRef<str>) -> Self {
        Self {
            name: name.as_ref().to_string(),
            variables: Vec::new(),
            output: Vec::new(),
        }
    }

    /// Adds a variable to the stack frame.
    pub fn add_variable(&mut self, name: impl AsRef<str>, value: Value) {
        self.variables.push((name.as_ref().to_string(), value));
    }

    /// Returns the value of the variable with the given name.
    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v)
    }
}

/// Gets a value from inside a JSON object.
pub fn get_value(index: &[&str], json: &Value) -> Value {
    let mut current_json = json;

    for &index in index {
        match current_json.get(index) {
            Some(value) => current_json = value,
            None => match index
                .parse::<usize>()
                .ok()
                .and_then(|i| current_json.get(i))
            {
                Some(value) => current_json = value,
                None => return Value::Null,
            },
        }
    }

    current_json.clone()
}
