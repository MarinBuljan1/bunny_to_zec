//! Versioned snapshots and legacy readers, independent of browser APIs.
//! A snapshot is committed with one storage write. Legacy progress is retained;
//! legacy content copies are removed only after the new snapshot is committed.

use crate::model::{LearningContent, SavedProgress, ScoreData};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

const CONTENT_KEY: &str = "learning_content_v1";
// Keep the existing key so version-1 clients refuse to overwrite wider progress.
const PROGRESS_KEY: &str = "learning_progress_v1";

// Reviewed spelling corrections; the asset manifest is checked against this list.
// These aliases name the same word, not different grammatical forms.
const VOCABULARY_CORRECTIONS: [(&str, &str); 3] =
    [("covek", "čovjek"), ("pomoči", "pomoći"), ("o.k.", "okej")];

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum StorageError {
    Unavailable,
    ReadFailed,
    WriteFailed,
    Missing(&'static str),
    Invalid(&'static str),
    UnsupportedVersion(&'static str),
}

impl fmt::Display for StorageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Unavailable => "Browser storage is unavailable or blocked.",
            Self::ReadFailed => "Browser storage could not be read.",
            Self::WriteFailed => "Browser storage could not be written. It may be full or blocked.",
            Self::Missing(key) => {
                return write!(formatter, "No saved {} was found.", stored_label(key))
            }
            Self::Invalid(key) => {
                return write!(
                    formatter,
                    "Saved {} is malformed or invalid.",
                    stored_label(key)
                )
            }
            Self::UnsupportedVersion(key) => {
                return write!(
                    formatter,
                    "Saved {} uses an unsupported format version.",
                    stored_label(key)
                )
            }
        };
        formatter.write_str(message)
    }
}

fn stored_label(key: &'static str) -> &'static str {
    match key {
        CONTENT_KEY | "cards" | "sentences" | "explanations" => "learning content",
        PROGRESS_KEY | "card_scores" | "last_data_update" => "learning progress",
        label => label,
    }
}

/// Browser and in-memory implementations share this boundary. No broad storage
/// clearing is exposed; removal is used only for known legacy content keys.
/// Like Web Storage, `set` must leave the previous value intact on failure.
pub(crate) trait Store {
    fn get(&self, key: &str) -> Result<Option<String>, StorageError>;
    fn set(&mut self, key: &str, value: &str) -> Result<(), StorageError>;
    fn remove(&mut self, key: &str) -> Result<(), StorageError>;
}

#[derive(Serialize, Deserialize)]
struct Snapshot<T> {
    version: u32,
    value: T,
}

fn read_json<T: DeserializeOwned>(
    store: &impl Store,
    key: &'static str,
) -> Result<Option<T>, StorageError> {
    store
        .get(key)?
        .map(|raw| serde_json::from_str(&raw).map_err(|_| StorageError::Invalid(key)))
        .transpose()
}

fn read_snapshot<T: DeserializeOwned>(
    store: &impl Store,
    key: &'static str,
    supported_versions: &[u32],
) -> Result<Option<T>, StorageError> {
    let Some(raw) = store.get(key)? else {
        return Ok(None);
    };
    #[derive(Deserialize)]
    struct Version {
        version: u32,
    }
    let version: Version = serde_json::from_str(&raw).map_err(|_| StorageError::Invalid(key))?;
    if !supported_versions.contains(&version.version) {
        return Err(StorageError::UnsupportedVersion(key));
    }
    let snapshot: Snapshot<T> =
        serde_json::from_str(&raw).map_err(|_| StorageError::Invalid(key))?;
    Ok(Some(snapshot.value))
}

fn write_snapshot<T: Serialize>(
    store: &mut impl Store,
    key: &'static str,
    version: u32,
    value: &T,
) -> Result<(), StorageError> {
    let raw = serde_json::to_string(&Snapshot { version, value })
        .map_err(|_| StorageError::Invalid(key))?;
    store.set(key, &raw)
}

/// Ordered, in-range references are required by the sentence-selection rule.
pub(crate) fn validate_content(content: &LearningContent) -> Result<(), &'static str> {
    if content.cards.is_empty() {
        return Err("The vocabulary is empty.");
    }
    for card in content.cards.values() {
        if card
            .sentence_indexes
            .iter()
            .any(|&index| index < 0 || index as usize >= content.sentences.len())
        {
            return Err("Vocabulary contains an out-of-range sentence reference.");
        }
        if card
            .sentence_indexes
            .windows(2)
            .any(|pair| pair[0] > pair[1])
        {
            return Err("Vocabulary sentence references are not ordered.");
        }
    }
    Ok(())
}

