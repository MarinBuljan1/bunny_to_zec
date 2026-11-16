//! One compact entry per relevant Croatian form, without duplicate translation headings.
use crate::model::{Card, LearningContent, ScoreData};
use std::collections::{HashMap, HashSet};

#[derive(Debug, PartialEq)]
pub(crate) struct WordNote {
    pub form: String,
    pub translation: String,
    pub explanation: Option<String>,
}

pub(crate) fn word_notes(
    words: &[Card],
    content: &LearningContent,
    scores: &HashMap<String, ScoreData>,
) -> Vec<WordNote> {
    let mut notes = Vec::new();
    let mut seen = HashSet::new();
    // Keep the tested forms first, in message order.
    for word in words {
        if !seen.insert(word.target_language.clone()) {
            continue;
        }
        let explanation = content
            .explanations
            .get(&word.base_language)
            .and_then(|entries| entries.get(&word.target_language))
            .cloned();
        notes.push(WordNote {
            form: word.target_language.clone(),
            translation: word.base_language.clone(),
            explanation,
        });
    }
    // Retain notes for introduced related forms, once each and in vocabulary order.
    let mut related = Vec::new();
    for word in words {
        let Some(entries) = content.explanations.get(&word.base_language) else {
            continue;
        };
        for (form, explanation) in entries {
            if !scores.contains_key(form) || seen.contains(form) {
                continue;
            }
            let Some((rank, _, card)) = content.cards.get_full(form) else {
                continue;
            };
            seen.insert(form.clone());
            related.push((
                rank,
                WordNote {
                    form: form.clone(),
                    translation: card.base_language.clone(),
                    explanation: Some(explanation.clone()),
                },
            ));
        }
    }
    related.sort_by_key(|(rank, _)| *rank);
    notes.extend(related.into_iter().map(|(_, note)| note));
    notes
}

#[cfg(test)]
#[path = "tests/word_note_tests.rs"]
mod tests;
