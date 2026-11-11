//! Shared vocabulary and progress schemas. Field names and timestamp units match existing JSON/local storage.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct Card {
    pub(crate) base_language: String,
    pub(crate) target_language: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct CurrentTranslations {
    pub(crate) list: Vec<Card>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct CardList {
    pub(crate) base_language: String,
    pub(crate) sentence_indexes: Vec<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct CardSentences {
    pub(crate) target_language: String,
    pub(crate) base_language: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub(crate) struct ScoreData {
    pub(crate) score: i32,
    pub(crate) last_review: i32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub(crate) struct ProcessingData {
    pub(crate) word: String,
    pub(crate) found: bool,
    pub(crate) translation: String,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
pub(crate) struct LearningContent {
    pub(crate) cards: IndexMap<String, CardList>,
    pub(crate) sentences: Vec<CardSentences>,
    pub(crate) explanations: HashMap<String, HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub(crate) struct SavedProgress {
    pub(crate) scores: HashMap<String, ScoreData>,
    pub(crate) last_data_update: i32,
}
