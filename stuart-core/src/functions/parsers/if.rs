macro_rules! if_parsers {
    ($($name:ident, $ty:ident, $cond:tt;)*) => {
        $(
            mod $name {
                #[doc = concat!("Parses the `", stringify!($name), "` function.")]
                pub struct Parser;

                #[derive(Debug, Clone)]
                pub struct Function {
                    input_1: $crate::functions::Input,
                    input_2: $crate::functions::Input,
                }

                impl $crate::functions::FunctionParser for Parser {
                    fn name(&self) -> &'static str {
                        stringify!($name)
                    }

                    fn parse(&self, mut raw: $crate::parse::RawFunction) -> Result<Box<dyn $crate::functions::Function>, $crate::parse::ParseError> {
                        $crate::quiet_assert!(raw.positional_args.len() == 2)?;
                        $crate::quiet_assert!(raw.named_args.is_empty())?;

                        let input_2 = match raw.positional_args.pop().unwrap() {
                            $crate::parse::RawArgument::Variable(v) => $crate::functions::Input::Variable(v),
                            $crate::parse::RawArgument::String(s) => $crate::functions::Input::String(s),
                            $crate::parse::RawArgument::Integer(i) => $crate::functions::Input::Integer(i),
                            _ => return Err($crate::parse::ParseError::InvalidArgument),
                        };

                        let input_1 = match raw.positional_args.pop().unwrap() {
                            $crate::parse::RawArgument::Variable(v) => $crate::functions::Input::Variable(v),
                            $crate::parse::RawArgument::String(s) => $crate::functions::Input::String(s),
                            $crate::parse::RawArgument::Integer(i) => $crate::functions::Input::Integer(i),
                            _ => return Err($crate::parse::ParseError::InvalidArgument),
                        };

                        Ok(Box::new(Function { input_1, input_2 }))
                    }
                }

                impl $crate::functions::Function for Function {
                    fn name(&self) -> &'static str {
                        stringify!($name)
                    }

                    fn execute(&self, scope: &mut $crate::process::Scope) -> Result<(), $crate::TracebackError<$crate::process::ProcessError>> {
                        let self_token = scope.tokens.current().unwrap().clone();

                        let input_1 = self.input_1.evaluate_variable(scope).ok_or_else(|| {
                            self_token.traceback($crate::process::ProcessError::UndefinedVariable(self.input_1.to_string()))
                        })?;

                        let input_2 = self.input_2.evaluate_variable(scope).ok_or_else(|| {
                            self_token.traceback($crate::process::ProcessError::UndefinedVariable(self.input_2.to_string()))
                        })?;

                        let condition = input_1 $cond input_2;

                        let frame = $crate::process::stack::StackFrame::new(format!(
                            "{}:{}:{}",
                            stringify!($name),
                            self.input_1.to_string(),
                            self.input_2.to_string()
                        ));

                        let stack_height = scope.stack.len();
                        scope.stack.push(frame);

                        while scope.stack.len() > stack_height {
                            let token = scope
                                .tokens
                                .next()
                                .ok_or_else(|| self_token.traceback($crate::process::ProcessError::UnexpectedEndOfFile))?;

                            if condition
                                || (token
                                    .as_function()
                                    .map(|f| f.name() == "end")
                                    .unwrap_or(false)
                                    && scope.stack.len() == stack_height + 1)
                            {
                                token.process(scope)?;
                            }
                        }

                        Ok(())
                    }
                }
            }

            pub use $name::Parser as $ty;
        )*
    }
}
