//! Tool-argument validation against the TypeScript validator's own answers
//! (`tests/fixtures/tool_args_ts.json`): every violation text, in order, for
//! the schedule tools' schemas over a grid of good and bad arguments.

use dsh_rs::tools::args_schema::violations;
use serde_json::Value;

#[test]
fn every_argument_gets_the_violations_typescript_reports() {
    let schemas: Value = serde_json::from_str(include_str!("../src/schedule/tool_schemas.json")).unwrap();
    let cases: Vec<Value> = serde_json::from_str(include_str!("fixtures/tool_args_ts.json")).unwrap();
    assert!(cases.len() > 500);
    let mut wrong = Vec::new();
    for case in &cases {
        let tool = case["tool"].as_str().unwrap();
        let got = violations(&schemas[tool]["parameters"], &case["args"]);
        let expected: Vec<String> = case["violations"].as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_string()).collect();
        if got != expected && !is_undeclared_key_order(&got, &expected) {
            wrong.push(format!("{tool} {}\n   ts: {expected:?}\n   rs: {got:?}", case["args"]));
        }
    }
    assert!(wrong.is_empty(), "{} of {} differ:\n{}", wrong.len(), cases.len(), wrong.iter().take(8).cloned().collect::<Vec<_>>().join("\n"));
}

/// The one known difference: when an object carries several undeclared keys,
/// TypeScript lists them in the order the model wrote them, and RS lists them in
/// sorted order (a parsed JSON object here has no key order). The violations are
/// the same ones, and the leading `missing required` ones are in the same order.
fn is_undeclared_key_order(got: &[String], expected: &[String]) -> bool {
    let undeclared = |v: &&String| v.ends_with("(additionalProperties: false)");
    let (g_rest, e_rest): (Vec<&String>, Vec<&String>) = (got.iter().filter(|v| !undeclared(v)).collect(), expected.iter().filter(|v| !undeclared(v)).collect());
    let (mut g_tail, mut e_tail): (Vec<&String>, Vec<&String>) = (got.iter().filter(undeclared).collect(), expected.iter().filter(undeclared).collect());
    // The other violations must be equal AND sit at the same positions.
    let positions = |list: &[String]| list.iter().enumerate().filter(|(_, v)| !v.ends_with("(additionalProperties: false)")).map(|(i, _)| i).collect::<Vec<_>>();
    if g_rest != e_rest || positions(got) != positions(expected) || g_tail.len() < 2 {
        return false;
    }
    g_tail.sort();
    e_tail.sort();
    g_tail == e_tail
}

#[test]
fn the_undeclared_key_order_allowance_accepts_nothing_else() {
    let s = |text: &str| text.to_string();
    let one = s("\"a.x\" is not a declared property (additionalProperties: false)");
    let two = s("\"a.y\" is not a declared property (additionalProperties: false)");
    assert!(is_undeclared_key_order(&[one.clone(), two.clone()], &[two.clone(), one.clone()]));
    assert!(!is_undeclared_key_order(&[one.clone()], &[one.clone()]), "a single key has no order to differ in");
    assert!(!is_undeclared_key_order(&[one.clone(), two.clone()], &[one.clone(), s("other")]), "a different violation is not an ordering difference");
    assert!(!is_undeclared_key_order(&[s("missing"), one.clone(), two.clone()], &[one, two, s("missing")]), "other violations keep their order");
}
