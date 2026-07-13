//! YAML front matter detection and parsing.

use crate::model::{FrontMatterError, YamlFrontMatter};

/// Splits a leading `--- ... ---` (or `...`) block from the document body.
/// Returns the raw front-matter text and the byte offset where the body starts.
pub(crate) fn split_front_matter(text: &str) -> Option<(&str, usize)> {
    let first_body_index = if text.starts_with("---\r\n") {
        5
    } else if text.starts_with("---\n") {
        4
    } else {
        return None;
    };

    let mut line_start = first_body_index;
    while line_start <= text.len() {
        let relative_line_end = text[line_start..].find('\n');
        let (line_end, next_line_start) = match relative_line_end {
            Some(relative) => {
                let line_end = line_start + relative;
                (line_end, line_end + 1)
            }
            None => (text.len(), text.len() + 1),
        };
        let line = text[line_start..line_end].trim_end_matches('\r');
        if matches!(line, "---" | "...") {
            let body_start = next_line_start.min(text.len());
            return Some((&text[first_body_index..line_start], body_start));
        }
        if relative_line_end.is_none() {
            break;
        }
        line_start = next_line_start;
    }

    None
}

pub(crate) fn parse_front_matter(raw: &str) -> Result<YamlFrontMatter, FrontMatterError> {
    let value = serde_yaml::from_str::<serde_yaml::Value>(raw).map_err(|err| FrontMatterError {
        message: err.to_string(),
    })?;
    let values = match value {
        serde_yaml::Value::Null => serde_yaml::Mapping::new(),
        serde_yaml::Value::Mapping(values) => values,
        _ => {
            return Err(FrontMatterError {
                message: "front matter must be a YAML mapping".to_string(),
            });
        }
    };

    Ok(YamlFrontMatter {
        raw: raw.to_string(),
        title: yaml_string_value(&values, "title"),
        author: yaml_string_value(&values, "author"),
        date: yaml_string_value(&values, "date"),
        values,
    })
}

fn yaml_string_value(values: &serde_yaml::Mapping, key: &str) -> Option<String> {
    values
        .get(serde_yaml::Value::String(key.to_string()))
        .and_then(|value| match value {
            serde_yaml::Value::String(value) => Some(value.clone()),
            serde_yaml::Value::Number(value) => Some(value.to_string()),
            serde_yaml::Value::Bool(value) => Some(value.to_string()),
            _ => None,
        })
}
