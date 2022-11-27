//! Provides functionality for parsing the contents of files.

use crate::parse::{LocatableToken, ParsedMarkdown};
use crate::plugins::NodeProcessor;

use humphrey_json::prelude::*;
use humphrey_json::Value;

use std::fmt::Debug;
use std::rc::Rc;

/// The parsed contents of a file.
#[derive(Clone)]
pub enum ParsedContents {
    /// An HTML file, parsed into template tokens.
    Html(Vec<LocatableToken>),
    /// A markdown file, parsed into frontmatter and HTML.
    Markdown(ParsedMarkdown),
    /// A JSON file.
    Json(Value),
    /// A file that was parsed by a plugin.
    Custom(Rc<Box<dyn NodeProcessor>>),
    /// The file was not parsed because no parser was available.
    None,
    /// The file was not parsed because it was ignored.
    Ignored,
}

impl ParsedContents {
    /// Returns the template tokens of the parsed contents, if applicable.
    pub fn tokens(&self) -> Option<&[LocatableToken]> {
        match self {
            Self::Html(tokens) => Some(tokens),
            _ => None,
        }
    }

    /// Returns the parsed markdown data, if applicable.
    pub fn markdown(&self) -> Option<&ParsedMarkdown> {
        match self {
            Self::Markdown(markdown) => Some(markdown),
            _ => None,
        }
    }

    /// Returns `true` if the contents were ignored.
    pub fn is_ignored(&self) -> bool {
        matches!(self, Self::Ignored)
    }

    /// Converts the parsed contents to a JSON value, if applicable.
    pub fn to_json(&self) -> Option<Value> {
        match self {
            ParsedContents::Html(_) => None,
            ParsedContents::None => None,
            ParsedContents::Ignored => None,

            ParsedContents::Markdown(md) => Some(json!({
                "type": "markdown",
                "value": (md.frontmatter_to_value())
            })),

            ParsedContents::Json(v) => Some(json!({
                "type": "json",
                "value": (v.clone())
            })),

            ParsedContents::Custom(c) => Some(c.to_json()),
        }
    }
}

impl Debug for ParsedContents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Html(arg0) => f.debug_tuple("Html").field(arg0).finish(),
            Self::Markdown(arg0) => f.debug_tuple("Markdown").field(arg0).finish(),
            Self::Json(arg0) => f.debug_tuple("Json").field(arg0).finish(),
            Self::Custom(_) => f.debug_tuple("Custom").finish(),
            Self::None => write!(f, "None"),
            Self::Ignored => write!(f, "Ignored"),
        }
    }
}
