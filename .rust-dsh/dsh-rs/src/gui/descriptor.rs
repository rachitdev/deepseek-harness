//! The TypeScript gateway's argument check, driven by the endpoint inventory.
//!
//! `docs/gui-endpoints.json` (written by `tools/gui-endpoints.mjs` from the
//! official packages' remote-client descriptors) names each endpoint's
//! parameters. The gateway refuses a call whose `args` object does not carry
//! exactly those fields before any handler runs. A `lookup` parameter travels
//! under its name plus `Id` (`agent` is `agentId`); a `json` parameter keeps its
//! name.
//!
//! A parameter the descriptor marks `optional` (`acceptsUndefined`) may be
//! absent; every other parameter must be present. Both an unexpected and a
//! missing field fail as `gateway/arguments-invalid`, with one message naming
//! both, the way the TypeScript gateway words it.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde_json::{json, Value};

use super::router::boundary;
use super::wire::{codes, Failure};

const INVENTORY: &str = include_str!("../../docs/gui-endpoints.json");

/// One endpoint's wire fields.
struct Fields {
    /// Every field the endpoint may carry.
    all: Vec<String>,
    /// The fields it must carry.
    required: Vec<String>,
    /// `(parameter name, wire field, JSON Schema)` of each `json` parameter, in
    /// declaration order. Lookups are resolved by the handler, not typed here.
    typed: Vec<(String, String, Value)>,
}

/// Wire field names per endpoint, parsed once.
fn fields() -> &'static HashMap<String, Fields> {
    static FIELDS: OnceLock<HashMap<String, Fields>> = OnceLock::new();
    FIELDS.get_or_init(|| {
        let doc: Value = serde_json::from_str(INVENTORY).expect("docs/gui-endpoints.json is JSON");
        let endpoints = doc["endpoints"].as_array().expect("the inventory lists endpoints");
        endpoints
            .iter()
            .map(|endpoint| {
                let name = endpoint["endpoint"].as_str().expect("endpoint name").to_string();
                let mut fields = Fields { all: Vec::new(), required: Vec::new(), typed: Vec::new() };
                for p in endpoint["parameters"].as_array().expect("endpoint parameters") {
                    let base = p["name"].as_str().expect("parameter name");
                    let wire = if p["source"] == "lookup" { format!("{base}Id") } else { base.to_string() };
                    if p["optional"] != true {
                        fields.required.push(wire.clone());
                    }
                    if p["source"] != "lookup" && !p["schema"].is_null() {
                        fields.typed.push((base.to_string(), wire.clone(), p["schema"].clone()));
                    }
                    fields.all.push(wire);
                }
                (name, fields)
            })
            .collect()
    })
}

/// `value` reduced to what `schema` describes: object properties the schema does
/// not name are dropped, at every depth, as the TypeScript gateway's decoder does
/// before a handler sees the value. Where a schema has alternatives (`oneOf`),
/// the first the value satisfies decides; a value that fits none is returned as is.
pub fn strip_unknown(value: &Value, schema: &Value) -> Value {
    let Some(schema_object) = schema.as_object() else {
        return value.clone();
    };
    if let Some(variants) = schema_object.get("oneOf").and_then(Value::as_array) {
        return match variants.iter().find(|variant| matches_schema(value, variant)) {
            Some(variant) => strip_unknown(value, variant),
            None => value.clone(),
        };
    }
    match value {
        Value::Array(items) => match schema_object.get("items") {
            Some(item_schema) => Value::Array(items.iter().map(|item| strip_unknown(item, item_schema)).collect()),
            None => value.clone(),
        },
        Value::Object(object) => {
            let Some(properties) = schema_object.get("properties").and_then(Value::as_object) else {
                // A schema that names no properties describes an open object.
                return value.clone();
            };
            // `additionalProperties: true` is how a descriptor says the object is open:
            // the named properties are still shaped, and every other one is kept as sent.
            let open = schema_object.get("additionalProperties") == Some(&Value::Bool(true)) && properties.is_empty();
            if open {
                return value.clone();
            }
            let kept = object
                .iter()
                .filter_map(|(key, inner)| properties.get(key).map(|schema| (key.clone(), strip_unknown(inner, schema))))
                .collect();
            Value::Object(kept)
        }
        _ => value.clone(),
    }
}

/// `args` with each typed parameter reduced to its schema, and the same fields otherwise.
pub(super) fn strip_args(endpoint: &str, args: &Value) -> Value {
    let (Some(fields), Some(object)) = (fields().get(endpoint), args.as_object()) else {
        return args.clone();
    };
    let mut out = object.clone();
    for (_, wire, schema) in &fields.typed {
        if let Some(value) = object.get(wire) {
            out.insert(wire.clone(), strip_unknown(value, schema));
        }
    }
    Value::Object(out)
}

