//! The base64 alphabets a JSON wire carries bytes in.

/// The standard alphabet, with `+` and `/`.
const STANDARD: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// The URL-safe alphabet, with `-` and `_`.
const URL_SAFE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Encode some bytes in the standard alphabet.
pub fn encode(bytes: &[u8]) -> String {
    encode_with(bytes, STANDARD)
}

/// Encode some bytes in the URL-safe alphabet, without padding.
pub fn encode_url(bytes: &[u8]) -> String {
    let mut encoded = encode_with(bytes, URL_SAFE);
    while encoded.ends_with('=') {
        encoded.pop();
    }
    encoded
}

/// Encode some bytes in one alphabet.
fn encode_with(bytes: &[u8], alphabet: &[u8; 64]) -> String {
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let first = chunk[0] as u32;
        let second = chunk.get(1).copied().unwrap_or(0) as u32;
        let third = chunk.get(2).copied().unwrap_or(0) as u32;
        let triple = (first << 16) | (second << 8) | third;
        encoded.push(alphabet[(triple >> 18) as usize & 63] as char);
        encoded.push(alphabet[(triple >> 12) as usize & 63] as char);
        encoded.push(if chunk.len() > 1 {
            alphabet[(triple >> 6) as usize & 63] as char
        } else {
            '='
        });
        encoded.push(if chunk.len() > 2 {
            alphabet[triple as usize & 63] as char
        } else {
            '='
        });
    }
    encoded
}