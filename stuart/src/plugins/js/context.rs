use stuart_core::process::Scope;

/// Makes the Stuart scope accessible to `set_variable` and `get_variable` when they're called from JavaScript code.
pub fn set_stuart_context(scope: &mut v8::HandleScope, context: &mut Scope) {
    let stuart_context = v8::Object::new(scope);

    let k_context = v8::String::new(scope, "STUART").unwrap();
    let k_set_variable = v8::String::new(scope, "set").unwrap();
    let k_get_variable = v8::String::new(scope, "get").unwrap();
    let k_external = v8::String::new(scope, "_ptr").unwrap();

    let set_variable = v8::FunctionTemplate::new(scope, set_variable)
        .get_function(scope)
        .unwrap();
    let get_variable = v8::FunctionTemplate::new(scope, get_variable)
        .get_function(scope)
        .unwrap();
    let external = v8::External::new(scope, context as *mut _ as *mut std::ffi::c_void);

    stuart_context.set(scope, k_set_variable.into(), set_variable.into());
    stuart_context.set(scope, k_get_variable.into(), get_variable.into());
    stuart_context.set(scope, k_external.into(), external.into());

    scope
        .get_current_context()
        .global(scope)
        .set(scope, k_context.into(), stuart_context.into());
}

unsafe fn get_stuart_context<'s>(
    scope: &mut v8::HandleScope,
    obj: v8::Local<'_, v8::Object>,
) -> &'s mut Scope<'s> {
    let k_external = v8::String::new(scope, "_ptr").unwrap();

    (v8::Local::<v8::External>::try_from(obj.get(scope, k_external.into()).unwrap())
        .unwrap()
        .value() as *mut Scope)
        .as_mut()
        .unwrap()
}

pub fn set_variable<'s>(
    scope: &mut v8::HandleScope<'s>,
    args: v8::FunctionCallbackArguments<'s>,
    mut ret: v8::ReturnValue,
) {
    let stuart_scope = unsafe { get_stuart_context(scope, args.this()) };
    let key = args.get(0).to_rust_string_lossy(scope);
    let value = args.get(1);
    let json_value = super::json::js_to_json(value, scope);

    stuart_scope
        .stack
        .last_mut()
        .unwrap()
        .add_variable(key, json_value.unwrap());
}

// TODO: test
pub fn get_variable<'s>(
    scope: &mut v8::HandleScope<'s>,
    args: v8::FunctionCallbackArguments<'s>,
    mut ret: v8::ReturnValue,
) {
    let stuart_scope = unsafe { get_stuart_context(scope, args.this()) };
    let key = args.get(0).to_rust_string_lossy(scope);
    let value = stuart_scope.get_variable(&key);
    let v8_value = super::json::json_to_js(value, scope);

    ret.set(v8_value);
}