/// A call's `args` that satisfies the inventory for `endpoint`: every required
/// field, each with a minimal value its schema accepts. Used by tests to reach
/// a handler without tripping the gateway's checks.
pub fn sample_args(endpoint: &str) -> Option<Value> {
    let fields = fields().get(endpoint)?;
    let mut args = serde_json::Map::new();
    for field in &fields.required {
        let value = fields
            .typed
            .iter()
            .find(|(_, wire, _)| wire == field)
            .map_or_else(|| json!("x"), |(_, _, schema)| sample_value(schema));
        args.insert(field.clone(), value);
    }
    Some(Value::Object(args))
}

/// The smallest JSON value `schema` accepts, over the keywords `matches_schema` reads.
pub fn sample_value(schema: &Value) -> Value {
    let Some(schema) = schema.as_object() else {
        return json!("x");
    };
    if let Some(first) = schema.get("oneOf").and_then(Value::as_array).and_then(|v| v.first()) {
        return sample_value(first);
    }
    if let Some(constant) = schema.get("const") {
        return constant.clone();
    }
    if let Some(first) = schema.get("enum").and_then(Value::as_array).and_then(|v| v.first()) {
        return first.clone();
    }
    match schema.get("type").and_then(Value::as_str) {
        Some("number") => json!(0),
        Some("boolean") => json!(false),
        Some("null") => Value::Null,
        Some("array") => json!([]),
        Some("object") => {
            let mut object = serde_json::Map::new();
            let properties = schema.get("properties").and_then(Value::as_object);
            for key in schema.get("required").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str) {
                let value = properties.and_then(|p| p.get(key)).map_or_else(|| json!("x"), sample_value);
                object.insert(key.to_string(), value);
            }
            Value::Object(object)
        }
        _ => json!("x"),
    }
}

/// Refuse `args` that carry a field the inventory does not list for `endpoint`
/// or lack one it requires. An endpoint the inventory does not list passes: the
/// router answers it.
pub(super) fn check(endpoint: &str, args: &Value) -> Result<(), Failure> {
    let Some(fields) = fields().get(endpoint) else {
        return Ok(());
    };
    let Some(object) = args.as_object() else {
        return Ok(());
    };
    let quote = |names: Vec<&String>| names.iter().map(|n| format!("{n:?}")).collect::<Vec<_>>().join(", ");
    let missing: Vec<&String> = fields.required.iter().filter(|n| !object.contains_key(*n)).collect();
    let unexpected: Vec<&String> = object.keys().filter(|k| !fields.all.contains(k)).collect();
    if missing.is_empty() && unexpected.is_empty() {
        // Types are checked only for a call whose fields are right, in parameter order.
        for (name, wire, schema) in &fields.typed {
            if let Some(value) = object.get(wire) {
                if !matches_schema(value, schema) {
                    return Err(boundary(endpoint, name));
                }
            }
        }
        return Ok(());
    }
    let mut parts = Vec::new();
    if !missing.is_empty() {
        parts.push(format!("missing {}", quote(missing)));
    }
    if !unexpected.is_empty() {
        parts.push(format!("unexpected {}", quote(unexpected)));
    }
    Err(Failure {
        code: codes::ARGUMENTS_INVALID.to_string(),
        message: format!(
            "typert gateway: {endpoint}: args fields do not match the descriptor: {}",
            parts.join("; ")
        ),
        details: json!({ "endpoint": endpoint }),
    })
}

