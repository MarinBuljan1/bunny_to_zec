//! Sentence rules independent of rendering and browser randomness.
//! The eligible indices are an ordered prefix below the previous review's word count;
//! if that prefix is empty, the first index remains the fallback. Only the first
//! eligible occurrence of a repeated word is hidden, matching existing behavior.

use crate::model::{Card, CardList, ProcessingData, ScoreData};
use indexmap::IndexMap;
use regex::Regex;
use std::collections::HashMap;

/// Use a caller-supplied sample in [0, 1), as produced by browser Math.random.
pub(crate) fn choose_sentence_index(
    sentence_indexes: &[i32],
    max_index: i32,
    random_sample: f64,
) -> Option<i32> {
    if sentence_indexes.is_empty() {
        return None;
    }

    let effective_max = max_index.max(1);

    let valid_sentence_indexes: Vec<i32> = sentence_indexes
        .iter()
        .take_while(|&&val| val < effective_max)
        .cloned()
        .collect();

    let valid_len = valid_sentence_indexes.len();

    if valid_len == 0 {
        return Some(sentence_indexes[0]);
    }

    let random_index = (random_sample * valid_len as f64).floor() as usize;
    valid_sentence_indexes.get(random_index).copied()
}

fn punctuation_list() -> Vec<char> {
    vec![' ', ',', '.', '!', '?']
}

pub(crate) fn find_related_cards(
    updated_scores: &HashMap<String, ScoreData>,
    cards: &IndexMap<String, CardList>,
    words: Vec<String>,
    current_key: &str,
) -> (Vec<Card>, Vec<ProcessingData>) {
    let mut related_cards = Vec::new();

    let mut processed_sentence_data: Vec<ProcessingData> = words
        .iter()
        .map(|word| ProcessingData {
            word: word.clone(),
            found: false,
            translation: "".to_string(),
        })
        .collect();

    for processed_word in processed_sentence_data.iter_mut() {
        let word_key = &processed_word.word;

        if updated_scores.contains_key(word_key) || word_key == current_key {
            // Only add if not already in the related_cards list
            if !related_cards
                .iter()
                .any(|c: &Card| &c.target_language == word_key)
            {
                if let Some(matching_card) = cards.get(word_key) {
                    processed_word.found = true;
                    processed_word.translation = matching_card.base_language.clone();

                    related_cards.push(Card {
                        base_language: matching_card.base_language.clone(),
                        target_language: word_key.clone(),
                    });
                }
            }
        }
    }

    (related_cards, processed_sentence_data)
}

pub(crate) fn split_text(text: &str) -> Vec<String> {
    let punctuations: String = punctuation_list()
        .iter()
        .map(|&c| regex::escape(&c.to_string()))
        .collect::<Vec<String>>()
        .join("|");

    let pattern = format!("({punctuations})|([^{punctuations}]+)");
    let re = Regex::new(&pattern).expect("the fixed, escaped punctuation pattern must be valid");

    re.find_iter(text).map(|m| m.as_str().to_string()).collect()
}

#[cfg(test)]
#[path = "tests/sentence_tests.rs"]
mod tests;
