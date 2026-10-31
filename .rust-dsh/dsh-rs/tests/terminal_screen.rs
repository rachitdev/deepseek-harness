//! The terminal screen against `@xterm/addon-serialize` itself.
//!
//! `tests/fixtures/xterm_serialize_ts.json` holds, for 112 inputs at three
//! screen sizes, the text `SerializeAddon.serialize()` produced after the same
//! bytes were written to an `@xterm/headless` terminal.

use dsh_rs::terminal::screen::Screen;
use serde_json::Value;

#[test]
fn every_screen_serialises_as_xterm_does() {
    let cases: Vec<Value> = serde_json::from_str(include_str!("fixtures/xterm_serialize_ts.json")).unwrap();
    assert_eq!(cases.len(), 112);
    let mut wrong = Vec::new();
    for case in &cases {
        let (cols, rows, scrollback) = (case["cols"].as_u64().unwrap() as u16, case["rows"].as_u64().unwrap() as u16, case["scrollback"].as_u64().unwrap() as usize);
        let mut screen = Screen::new(cols, rows, scrollback);
        screen.write(case["data"].as_str().unwrap().as_bytes());
        if let Some(size) = case.get("resizeTo") {
            screen.resize(size[0].as_u64().unwrap() as u16, size[1].as_u64().unwrap() as u16);
            screen.write(case["after"].as_str().unwrap().as_bytes());
        }
        let got = screen.serialize();
        if got != case["screen"].as_str().unwrap() {
            wrong.push(format!("{} {cols}x{rows}/{scrollback} {:?}\n   ts: {:?}\n   rs: {got:?}", case["name"], case["data"], case["screen"]));
        }
    }
    assert!(wrong.is_empty(), "{} of {} differ:\n{}", wrong.len(), cases.len(), wrong.iter().take(14).cloned().collect::<Vec<_>>().join("\n"));
}

/// Random streams of text, colours, cursor moves, erases, scrolls, line and
/// cell edits, and mode switches (including the alternate screen), written to an
/// `@xterm/headless` terminal. Every one must serialise as xterm serialises it.
#[test]
fn random_terminal_output_serialises_as_xterm_does() {
    let cases: Vec<Value> = serde_json::from_str(include_str!("fixtures/xterm_fuzz_ts.json")).unwrap();
    assert_eq!(cases.len(), 400);
    let mut wrong = Vec::new();
    for case in &cases {
        let (cols, rows, scrollback) = (case["cols"].as_u64().unwrap() as u16, case["rows"].as_u64().unwrap() as u16, case["scrollback"].as_u64().unwrap() as usize);
        let mut screen = Screen::new(cols, rows, scrollback);
        screen.write(case["data"].as_str().unwrap().as_bytes());
        let got = screen.serialize();
        if got != case["screen"].as_str().unwrap() {
            wrong.push(format!("{} {cols}x{rows}/{scrollback} {:?}\n   ts: {:?}\n   rs: {got:?}", case["name"], case["data"], case["screen"]));
        }
    }
    assert!(wrong.is_empty(), "{} of {} differ:\n{}", wrong.len(), cases.len(), wrong.iter().take(10).cloned().collect::<Vec<_>>().join("\n"));
}

/// Output arrives in arbitrary chunks, and a multi-byte character can be split.
#[test]
fn chunked_writes_give_the_same_screen_as_one_write() {
    let text = "ab\u{1b}[31mred\u{1b}[0m 日本 😀\r\nsecond\r\n".as_bytes();
    let mut whole = Screen::new(40, 5, 100);
    whole.write(text);
    let mut split = Screen::new(40, 5, 100);
    for byte in text {
        split.write(std::slice::from_ref(byte));
    }
    assert_eq!(whole.serialize(), split.serialize());
}
