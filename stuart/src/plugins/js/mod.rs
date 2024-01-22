mod json;

use stuart_core::functions::{Function, FunctionParser};
use stuart_core::parse::{ParseError, RawArgument, RawFunction};
use stuart_core::plugins::Plugin;
use stuart_core::process::{ProcessError, Scope};
use stuart_core::TracebackError;

use std::path::Path;
use std::rc::Rc;
use std::sync::{Mutex, Once};

static INITIALISED: Once = Once::new();

pub struct JSFunctionParser {
    name: String,
    isolate: Rc<Mutex<v8::OwnedIsolate>>,
    context: v8::Global<v8::Context>,
}

#[derive(Debug)]
pub struct JSFunction {
    name: String,
    isolate: Rc<Mutex<v8::OwnedIsolate>>,
    context: v8::Global<v8::Context>,
    args: Vec<RawArgument>,
}

/// Attempts to load a JavaScript plugin from the given path, spinning up a new V8 isolate.
pub fn load_js_plugin(path: impl AsRef<Path>) -> Result<Plugin, String> {
    INITIALISED.call_once(|| {
        v8::V8::initialize_platform(v8::new_default_platform(0, false).make_shared());
        v8::V8::initialize();
    });

    let mut isolate = v8::Isolate::new(Default::default());
    let global_context;

    let (name, version, functions) = {
        let handle_scope = &mut v8::HandleScope::new(&mut isolate);
        let context = v8::Context::new(handle_scope);
        global_context = v8::Global::new(handle_scope, context);
        let scope = &mut v8::ContextScope::new(handle_scope, context);

        let name: v8::Local<'_, v8::Value> =
            v8::String::new(scope, &path.as_ref().to_string_lossy())
                .unwrap()
                .into();
        let origin = v8::ScriptOrigin::new(scope, name, 0, 0, false, 0, name, false, false, true);
        let source_string = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let source = v8::String::new(scope, &source_string).unwrap();
        let compile_source = v8::script_compiler::Source::new(source, Some(&origin));
        let module = v8::script_compiler::compile_module(scope, compile_source).unwrap();

        module
            .instantiate_module(scope, |_, _, _, m| Some(m))
            .unwrap();
        module.evaluate(scope).unwrap();

        let key = v8::String::new(scope, "default").unwrap();
        let default = module
            .get_module_namespace()
            .to_object(scope)
            .unwrap()
            .get(scope, key.into())
            .unwrap()
            .to_object(scope)
            .unwrap();

        // context.global(scope).set(scope, key.into(), default.into());

        let key = v8::String::new(scope, "name").unwrap();
        let plugin_name = default
            .get(scope, key.into())
            .unwrap()
            .to_rust_string_lossy(scope);

        let key = v8::String::new(scope, "version").unwrap();
        let plugin_version = default
            .get(scope, key.into())
            .unwrap()
            .to_rust_string_lossy(scope);

        let key = v8::String::new(scope, "functions").unwrap();
        let functions = default
            .get(scope, key.into())
            .unwrap()
            .to_object(scope)
            .unwrap();
        let key = v8::String::new(scope, "length").unwrap();
        let length = functions
            .get(scope, key.into())
            .unwrap()
            .uint32_value(scope)
            .unwrap();

        let mut functions_vec = Vec::with_capacity(length as usize);

        for i in 0..length {
            let function_object = functions
                .get_index(scope, i)
                .unwrap()
                .to_object(scope)
                .unwrap();

            let key = v8::String::new(scope, "name").unwrap();
            let function_name = function_object
                .get(scope, key.into())
                .unwrap()
                .to_rust_string_lossy(scope);

            let key = v8::String::new(scope, "fn").unwrap();
            let function_fn = function_object.get(scope, key.into()).unwrap();

            println!("{}", function_name);
            let key = v8::String::new(scope, &format!("_stuart_{}", function_name)).unwrap();
            context.global(scope).set(scope, key.into(), function_fn);

            functions_vec.push(function_name);
        }

        (plugin_name, plugin_version, functions_vec)
    };

    let isolate = Rc::new(Mutex::new(isolate));
    let mut function_parsers = Vec::with_capacity(functions.len());
    for function in &functions {
        function_parsers.push(Box::new(JSFunctionParser {
            name: function.clone(),
            isolate: isolate.clone(),
            context: global_context.clone(),
        }) as Box<dyn FunctionParser>);
    }

    Ok(Plugin {
        name,
        version,
        functions: function_parsers,
        parsers: Vec::new(),
    })
}

impl FunctionParser for JSFunctionParser {
    fn name(&self) -> &str {
        &self.name
    }

    fn parse(&self, raw: RawFunction) -> Result<Box<dyn Function>, ParseError> {
        Ok(Box::new(JSFunction {
            name: self.name.clone(),
            isolate: self.isolate.clone(),
            context: self.context.clone(),
            args: raw.positional_args,
        }))
    }
}

impl Function for JSFunction {
    fn name(&self) -> &str {
        todo!()
    }

    fn execute(&self, stuart_scope: &mut Scope) -> Result<(), TracebackError<ProcessError>> {
        let self_token = stuart_scope.tokens.current().unwrap().clone();

        let mut isolate = self.isolate.lock().unwrap();
        let handle_scope = &mut v8::HandleScope::new(&mut *isolate);
        let context = v8::Local::new(handle_scope, &self.context);
        let scope = &mut v8::ContextScope::new(handle_scope, context);

        let evaluated_args = self
            .args
            .iter()
            .map(|a| match a {
                RawArgument::Variable(name) => match stuart_scope.get_variable(name) {
                    Some(v) => Ok(json::json_to_js(v, scope)),
                    None => {
                        Err(self_token.traceback(ProcessError::UndefinedVariable(name.to_string())))
                    }
                },
                RawArgument::String(s) => Ok(v8::String::new(scope, s).unwrap().into()),
                RawArgument::Integer(i) => Ok(v8::Integer::new(scope, *i).into()),
                _ => Err(self_token.traceback(ProcessError::StackError)),
            })
            .collect::<Result<Vec<_>, _>>()?;

        let key = v8::String::new(scope, &format!("_stuart_{}", self.name)).unwrap();
        println!("{}", self.name);
        let function_obj = context.global(scope).get(scope, key.into()).unwrap();
        let function = v8::Local::<v8::Function>::try_from(function_obj).unwrap();

        let result = function.call(scope, function_obj, &evaluated_args).unwrap();

        stuart_scope
            .output(result.to_rust_string_lossy(scope))
            .unwrap();

        Ok(())
    }
}
