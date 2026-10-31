//! Tool-argument validation against a tool's JSON Schema, ported from the
//! TypeScript `validateJsonSchemaValue` (`@deepseek-ai/dsh-tools`).
//!
//! It reports every violation in schema-walk order, as `"path" must be ...`
//! texts, so the model reads the same diagnostics from either host. Supported
//! keywords are the ones tool schemas use: `type` (`object`, `array`,
//! `string`, `number`, `integer`, `boolean`, `null`), `oneOf`, `properties`,
//! `required`, `additionalProperties: false`, `items`, `enum`, `const`.
//! A node with no `type` and no `oneOf` accepts any value.

use serde_json::Value;

/// The root path in diagnostics.
const ROOT: &str = "arguments";

/// Every violation of `schema` by `args`; empty when `args` is valid.
pub fn violations(schema: &Value, args: &Value) -> Vec<String> {
    check(schema, args, "")
}

/// `INVALID_ARGS` message text for a non-empty violation list.
pub fn invalid_arguments_message(violations: &[String]) -> String {
    format!("invalid arguments: {}", violations.join("; "))
}

fn shown(path: &str) -> &str {
    if path.is_empty() {
        ROOT
    } else {
        path
    }
}

fn property_path(path: &str, key: &str) -> String {
    if path.is_empty() {
        key.to_string()
    } else {
        format!("{path}.{key}")
    }
}

/// `JSON.stringify` of a value.
fn js(value: &Value) -> String {
    value.to_string()
}

fn scalar(node: &Value, value: &Value, path: &str) -> Vec<String> {
    if let Some(allowed) = node.get("enum").and_then(Value::as_array) {
        if !allowed.contains(value) {
            return vec![format!("\"{}\" must be one of {}", shown(path), js(&Value::Array(allowed.clone())))];
        }
    }
    if let Some(constant) = node.get("const") {
        if value != constant {
            return vec![format!("\"{}\" must be {}", shown(path), js(constant))];
        }
    }
    Vec::new()
}

/// A whole JSON number: finite, and an integer when `integer` is set.
fn is_integer(value: &Value) -> bool {
    value.as_f64().is_some_and(|n| n.is_finite() && n.fract() == 0.0)
}

fn check(node: &Value, value: &Value, path: &str) -> Vec<String> {
    if let Some(branches) = node.get("oneOf").and_then(Value::as_array) {
        let matches = branches.iter().filter(|branch| check(branch, value, path).is_empty()).count();
        return if matches == 1 {
            Vec::new()
        } else {
            vec![format!("\"{}\" must match exactly one oneOf branch (matched {matches})", shown(path))]
        };
    }
    let Some(kind) = node.get("type").and_then(Value::as_str) else {
        return Vec::new();
    };
    match kind {
        "object" => {
            let Some(object) = value.as_object() else {
                return vec![format!("\"{}\" must be an object", shown(path))];
            };
            let empty = serde_json::Map::new();
            let properties = node.get("properties").and_then(Value::as_object).unwrap_or(&empty);
            let mut out = Vec::new();
            for key in node.get("required").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str) {
                // JSON has no `undefined`, so only an absent key is missing.
                if !object.contains_key(key) {
                    out.push(format!("missing required property \"{}\"", property_path(path, key)));
                }
            }
            for (key, child) in properties {
                if let Some(present) = object.get(key) {
                    out.extend(check(child, present, &property_path(path, key)));
                }
            }
            if node.get("additionalProperties") == Some(&Value::Bool(false)) {
                for key in object.keys().filter(|key| !properties.contains_key(*key)) {
                    out.push(format!(
                        "\"{}\" is not a declared property (additionalProperties: false)",
                        property_path(path, key)
                    ));
                }
            }
            out
        }
        "array" => {
            let Some(items) = value.as_array() else {
                return vec![format!("\"{}\" must be an array", shown(path))];
            };
            match node.get("items") {
                Some(item_schema) => items
                    .iter()
                    .enumerate()
                    .flat_map(|(index, item)| check(item_schema, item, &format!("{path}[{index}]")))
                    .collect(),
                None => Vec::new(),
            }
        }
        "string" => {
            if value.is_string() {
                scalar(node, value, path)
            } else {
                vec![format!("\"{}\" must be a string", shown(path))]
            }
        }
        "number" => match value {
            Value::Number(_) if value.as_f64().is_some_and(f64::is_finite) => scalar(node, value, path),
            Value::Number(_) => vec![format!("\"{}\" must be a finite JSON number", shown(path))],
            _ => vec![format!("\"{}\" must be a number", shown(path))],
        },
        "integer" => {
            if is_integer(value) {
                scalar(node, value, path)
            } else {
                vec![format!("\"{}\" must be an integer", shown(path))]
            }
        }
        "boolean" => {
            if value.is_boolean() {
                scalar(node, value, path)
            } else {
                vec![format!("\"{}\" must be a boolean", shown(path))]
            }
        }
        "null" => {
            if value.is_null() {
                scalar(node, value, path)
            } else {
                vec![format!("\"{}\" must be null", shown(path))]
            }
        }
        _ => Vec::new(),
    }
}
