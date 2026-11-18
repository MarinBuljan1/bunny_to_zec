//! Shared tokenization for conversation matching and rendering.

/// Preserve punctuation and individual spaces, including the original tokenizer's
/// quirk of discarding pipe characters. Tabs/newlines remain inside word fragments.
/// char_indices keeps every UTF-8 slice on a character boundary.
pub(crate) fn split_text(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut start = 0;
    for (index, character) in text.char_indices() {
        if !matches!(character, ' ' | ',' | '.' | '!' | '?' | '|') {
            continue;
        }
        if start < index {
            tokens.push(text[start..index].to_owned());
        }
        if character != '|' {
            tokens.push(character.to_string());
        }
        start = index + character.len_utf8();
    }
    if start < text.len() {
        tokens.push(text[start..].to_owned());
    }
    tokens
}

#[cfg(test)]
#[path = "tests/sentence_tests.rs"]
mod tests;
