//! Session title text rules, ported from the TypeScript `session-title`
//! package (`normalize.ts`) so a title accepted here reads back identically
//! there: terminal escapes and invisible direction controls are removed, runs
//! of whitespace become one space, and the result is cut to a UTF-8 byte
//! budget without splitting a character.

use std::sync::OnceLock;

use regex::Regex;

/// Patterns applied in order. Escape sequences go before the lone control
/// characters, or their bodies would be left behind as visible text.
fn patterns() -> &'static [Regex; 5] {
    static PATTERNS: OnceLock<[Regex; 5]> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        [
            // Operating-system commands, including an unterminated tail.
            Regex::new(r"(?s)(?:\x1B\]|\u{9D}).*?(?:\x07|\x1B\\|\z)").expect("OSC pattern"),
            // Control-sequence introducers such as colour codes.
            Regex::new(r"(?:\x1B\[|\u{9B})[0-?]*[ -/]*[@-~]").expect("CSI pattern"),
            // Remaining two-byte ESC sequences.
            Regex::new(r"\x1B[@-_]").expect("ESC pattern"),
            // C0/C1 controls other than tab, newline and carriage return.
            Regex::new(r"[\x00-\x08\x0B\x0C\x0E-\x1F\x7F-\u{9F}]").expect("control pattern"),
            // Direction and invisible controls that can disguise a title.
            Regex::new(r"[\u{200B}\u{200E}\u{200F}\u{202A}-\u{202E}\u{2060}-\u{2064}\u{2066}-\u{206F}\u{FEFF}]")
                .expect("direction pattern"),
        ]
    })
}

/// One trimmed, whitespace-normalised line with controls removed.
fn clean(input: &str) -> String {
    let mut text = input.to_string();
    for pattern in patterns() {
        text = pattern.replace_all(&text, "").into_owned();
    }
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Cut `text` to at most `max_bytes` UTF-8 bytes on a character boundary.
fn truncate_utf8(text: &str, max_bytes: usize) -> &str {
    if text.len() <= max_bytes {
        return text;
    }
    let mut end = max_bytes;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

/// Normalise an untrusted title. The result is empty when nothing visible is left.
pub fn normalize(input: &str, max_bytes: usize) -> String {
    truncate_utf8(&clean(input), max_bytes).trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whitespace_collapses_and_the_ends_are_trimmed() {
        assert_eq!(normalize("  a \t\n b   c  ", 100), "a b c");
    }

    #[test]
    fn terminal_escapes_are_removed_with_their_bodies() {
        assert_eq!(normalize("\x1b[31mred\x1b[0m title", 100), "red title");
        assert_eq!(normalize("a\x1b]0;evil window title\x07b", 100), "ab");
        assert_eq!(normalize("a\x1b]0;unterminated", 100), "a");
        assert_eq!(normalize("a\x1bMb", 100), "ab");
    }

    #[test]
    fn invisible_direction_controls_cannot_disguise_a_title() {
        assert_eq!(normalize("safe\u{202E}gnp.exe\u{200B}", 100), "safegnp.exe");
    }

    #[test]
    fn a_title_with_nothing_visible_normalises_to_empty() {
        assert_eq!(normalize("\x07\u{200B}  \n", 100), "");
    }

    #[test]
    fn truncation_never_splits_a_character() {
        // "é" is two bytes, so a three-byte budget holds only one of them.
        assert_eq!(normalize("éé", 3), "é");
        assert_eq!(normalize("ab", 1), "a");
        assert_eq!(normalize("a éb", 2), "a");
    }
}