/// Whether `value` satisfies `schema`, over the JSON Schema keywords the
/// descriptors use: `type`, `const`, `enum`, `oneOf`, `properties`, `required`
/// and `items`. Objects accept unknown properties, as the descriptors declare.
/// A schema with none of those keywords accepts anything.
pub(super) fn matches_schema(value: &Value, schema: &Value) -> bool {
    let Some(schema) = schema.as_object() else {
        return true;
    };
    if let Some(variants) = schema.get("oneOf").and_then(Value::as_array) {
        return variants.iter().any(|variant| matches_schema(value, variant));
    }
    if let Some(expected) = schema.get("const") {
        return value == expected;
    }
    if let Some(choices) = schema.get("enum").and_then(Value::as_array) {
        return choices.contains(value);
    }
    let type_ok = match schema.get("type").and_then(Value::as_str) {
        None => true,
        Some("string") => value.is_string(),
        Some("number") => value.is_number(),
        Some("boolean") => value.is_boolean(),
        Some("null") => value.is_null(),
        Some("array") => value.is_array(),
        Some("object") => value.is_object(),
        Some(_) => true,
    };
    if !type_ok {
        return false;
    }
    if let (Some(items), Some(schema)) = (value.as_array(), schema.get("items")) {
        if !items.iter().all(|item| matches_schema(item, schema)) {
            return false;
        }
    }
    if let Some(object) = value.as_object() {
        if let Some(required) = schema.get("required").and_then(Value::as_array) {
            if required.iter().filter_map(Value::as_str).any(|key| !object.contains_key(key)) {
                return false;
            }
        }
        if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
            for (key, property) in properties {
                if object.get(key).is_some_and(|present| !matches_schema(present, property)) {
                    return false;
                }
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_properties_are_dropped_at_every_depth() {
        let schema = json!({
            "type": "object", "required": ["a"],
            "properties": { "a": { "type": "object", "properties": { "b": { "type": "string" } } }, "list": { "type": "array", "items": { "type": "object", "properties": { "k": {} } } } },
        });
        let value = json!({ "a": { "b": "x", "z": 1 }, "extra": true, "list": [{ "k": 1, "drop": 2 }] });
        assert_eq!(strip_unknown(&value, &schema), json!({ "a": { "b": "x" }, "list": [{ "k": 1 }] }));
    }

    #[test]
    fn the_first_matching_alternative_decides_what_is_kept() {
        let schema = json!({ "oneOf": [
            { "type": "object", "required": ["kind"], "properties": { "kind": { "const": "at" }, "at": {} } },
            { "type": "object", "required": ["kind"], "properties": { "kind": { "const": "every" }, "every_seconds": {} } },
        ] });
        assert_eq!(strip_unknown(&json!({ "kind": "every", "every_seconds": 60, "x": 1, "at": 5 }), &schema), json!({ "kind": "every", "every_seconds": 60 }));
        assert_eq!(strip_unknown(&json!({ "nothing": 1 }), &schema), json!({ "nothing": 1 }), "no alternative fits: untouched");
    }

    #[test]
    fn an_object_declared_open_with_no_named_properties_keeps_everything() {
        let open = json!({ "type": "object", "properties": {}, "required": [], "additionalProperties": true });
        assert_eq!(strip_unknown(&json!({ "welcomeNoticeVersion": "a", "deep": { "x": 1 } }), &open), json!({ "welcomeNoticeVersion": "a", "deep": { "x": 1 } }));
    }

    #[test]
    fn an_open_object_schema_keeps_everything() {
        assert_eq!(strip_unknown(&json!({ "a": 1 }), &json!({ "type": "object" })), json!({ "a": 1 }));
        assert_eq!(strip_unknown(&json!({ "a": 1 }), &json!({})), json!({ "a": 1 }));
    }

    fn passes(value: Value, schema: Value) -> bool {
        matches_schema(&value, &schema)
    }

    #[test]
    fn scalar_types_are_exact() {
        assert!(passes(json!("a"), json!({ "type": "string" })));
        assert!(!passes(json!(1), json!({ "type": "string" })));
        assert!(passes(json!(1.5), json!({ "type": "number" })));
        assert!(!passes(json!("1"), json!({ "type": "number" })));
        assert!(passes(json!(false), json!({ "type": "boolean" })));
        assert!(!passes(json!(0), json!({ "type": "boolean" })));
        assert!(passes(Value::Null, json!({ "type": "null" })));
        assert!(!passes(json!({}), json!({ "type": "null" })));
    }

    #[test]
    fn a_json_null_is_not_an_object_or_an_array() {
        assert!(!passes(Value::Null, json!({ "type": "object" })));
        assert!(!passes(Value::Null, json!({ "type": "array" })));
        assert!(!passes(json!([]), json!({ "type": "object" })));
        assert!(!passes(json!({}), json!({ "type": "array" })));
    }

    #[test]
    fn const_and_enum_compare_whole_values() {
        assert!(passes(json!("set"), json!({ "const": "set" })));
        assert!(!passes(json!("unset"), json!({ "const": "set" })));
        assert!(passes(json!("queue"), json!({ "enum": ["queue", "steer"] })));
        assert!(!passes(json!("other"), json!({ "enum": ["queue", "steer"] })));
        assert!(!passes(json!(1), json!({ "enum": ["1"] })), "a number is not its string form");
    }

    #[test]
    fn one_of_accepts_any_variant() {
        let schema = json!({ "oneOf": [{ "type": "string" }, { "type": "number" }] });
        assert!(passes(json!("a"), schema.clone()));
        assert!(passes(json!(2), schema.clone()));
        assert!(!passes(json!(true), schema));
    }

    #[test]
    fn objects_need_their_required_keys_and_typed_properties_but_may_carry_more() {
        let schema = json!({
            "type": "object",
            "properties": { "id": { "type": "string" }, "n": { "type": "number" } },
            "required": ["id"],
        });
        assert!(passes(json!({ "id": "a" }), schema.clone()));
        assert!(passes(json!({ "id": "a", "n": 1, "extra": [] }), schema.clone()));
        assert!(!passes(json!({}), schema.clone()), "a required key is missing");
        assert!(!passes(json!({ "id": 5 }), schema.clone()), "a property has the wrong type");
        assert!(!passes(json!({ "id": "a", "n": "1" }), schema), "an optional property still has to fit when present");
    }

    #[test]
    fn array_items_are_each_checked() {
        let schema = json!({ "type": "array", "items": { "type": "string" } });
        assert!(passes(json!([]), schema.clone()));
        assert!(passes(json!(["a", "b"]), schema.clone()));
        assert!(!passes(json!(["a", 1]), schema));
    }

    #[test]
    fn a_schema_without_known_keywords_accepts_anything() {
        assert!(passes(json!(1), json!({})));
        assert!(passes(json!(null), json!({ "description": "free" })));
        assert!(passes(json!("x"), Value::Null));
    }

    #[test]
    fn sample_values_satisfy_their_own_schema() {
        for schema in [
            json!({ "type": "string" }),
            json!({ "type": "number" }),
            json!({ "const": "continuable" }),
            json!({ "enum": ["a", "b"] }),
            json!({ "oneOf": [{ "type": "null" }, { "type": "string" }] }),
            json!({ "type": "object", "required": ["a"], "properties": { "a": { "type": "array" } } }),
        ] {
            assert!(matches_schema(&sample_value(&schema), &schema), "{schema}");
        }
    }

    #[test]
    fn every_inventory_endpoint_accepts_its_own_sample_arguments() {
        let doc: Value = serde_json::from_str(INVENTORY).unwrap();
        for endpoint in doc["endpoints"].as_array().unwrap() {
            let name = endpoint["endpoint"].as_str().unwrap();
            let args = sample_args(name).expect("listed");
            assert!(check(name, &args).is_ok(), "{name}: {args}");
        }
    }

    #[test]
    fn missing_and_unexpected_fields_are_reported_together_before_any_type() {
        // `credentials/set` takes `ref` and `value`.
        let failure = check("credentials/set", &json!({ "ref": 5, "extra": 1 })).unwrap_err();
        assert_eq!(failure.code, codes::ARGUMENTS_INVALID);
        assert_eq!(
            failure.message,
            "typert gateway: credentials/set: args fields do not match the descriptor: missing \"value\"; unexpected \"extra\""
        );
        assert_eq!(failure.details, json!({ "endpoint": "credentials/set" }));
    }

    #[test]
    fn a_wrong_type_names_its_parameter_and_the_first_one_wins() {
        let failure = check("credentials/set", &json!({ "ref": 5, "value": 6 })).unwrap_err();
        assert_eq!(failure.code, codes::INPUT_INVALID);
        assert_eq!(failure.details, json!({ "endpoint": "credentials/set", "field": "ref" }));
    }

    #[test]
    fn a_lookup_parameter_travels_under_its_name_plus_id() {
        let failure = check("skills/list", &json!({ "request": { "sessionId": "s" }, "agent": "a" })).unwrap_err();
        assert!(failure.message.contains("unexpected \"agent\""), "{}", failure.message);
        assert!(check("terminal/list", &json!({ "sessionId": "s" })).is_ok());
    }

    #[test]
    fn an_optional_parameter_may_be_absent_but_not_misspelled() {
        assert!(check("directoryPicker/list", &json!({})).is_ok());
        assert!(check("directoryPicker/list", &json!({ "path": "/tmp" })).is_ok());
        assert!(check("directoryPicker/list", &json!({ "path": 5 })).is_err());
        assert!(check("directoryPicker/list", &json!({ "dir": "/tmp" })).is_err());
    }

    #[test]
    fn an_unlisted_endpoint_and_a_non_object_pass_through() {
        assert!(check("no/such", &json!({ "anything": 1 })).is_ok());
        assert!(check("credentials/set", &json!("not an object")).is_ok());
    }
}
