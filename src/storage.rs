//! Browser networking and storage access. Downloads never write caches; the UI
//! commits validated content only after checking that its load is still current.

use crate::model::{LearningContent, SavedProgress};
use crate::persistence::{self, LoadedProgress, StorageError, Store};
use gloo_net::http::Request;
use serde::de::DeserializeOwned;
use std::cell::Cell;
use std::fmt;

struct BrowserStore;

impl BrowserStore {
    fn storage() -> Result<web_sys::Storage, StorageError> {
        web_sys::window()
            .ok_or(StorageError::Unavailable)?
            .local_storage()
            .map_err(|_| StorageError::Unavailable)?
            .ok_or(StorageError::Unavailable)
    }
}

impl Store for BrowserStore {
    fn get(&self, key: &str) -> Result<Option<String>, StorageError> {
        Self::storage()?
            .get_item(key)
            .map_err(|_| StorageError::ReadFailed)
    }
    fn set(&mut self, key: &str, value: &str) -> Result<(), StorageError> {
        Self::storage()?
            .set_item(key, value)
            .map_err(|_| StorageError::WriteFailed)
    }

    fn remove(&mut self, key: &str) -> Result<(), StorageError> {
        Self::storage()?
            .remove_item(key)
            .map_err(|_| StorageError::WriteFailed)
    }
}

#[derive(Debug)]
pub(crate) enum DownloadError {
    Network(&'static str),
    Http(&'static str, u16),
    InvalidJson(&'static str),
    InvalidContent(&'static str),
}

impl fmt::Display for DownloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network(resource) => write!(f, "Could not download {resource}."),
            Self::Http(resource, status) => {
                write!(f, "Downloading {resource} returned HTTP {status}.")
            }
            Self::InvalidJson(resource) => write!(f, "Downloaded {resource} was not valid JSON."),
            Self::InvalidContent(reason) => f.write_str(reason),
        }
    }
}

#[derive(PartialEq)]
pub(crate) enum ContentOrigin {
    Downloaded,
    Saved,
}

#[derive(PartialEq)]
pub(crate) struct LoadedContent {
    pub content: LearningContent,
    pub notice: Option<String>,
    pub origin: ContentOrigin,
}

#[derive(PartialEq)]
pub(crate) enum ContentLoadState {
    Loading,
    Ready(LoadedContent),
    Failed(String),
}

/// Shared transport, status, and JSON checks for each content resource.
async fn fetch_json<T: DeserializeOwned>(
    resource: &'static str,
    label: &'static str,
) -> Result<T, DownloadError> {
    let response = Request::get(resource)
        .send()
        .await
        .map_err(|_| DownloadError::Network(label))?;
    if !response.ok() {
        return Err(DownloadError::Http(label, response.status()));
    }
    response
        .json()
        .await
        .map_err(|_| DownloadError::InvalidJson(label))
}

/// The PWA precaches this relative asset; unlike scores it is not duplicated in local storage.
pub(crate) async fn load_conversations() -> Result<crate::conversations::ConversationLibrary, String>
{
    fetch_json("assets/conversations.json", "conversations")
        .await
        .map_err(|e| format!("{e} Connect and retry if this is your first visit."))
}

async fn download_content() -> Result<LearningContent, DownloadError> {
    let content = LearningContent {
        cards: fetch_json("assets/croatian.json", "vocabulary").await?,
        sentences: fetch_json("assets/croatian_sentences.json", "sentences").await?,
        explanations: fetch_json("assets/croatian_explanations.json", "explanations").await?,
    };
    Ok(content)
}

/// Fall back as a complete dataset instead of mixing resource versions. This
/// operation reads caches but never writes them, even if the UI is torn down.
pub(crate) async fn load_learning_content() -> Result<LoadedContent, String> {
    choose_content(download_content().await, &BrowserStore)
}

fn choose_content(
    download: Result<LearningContent, DownloadError>,
    store: &impl Store,
) -> Result<LoadedContent, String> {
    let validated = download.and_then(|content| {
        persistence::validate_content(&content).map_err(DownloadError::InvalidContent)?;
        Ok(content)
    });
    match validated {
        Ok(content) => Ok(LoadedContent {
            content,
            notice: None,
            origin: ContentOrigin::Downloaded,
        }),
        Err(error) => match persistence::read_content(store) {
            Ok(content) => Ok(LoadedContent {
                content,
                notice: Some(
                    "Using saved learning content because the latest files could not be loaded."
                        .into(),
                ),
                origin: ContentOrigin::Saved,
            }),
            Err(cached) => Err(format!(
                "{error} {cached} Connect to the internet and try again."
            )),
        },
    }
}

/// Call only after confirming the load is still active. Failure leaves the
/// validated in-memory content usable.
pub(crate) fn commit_content_load(
    result: Result<LoadedContent, String>,
    cancelled: &Cell<bool>,
) -> Option<ContentLoadState> {
    commit_content_load_to_store(result, &mut BrowserStore, cancelled)
}

fn commit_content_load_to_store(
    result: Result<LoadedContent, String>,
    store: &mut impl Store,
    cancelled: &Cell<bool>,
) -> Option<ContentLoadState> {
    if cancelled.get() {
        return None;
    }
    Some(match result {
        Ok(mut loaded) => {
            if loaded.origin == ContentOrigin::Downloaded {
                match persistence::write_content(store, &loaded.content) {
                    Ok(warning) => loaded.notice = warning,
                    Err(error) => loaded.notice = Some(format!("{error} You can keep learning, but a local fallback copy of these learning files could not be saved.")),
                }
            }
            ContentLoadState::Ready(loaded)
        }
        Err(message) => ContentLoadState::Failed(message),
    })
}

pub(crate) fn load_progress(now_s: i64) -> Result<LoadedProgress, StorageError> {
    persistence::read_progress(&BrowserStore, now_s)
}

pub(crate) fn save_progress(progress: &SavedProgress) -> Result<(), StorageError> {
    persistence::write_progress(&mut BrowserStore, progress)
}

#[cfg(test)]
#[path = "tests/storage_tests.rs"]
mod tests;
