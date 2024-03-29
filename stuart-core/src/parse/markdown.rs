//! Provides functionality for parsing markdown files.

use crate::plugins::Manager;

use super::{parse_html, LocatableToken, ParseError, TracebackError};

use humphrey_json::Value;

use std::path::Path;

/// Represents the parsed contents of a markdown file.
#[derive(Clone, Debug)]
pub struct ParsedMarkdown {
    /// The frontmatter of the file.
    pub(crate) frontmatter: Vec<(String, String)>,
    /// The raw markdown body of the file.
    pub(crate) markdown: Vec<LocatableToken>,
    /// The raw markdown body of the file as a string.
    pub(crate) markdown_string: String,
    /// The final processed HTML body of the file.
    pub(crate) html: Option<String>,
}

/// Attempts to parse a markdown file into a [`ParsedMarkdown`] struct.
pub fn parse_markdown(
    input: String,
    path: &Path,
    plugins: Option<&dyn Manager>,
) -> Result<ParsedMarkdown, TracebackError<ParseError>> {
    let (lines_to_skip, frontmatter) = if input.starts_with("---\n") || input.starts_with("---\r\n")
    {
        let mut dashed_lines: u8 = 0;
        let mut lines_to_skip = 0;
        let mut frontmatter = Vec::new();

        for (i, line) in input.lines().enumerate() {
            if line.starts_with("---") {
                dashed_lines += 1;

                if dashed_lines == 2 {
                    lines_to_skip = i + 1;
                    break;
                }

                continue;
            }

            let e = || TracebackError {
                path: path.to_path_buf(),
                line: i as u32 + 1,
                column: 0,
                kind: ParseError::InvalidFrontmatter,
            };

            if dashed_lines == 1 {
                let mut parts = line.splitn(2, ':');
                let key = parts.next().ok_or_else(e)?.trim().to_string();

                let value = parts
                    .next()
                    .ok_or_else(e)?
                    .trim()
                    .strip_prefix('"')
                    .and_then(|v| v.strip_suffix('"'))
                    .ok_or_else(e)?
                    .to_string();

                frontmatter.push((key, value));
            }
        }

        if dashed_lines != 2 {
            return Err(TracebackError {
                path: path.to_path_buf(),
                kind: ParseError::UnexpectedEOF,
                line: input.lines().count() as u32,
                column: 0,
            });
        }

        (lines_to_skip, frontmatter)
    } else {
        (0, Vec::new())
    };

    let raw_markdown = input
        .lines()
        .skip(lines_to_skip)
        .collect::<Vec<_>>()
        .join("\n");

    let markdown = parse_html(&raw_markdown, path, plugins)?;

    Ok(ParsedMarkdown {
        frontmatter,
        markdown,
        markdown_string: raw_markdown,
        html: None,
    })
}

impl ParsedMarkdown {
    /// Converts the parsed markdown into a full JSON object for use by the Stuart program.
    ///
    /// **Warning:** this function also returns the body of the file as an HTML string. This can be very large, so if the contents
    ///   is not required, consider using [`ParsedMarkdown::to_json`], which does the same thing without returning the contents.
    pub fn to_value(&self) -> Value {
        let mut v = self.frontmatter_to_value();
        v["content"] = Value::String(self.html.as_ref().unwrap().clone());
        v["markdown"] = Value::String(self.markdown_string.clone());
        v
    }

    /// Converts the markdown frontmatter into a JSON object.
    pub fn frontmatter_to_value(&self) -> Value {
        let children = self
            .frontmatter
            .iter()
            .map(|(key, value)| (key.clone(), Value::String(value.clone())))
            .collect::<Vec<_>>();

        Value::Object(children)
    }
}
