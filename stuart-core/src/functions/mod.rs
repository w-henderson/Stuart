//! Provides the built-in functions and traits to create custom ones.

/// Contains all the built-in function parsers.
#[allow(clippy::missing_docs_in_private_items)]
pub mod parsers {
    mod begin;
    mod dateformat;
    mod end;
    mod excerpt;
    mod r#for;
    mod ifdefined;
    mod import;
    mod insert;
    mod timetoread;

    pub use begin::BeginParser as Begin;
    pub use dateformat::DateFormatParser as DateFormat;
    pub use end::EndParser as End;
    pub use excerpt::ExcerptParser as Excerpt;
    pub use ifdefined::IfDefinedParser as IfDefined;
    pub use import::ImportParser as Import;
    pub use insert::InsertParser as Insert;
    pub use r#for::ForParser as For;
    pub use timetoread::TimeToReadParser as TimeToRead;
}

use crate::parse::{ParseError, RawFunction};
use crate::process::error::ProcessError;
use crate::process::Scope;
use crate::TracebackError;

use std::fmt::Debug;

/// Represents a function parser.
///
/// A function parser is an object which is capable of parsing a raw function into an executable function object,
///   the inner workings of which are hidden from Stuart through the [`Function`] trait. The parser should also
///   define a name, which is used to identify the function when parsing a file. The name of the function parser
///   **must** be the same as that of the returned function.
pub trait FunctionParser: Send + Sync {
    /// Returns the name of the function which the parser can parse.
    ///
    /// This **must** return the same value as the `name` method of the returned function.
    fn name(&self) -> &'static str;

    /// Attempts to parse the raw function into an executable function object.
    fn parse(&self, raw: RawFunction) -> Result<Box<dyn Function>, ParseError>;

    /// Returns `true` if the raw function can be parsed by this function parser.
    fn can_parse(&self, raw: &RawFunction) -> bool {
        raw.name == self.name()
    }
}

/// Represents an executable function.
///
/// When the function is executed, it is given a [`Scope`] object, which contains information about the current state
///   of the program, including variables, stack frames and more.
pub trait Function: Debug + Send + Sync {
    /// Returns the name of the function.
    fn name(&self) -> &'static str;

    /// Executes the function in the given scope.
    fn execute(&self, scope: &mut Scope) -> Result<(), TracebackError<ProcessError>>;
}

/// A macro which counts its arguments.
macro_rules! count {
    () => { 0_usize };
    ($head:tt $($tail:tt)*) => { 1_usize + count!($($tail)*) };
}

/// Defines the functions available in the program by way of a global variable.
macro_rules! define_functions {
    ($($name:expr,)*) => {
        const FUNCTION_COUNT: usize = count!($($name)*);

        ::lazy_static::lazy_static! {
            static ref FUNCTION_PARSERS: [Box<dyn $crate::functions::FunctionParser>; FUNCTION_COUNT] = [
                $(Box::new($name)),*
            ];
        }
    }
}

/// Quietly asserts that the given condition is true.
///
/// If the condition is false, this macro will not panic, and will instead return an error.
#[macro_export]
macro_rules! quiet_assert {
    ($cond:expr) => {
        match $cond {
            true => Ok(()),
            false => Err(ParseError::AssertionError(stringify!($cond).to_string())),
        }
    };
}

/// Returns true if the string is an identifier of a function.
#[inline]
pub fn is_ident(s: &str) -> bool {
    s == "begin"
        || s == "dateformat"
        || s == "end"
        || s == "excerpt"
        || s == "for"
        || s == "ifdefined"
        || s == "import"
        || s == "insert"
        || s == "timetoread"
}
