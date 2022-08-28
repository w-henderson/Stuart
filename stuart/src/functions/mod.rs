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

pub trait FunctionParser: Send + Sync {
    fn name(&self) -> &'static str;
    fn parse(&self, raw: RawFunction) -> Result<Box<dyn Function>, ParseError>;

    fn can_parse(&self, raw: &RawFunction) -> bool {
        raw.name == self.name()
    }
}

pub trait Function: Debug + Send + Sync {
    fn name(&self) -> &'static str;
    fn execute(&self, scope: &mut Scope) -> Result<(), TracebackError<ProcessError>>;
}

macro_rules! count {
    () => { 0_usize };
    ($head:tt $($tail:tt)*) => { 1_usize + count!($($tail)*) };
}

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

#[macro_export]
macro_rules! quiet_assert {
    ($cond:expr) => {
        match $cond {
            true => Ok(()),
            false => Err(ParseError::AssertionError(stringify!($cond).to_string())),
        }
    };
}

#[inline]
pub fn is_ident(s: &str) -> bool {
    s == "begin"
        || s == "end"
        || s == "for"
        || s == "ifdefined"
        || s == "dateformat"
        || s == "timetoread"
        || s == "excerpt"
}
