use humphrey_json::Value;

pub fn json_to_js<'a>(
    value: Option<Value>,
    scope: &mut v8::HandleScope<'a>,
) -> v8::Local<'a, v8::Value> {
    match value {
        Some(v) => match v {
            Value::Null => v8::null(scope).into(),
            Value::Bool(boolean) => v8::Boolean::new(scope, boolean).into(),
            Value::Number(number) => v8::Number::new(scope, number).into(),
            Value::String(string) => v8::String::new(scope, &string).unwrap().into(),
            Value::Array(array) => {
                let v8_array = v8::Array::new(scope, array.len() as i32);
                for (i, value) in array.into_iter().enumerate() {
                    let v8_value = json_to_js(Some(value), scope);
                    v8_array.set_index(scope, i as u32, v8_value);
                }
                v8_array.into()
            }
            Value::Object(object) => {
                let v8_object = v8::Object::new(scope);
                for (key, value) in object.into_iter() {
                    let v8_key = v8::String::new(scope, &key).unwrap().into();
                    let v8_value = json_to_js(Some(value), scope);
                    v8_object.set(scope, v8_key, v8_value);
                }
                v8_object.into()
            }
        },
        None => v8::undefined(scope).into(),
    }
}

pub fn js_to_json<'a>(
    value: v8::Local<'a, v8::Value>,
    scope: &mut v8::HandleScope<'a>,
) -> Option<Value> {
    if value.is_undefined() {
        return None;
    }

    if value.is_null() {
        return Some(Value::Null);
    }

    if value.is_boolean() {
        return Some(Value::Bool(value.boolean_value(scope)));
    }

    if value.is_number() {
        return Some(Value::Number(value.number_value(scope).unwrap()));
    }

    if value.is_string() {
        return Some(Value::String(value.to_rust_string_lossy(scope).to_string()));
    }

    if value.is_array() {
        let v8_array = value.to_object(scope).unwrap();
        let k_length = v8::String::new(scope, "length").unwrap();
        let length = v8_array
            .get(scope, k_length.into())
            .unwrap()
            .uint32_value(scope)
            .unwrap();
        let mut array = Vec::with_capacity(length as usize);
        for i in 0..length {
            let v8_value = v8_array.get_index(scope, i).unwrap();
            let value = js_to_json(v8_value, scope);
            array.push(value.unwrap());
        }
        return Some(Value::Array(array));
    }

    if value.is_object() {
        let v8_object = value.to_object(scope).unwrap();
        let keys = v8_object
            .get_own_property_names(scope, v8::GetPropertyNamesArgs::default())
            .unwrap();
        let length = keys.length();
        let mut object = Vec::with_capacity(length as usize);

        for i in 0..length {
            let v8_key = keys.get_index(scope, i).unwrap();
            let key = v8_key.to_rust_string_lossy(scope).to_string();
            let v8_value = v8_object.get(scope, v8_key.into()).unwrap();
            let value = js_to_json(v8_value, scope);
            object.push((key, value.unwrap()));
        }

        return Some(Value::Object(object));
    }

    None
}
