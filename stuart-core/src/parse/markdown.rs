use super::{ParseError, TracebackError};

use humphrey_json::Value;
use pulldown_cmark::{html, Options, Parser};

use std::path::Path;

#[derive(Clone, Debug)]
pub struct ParsedMarkdown {
    frontmatter: Vec<(String, String)>,
    body: String,
}

pub fn parse_markdown(
    input: String,
    path: &Path,
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

            if dashed_lines == 1 {
                let mut parts = line.split(':');
                let key = parts
                    .next()
                    .ok_or(TracebackError {
                        path: path.to_path_buf(),
                        line: i as u32 + 1,
                        column: 0,
                        kind: ParseError::InvalidFrontmatter,
                    })?
                    .trim()
                    .to_string();

                let value = parts
                    .next()
                    .ok_or(TracebackError {
                        path: path.to_path_buf(),
                        line: i as u32 + 1,
                        column: 0,
                        kind: ParseError::InvalidFrontmatter,
                    })?
                    .trim()
                    .strip_prefix('"')
                    .and_then(|v| v.strip_suffix('"'))
                    .ok_or(TracebackError {
                        path: path.to_path_buf(),
                        kind: ParseError::InvalidFrontmatter,
                        line: i as u32 + 1,
                        column: 0,
                    })?
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

    let markdown = input
        .lines()
        .skip(lines_to_skip as usize)
        .collect::<Vec<_>>()
        .join("\n");

    let parser = Parser::new_ext(&markdown, Options::all());
    let mut body = String::new();
    html::push_html(&mut body, parser);

    Ok(ParsedMarkdown { frontmatter, body })
}

impl ParsedMarkdown {
    pub fn to_value(&self) -> Value {
        let mut v = self.to_json();
        v["content"] = Value::String(self.body.clone());
        v
    }

    pub fn to_json(&self) -> Value {
        let children = self
            .frontmatter
            .iter()
            .map(|(key, value)| (key.clone(), Value::String(value.clone())))
            .collect::<Vec<_>>();

        Value::Object(children)
    }
}
