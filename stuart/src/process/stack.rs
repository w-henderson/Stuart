use humphrey_json::Value;

pub struct StackFrame {
    pub name: String,
    pub variables: Vec<(String, Value)>,
    pub output: Vec<u8>,
}

impl StackFrame {
    pub fn new(name: impl AsRef<str>) -> Self {
        Self {
            name: name.as_ref().to_string(),
            variables: Vec::new(),
            output: Vec::new(),
        }
    }

    pub fn add_variable(&mut self, name: impl AsRef<str>, value: Value) {
        self.variables.push((name.as_ref().to_string(), value));
    }

    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v)
    }
}

pub fn get_value(index: &[&str], json: &Value) -> Value {
    let mut current_json = json;

    for &index in index {
        match current_json.get(index) {
            Some(value) => current_json = value,
            None => return Value::Null,
        }
    }

    current_json.clone()
}
