//! Sentence rules independent of rendering and browser randomness.
//! Eligibility uses the ordered prefix below the previous review's word count,
//! falling back to the first index. Only the first eligible repeated word is hidden.

use crate::model::{Card, CardList, ScoreData};
use indexmap::IndexMap;
use std::collections::HashMap;

/// A sentence fragment with a translation only when its word should be hidden.
/// Some("") still means a matched word with an empty base-language entry.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ProcessedToken {
    pub text: String,
    pub translation: Option<String>,
}

/// Use a caller-supplied sample in [0, 1), as produced by browser Math.random.
pub(crate) fn choose_sentence_index(
    sentence_indexes: &[i32],
    max_index: i32,
    random_sample: f64,
) -> Option<i32> {
    let eligible_count = sentence_indexes
        .iter()
        .take_while(|&&index| index < max_index.max(1))
        .count();
    if eligible_count == 0 {
        return sentence_indexes.first().copied();
    }
    let random_index = (random_sample * eligible_count as f64).floor() as usize;
    sentence_indexes[..eligible_count]
        .get(random_index)
        .copied()
}

pub(crate) fn prepare_sentence(
    scores: &HashMap<String, ScoreData>,
    cards: &IndexMap<String, CardList>,
    words: Vec<String>,
    current_key: &str,
) -> (Vec<Card>, Vec<ProcessedToken>) {
    let mut review_words: Vec<Card> = Vec::new();
    let mut tokens = Vec::with_capacity(words.len());
    for text in words {
        let eligible = scores.contains_key(&text) || text == current_key;
        let already_included = review_words.iter().any(|word| word.target_language == text);
        let mut translation = None;
        if eligible && !already_included {
            if let Some(card) = cards.get(&text) {
                translation = Some(card.base_language.clone());
                review_words.push(Card {
                    base_language: card.base_language.clone(),
                    target_language: text.clone(),
                });
            }
        }
        tokens.push(ProcessedToken { text, translation });
    }
    (review_words, tokens)
}

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
