//! Browser loading and persistence boundaries. Existing keys and JSON formats are retained.
//! Content loading retains the existing network-first/cache-fallback behavior;
//! HTTP/parse failures and cache-write recovery are tracked separately under R5.

use crate::model::{CardList, CardSentences, LearningContent, SavedProgress};
use gloo_net::http::Request;
use gloo_storage::{errors::StorageError, LocalStorage, Storage};
use indexmap::IndexMap;
use std::collections::HashMap;

pub(crate) async fn load_learning_content() -> LearningContent {
    // Fetch cards
    let fetched_cards: IndexMap<String, CardList> =
        match Request::get("assets/croatian.json").send().await {
            Ok(response) => {
                let cards: IndexMap<String, CardList> =
                    response.json().await.expect("Failed to parse JSON");

                LocalStorage::set("cards", &cards).expect("Failed to store cards");
                cards
            }
            Err(_) => LocalStorage::get::<IndexMap<String, CardList>>("cards")
                .unwrap_or_else(|_| IndexMap::new()),
        };

    // Fetch sentences
    let card_sentences: Vec<CardSentences> = match Request::get("assets/croatian_sentences.json")
        .send()
        .await
    {
        Ok(response) => {
            let sentences: Vec<CardSentences> =
                response.json().await.expect("Failed to parse JSON");

            LocalStorage::set("sentences", &sentences).expect("Failed to store sentences");
            sentences
        }
        Err(_) => LocalStorage::get::<Vec<CardSentences>>("sentences").unwrap_or_else(|_| vec![]),
    };

    // Fetch explanations
    let fetched_explanations: HashMap<String, HashMap<String, String>> =
        match Request::get("assets/croatian_explanations.json")
            .send()
            .await
        {
            Ok(response) => {
                let explanations: HashMap<String, HashMap<String, String>> = response
                    .json()
                    .await
                    .expect("Failed to parse explanations JSON");

                LocalStorage::set("explanations", &explanations)
                    .expect("Failed to store explanations");
                explanations
            }
            Err(_) => LocalStorage::get("explanations").unwrap_or_else(|_| HashMap::new()),
        };

    LearningContent {
        cards: fetched_cards,
        sentences: card_sentences,
        explanations: fetched_explanations,
    }
}

pub(crate) fn load_progress(now_s: i32) -> SavedProgress {
    SavedProgress {
        scores: LocalStorage::get("card_scores").unwrap_or_default(),
        last_data_update: LocalStorage::get("last_data_update").unwrap_or(now_s),
    }
}

/// The two legacy keys are written in their original order. A partial write can
/// still occur if the second write fails; atomic/versioned storage is part of R5.
pub(crate) fn save_progress(progress: &SavedProgress) -> Result<(), StorageError> {
    LocalStorage::set("card_scores", &progress.scores)?;
    LocalStorage::set("last_data_update", progress.last_data_update)
}
