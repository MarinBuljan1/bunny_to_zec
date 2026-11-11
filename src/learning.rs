//! Learning rules without Yew, browser storage, clocks, or randomness.
//!
//! Scores measure seconds of retained learning. Stored review/update timestamps are
//! Unix seconds (`i32`, the existing schema); recall duration is milliseconds.
//! Capture `now_s` once at the browser boundary for a complete calculation.

use std::collections::HashMap;

use indexmap::IndexMap;

use crate::model::{Card, CardList, SavedProgress, ScoreData};

pub(crate) const LEARNED_INTERVAL_SECONDS: i32 = 10 * 86_400;
const REVIEW_PRIORITY_INTERVAL_SECONDS: i32 = LEARNED_INTERVAL_SECONDS + 120;

pub(crate) enum ReviewOutcome {
    Correct,
    Incorrect,
}

/// Recall inputs shared by all eligible words in one displayed card.
pub(crate) struct ReviewAttempt {
    pub elapsed_ms: i64,
    /// Retains the original combined UTF-8 byte length of the two displayed strings.
    pub character_length: i32,
    pub outcome: ReviewOutcome,
}

/// Calculate new progress without changing the input or saving it.
/// Unrevealed/near-zero-duration reviews are skipped, including decay, as before.
pub(crate) fn apply_review(
    progress: &SavedProgress,
    words: &[Card],
    attempt: &ReviewAttempt,
    now_s: i32,
) -> Option<SavedProgress> {
    if attempt.elapsed_ms <= 1 {
        return None;
    }

    let raw_scores: Vec<ScoreData> = words
        .iter()
        .map(|word| {
            progress
                .scores
                .get(&word.target_language)
                .cloned()
                .unwrap_or(ScoreData {
                    score: 0,
                    last_review: 0,
                })
        })
        .collect();
    let weights = review_weights(&raw_scores);
    let mut updated_scores = progress.scores.clone();
    let decay_seconds = now_s - progress.last_data_update;
    for score in updated_scores.values_mut() {
        score.score = (score.score - decay_seconds).max(0);
    }

    // Reviewed words use their pre-decay values and then replace the decayed
    // entries. Reordering those operations would change the existing formula.
    for ((word, raw), weight) in words.iter().zip(&raw_scores).zip(weights) {
        updated_scores.insert(
            word.target_language.clone(),
            calculate_raw_score(raw, attempt, weight, now_s),
        );
    }
    Some(SavedProgress {
        scores: updated_scores,
        last_data_update: now_s,
    })
}

/// Invert the clamped score shares so weaker words receive more review weight.
fn review_weights(raw_values: &[ScoreData]) -> Vec<f32> {
    let total_score: f32 = raw_values.iter().map(|entry| entry.score as f32).sum();
    let mut weights: Vec<f32> = raw_values
        .iter()
        .map(|entry| {
            let normalised = if total_score > 0.0 {
                entry.score as f32 / total_score
            } else {
                0.0
            };
            1.0 / normalised.max(0.1)
        })
        .collect();
    let total_weight: f32 = weights.iter().sum();
    for weight in &mut weights {
        *weight /= total_weight;
    }
    weights
}

fn calculate_raw_score(
    previous: &ScoreData,
    attempt: &ReviewAttempt,
    inverted_normalised_score: f32,
    now_s: i32,
) -> ScoreData {
    let mut new_score = previous.score;
    let mut new_last_review = now_s;
    let seconds_for_review =
        (attempt.elapsed_ms as f64 / 1000.0) as f32 * inverted_normalised_score;
    let mut seconds_since_last_review = (now_s - previous.last_review) as f32;
    let mut calculated_divisor =
        (seconds_for_review / 1.5) - attempt.character_length as f32 * 0.02;

    if previous.last_review == 0 {
        new_score = 0;
        seconds_since_last_review = LEARNED_INTERVAL_SECONDS as f32;
    }
    calculated_divisor = calculated_divisor.clamp(0.1, 10.0);
    match attempt.outcome {
        ReviewOutcome::Correct => {
            new_score += (seconds_since_last_review / calculated_divisor) as i32;
        }
        ReviewOutcome::Incorrect if previous.last_review != 0 => {
            new_last_review = (previous.last_review as f64
                + (seconds_since_last_review * inverted_normalised_score) as f64)
                as i32;
        }
        ReviewOutcome::Incorrect => {}
    }
    ScoreData {
        score: new_score,
        last_review: new_last_review,
    }
}

fn calculate_final_score(score: &ScoreData, now_s: i32) -> i32 {
    if score.last_review == 0 {
        return LEARNED_INTERVAL_SECONDS;
    }
    score.score + (score.last_review + REVIEW_PRIORITY_INTERVAL_SECONDS - now_s).max(0)
}

/// Exclude current review words, then select the lowest priority score.
/// `IndexMap` insertion order resolves equal scores; retain that order.
pub(crate) fn find_key_of_lowest_score(
    cards: &IndexMap<String, CardList>,
    scores: &HashMap<String, ScoreData>,
    current_words: &[Card],
    now_s: i32,
) -> Option<String> {
    cards
        .iter()
        .filter(|(key, _)| {
            !current_words
                .iter()
                .any(|word| word.target_language == key.as_str())
        })
        .min_by_key(|(key, _)| {
            scores
                .get(*key)
                .map(|score| calculate_final_score(score, now_s))
                .unwrap_or(LEARNED_INTERVAL_SECONDS)
        })
        .map(|(key, _)| key.clone())
}

pub(crate) fn calculate_progress_sum(scores: &HashMap<String, ScoreData>) -> i64 {
    scores
        .values()
        .map(|score| i64::from(score.score.min(LEARNED_INTERVAL_SECONDS)))
        .sum()
}

#[cfg(test)]
#[path = "tests/learning_tests.rs"]
mod tests;
