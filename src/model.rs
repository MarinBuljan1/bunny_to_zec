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
pub(crate) struct CardList {
    pub(crate) base_language: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
/// Stored score in learning seconds and review time in Unix seconds, both i64.
pub(crate) struct ScoreData {
    pub(crate) score: i64,
    pub(crate) last_review: i64,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
pub(crate) struct LearningContent {
    pub(crate) cards: IndexMap<String, CardList>,
    pub(crate) explanations: HashMap<String, HashMap<String, String>>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub(crate) struct SavedProgress {
    pub(crate) scores: HashMap<String, ScoreData>,
    pub(crate) last_data_update: i64,
    #[serde(default)]
    pub(crate) conversations: crate::conversations::ConversationProgress,
}