pub(crate) fn read_content(store: &impl Store) -> Result<LearningContent, StorageError> {
    let content = match read_snapshot(store, CONTENT_KEY, &[1])? {
        Some(content) => content,
        None => LearningContent {
            cards: read_json(store, "cards")?.ok_or(StorageError::Missing("vocabulary"))?,
            sentences: read_json(store, "sentences")?.ok_or(StorageError::Missing("sentences"))?,
            explanations: read_json(store, "explanations")?.unwrap_or_default(),
        },
    };
    validate_content(&content).map_err(|_| StorageError::Invalid("learning content"))?;
    Ok(content)
}

pub(crate) fn write_content(
    store: &mut impl Store,
    content: &LearningContent,
) -> Result<Option<String>, StorageError> {
    validate_content(content).map_err(|_| StorageError::Invalid("learning content"))?;
    write_snapshot(store, CONTENT_KEY, 1, content)?;
    // Free duplicate content space for progress, but only after an atomic commit.
    for key in ["cards", "sentences", "explanations"] {
        if store.remove(key).is_err() {
            return Ok(Some("Learning files were saved, but older cached copies could not be removed. Browser storage may fill sooner.".into()));
        }
    }
    Ok(None)
}

pub(crate) struct LoadedProgress {
    pub progress: SavedProgress,
    pub warning: Option<String>,
}

/// Keep the latest actual review when two former spellings have separate scores.
/// Equal review times use the higher score; summing would invent learning credit.
fn correct_progress_spellings(progress: &mut SavedProgress) {
    for (original, corrected) in VOCABULARY_CORRECTIONS {
        let Some(previous) = progress.scores.remove(original) else {
            continue;
        };
        let keep_previous = progress.scores.get(corrected).is_none_or(|existing| {
            (previous.last_review, previous.score) > (existing.last_review, existing.score)
        });
        if keep_previous {
            progress.scores.insert(corrected.to_owned(), previous);
        }
    }
}

fn validate_progress(progress: &SavedProgress) -> Result<(), StorageError> {
    if progress.last_data_update < 0
        || progress
            .scores
            .values()
            .any(|entry| entry.score < 0 || entry.last_review < 0)
    {
        return Err(StorageError::Invalid("progress"));
    }
    Ok(())
}

pub(crate) fn read_progress(
    store: &impl Store,
    now_s: i64,
) -> Result<LoadedProgress, StorageError> {
    // Existing JSON integer values decode exactly into i64; no rescaling is needed.
    if let Some(mut progress) = read_snapshot(store, PROGRESS_KEY, &[1, 2])? {
        validate_progress(&progress)?;
        correct_progress_spellings(&mut progress);
        return Ok(LoadedProgress {
            progress,
            warning: None,
        });
    }
    let scores: HashMap<String, ScoreData> = read_json(store, "card_scores")?.unwrap_or_default();
    // Preserve readable scores when the separate legacy timestamp is bad. Using
    // now avoids inventing a large decay interval; legacy keys are left untouched.
    let (last_data_update, warning) = match read_json::<i64>(store, "last_data_update") {
        Ok(Some(timestamp)) if timestamp >= 0 => (timestamp, None),
        Ok(None) if scores.is_empty() => (now_s, None),
        Ok(_) | Err(StorageError::Invalid(_)) => (now_s, Some("Saved scores were restored, but their update time was missing or invalid. Decay resumes from now.".into())),
        Err(error) => return Err(error),
    };
    let mut progress = SavedProgress {
        scores,
        last_data_update,
    };
    validate_progress(&progress)?;
    correct_progress_spellings(&mut progress);
    Ok(LoadedProgress { progress, warning })
}

/// Refuse to overwrite unreadable/newer progress with an empty fallback.
/// Successful saves lazily migrate legacy progress into one atomic snapshot.
pub(crate) fn write_progress(
    store: &mut impl Store,
    progress: &SavedProgress,
) -> Result<(), StorageError> {
    validate_progress(progress)?;
    read_progress(store, progress.last_data_update)?;
    if VOCABULARY_CORRECTIONS
        .iter()
        .any(|(original, _)| progress.scores.contains_key(*original))
    {
        let mut corrected = SavedProgress {
            scores: progress.scores.clone(),
            last_data_update: progress.last_data_update,
        };
        correct_progress_spellings(&mut corrected);
        return write_snapshot(store, PROGRESS_KEY, 2, &corrected);
    }
    write_snapshot(store, PROGRESS_KEY, 2, progress)
}

#[cfg(test)]
#[path = "tests/persistence_tests.rs"]
mod tests;
