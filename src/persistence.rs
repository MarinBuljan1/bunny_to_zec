//! Versioned snapshots and legacy readers, independent of browser APIs.
//! Progress snapshots are committed with one storage write. Historical content
//! remains readable until validated downloads replace it; new assets use PWA caching.

use crate::model::{LearningContent, SavedProgress, ScoreData};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::LazyLock;

const CONTENT_KEY: &str = "learning_content_v1";
// Last localStorage content schema; new content is cached by the service worker.
const CONTENT_VERSION: u32 = 2;
// Keep the existing key so version-1 clients refuse to overwrite wider progress.
const PROGRESS_KEY: &str = "learning_progress_v1";

#[derive(Deserialize)]
struct VocabularyCorrection {
    original: String,
    corrected: String,
}

#[derive(Deserialize)]
struct VocabularyCorrections {
    corrections: Vec<VocabularyCorrection>,
}

// Share the reviewed content manifest instead of maintaining a second alias list.
// Split phrases deliberately have no score alias: choosing one constituent would
// award credit for a different review target. Unmatched legacy scores are retained.
static VOCABULARY_CORRECTIONS: LazyLock<VocabularyCorrections> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../assets/vocabulary_corrections.json")).expect(
        "The bundled vocabulary correction manifest must be valid; content tests validate it",
    )
});

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

/// Vocabulary order determines review ties. Older sentence fields are ignored on read.
pub(crate) fn validate_content(content: &LearningContent) -> Result<(), &'static str> {
    if content.cards.is_empty() {
        return Err("The vocabulary is empty.");
    }
    if content.cards.keys().any(|word| word.trim().is_empty()) {
        return Err("Vocabulary contains an empty word.");
    }
    Ok(())
}

pub(crate) fn read_content(store: &impl Store) -> Result<LearningContent, StorageError> {
    let content = match read_snapshot(store, CONTENT_KEY, &[1, CONTENT_VERSION])? {
        Some(content) => content,
        None => LearningContent {
            cards: read_json(store, "cards")?.ok_or(StorageError::Missing("vocabulary"))?,
            explanations: read_json(store, "explanations")?.unwrap_or_default(),
        },
    };
    validate_content(&content).map_err(|_| StorageError::Invalid("learning content"))?;
    Ok(content)
}

/// Retire only obsolete learning files, after a complete download has been validated.
/// Progress, conversation history and unrelated same-origin keys are never removed.
pub(crate) fn remove_legacy_content(store: &mut impl Store) -> Result<(), StorageError> {
    for key in [CONTENT_KEY, "cards", "sentences", "explanations"] {
        store.remove(key)?;
    }
    Ok(())
}

pub(crate) struct LoadedProgress {
    pub progress: SavedProgress,
    pub warning: Option<String>,
}

/// Keep the latest actual review when two former spellings have separate scores.
/// Equal review times use the higher score; summing would invent learning credit.
fn correct_progress_spellings(progress: &mut SavedProgress) {
    for correction in &VOCABULARY_CORRECTIONS.corrections {
        let Some(previous) = progress.scores.remove(&correction.original) else {
            continue;
        };
        let keep_previous = progress
            .scores
            .get(&correction.corrected)
            .is_none_or(|existing| {
                (previous.last_review, previous.score) > (existing.last_review, existing.score)
            });
        if keep_previous {
            progress
                .scores
                .insert(correction.corrected.clone(), previous);
        }
    }
}

fn validate_progress(progress: &SavedProgress) -> Result<(), StorageError> {
    if !progress.conversations.structurally_valid()
        || progress.last_data_update < 0
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
    if let Some(mut progress) = read_snapshot(store, PROGRESS_KEY, &[1, 2, 3, 4])? {
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
        conversations: Default::default(),
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
        .corrections
        .iter()
        .any(|correction| progress.scores.contains_key(&correction.original))
    {
        let mut corrected = SavedProgress {
            scores: progress.scores.clone(),
            last_data_update: progress.last_data_update,
            conversations: progress.conversations.clone(),
        };
        correct_progress_spellings(&mut corrected);
        return write_snapshot(store, PROGRESS_KEY, 4, &corrected);
    }
    write_snapshot(store, PROGRESS_KEY, 4, progress)
}

#[cfg(test)]
#[path = "tests/persistence_tests.rs"]
mod tests;
