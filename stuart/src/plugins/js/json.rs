use humphrey_json::Value;

pub fn json_to_js<'a>(
    value: Value,
    scope: &mut v8::ContextScope<v8::HandleScope<'a>>,
) -> v8::Local<'a, v8::Value> {
    match value {
        Value::Null => v8::null(scope).into(),
        Value::Bool(boolean) => v8::Boolean::new(scope, boolean).into(),
        Value::Number(number) => v8::Number::new(scope, number).into(),
        Value::String(string) => v8::String::new(scope, &string).unwrap().into(),
        Value::Array(array) => {
            let v8_array = v8::Array::new(scope, array.len() as i32);
            for (i, value) in array.into_iter().enumerate() {
                let v8_value = json_to_js(value, scope);
                v8_array.set_index(scope, i as u32, v8_value);
            }
            v8_array.into()
        }
        Value::Object(object) => {
            let v8_object = v8::Object::new(scope);
            for (key, value) in object.into_iter() {
                let v8_key = v8::String::new(scope, &key).unwrap().into();
                let v8_value = json_to_js(value, scope);
                v8_object.set(scope, v8_key, v8_value);
            }
            v8_object.into()
        }
    }
}
