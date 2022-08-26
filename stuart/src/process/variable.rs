use humphrey_json::Value;

pub struct StackFrame {
    pub variables: Vec<(String, Value)>,
}

impl StackFrame {
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
