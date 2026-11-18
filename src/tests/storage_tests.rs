use super::*;
use crate::memory_store::{content, MemoryStore};

#[test]
fn network_http_and_parse_errors_use_the_last_valid_complete_cache() {
    let mut store = MemoryStore::default();
    store.cache_content(&content());
    let before = store.values.clone();
    for error in [
        DownloadError::Network("vocabulary"),
        DownloadError::Http("vocabulary", 404),
        DownloadError::InvalidJson("explanations"),
    ] {
        let loaded = choose_content(Err(error), &store).unwrap();
        assert_eq!(loaded.content, content());
        assert!(loaded.notice.is_some());
    }
    assert_eq!(store.values, before);
}

#[test]
fn invalid_downloads_do_not_replace_valid_cached_content() {
    let mut store = MemoryStore::default();
    store.cache_content(&content());
    let before = store.values.clone();
    let mut invalid = content();
    invalid.cards.clear();
    let loaded = choose_content(Ok(invalid), &store).unwrap();
    let state = commit_content_load_to_store(loaded, &mut store, &Cell::new(false)).unwrap();
    assert_eq!(state.content, content());
    assert_eq!(store.values, before);
}

#[test]
fn no_network_or_saved_content_produces_an_actionable_failure() {
    let message = choose_content(
        Err(DownloadError::Network("vocabulary")),
        &MemoryStore::default(),
    )
    .err()
    .unwrap();
    assert!(message.contains("Connect to the internet and try again"));
}

#[test]
fn successful_download_does_not_write_content_even_when_storage_is_full() {
    let mut store = MemoryStore {
        write_error: Some(StorageError::WriteFailed),
        ..MemoryStore::default()
    };
    let loaded = choose_content(Ok(content()), &store).unwrap();
    let loaded = commit_content_load_to_store(loaded, &mut store, &Cell::new(false)).unwrap();
    assert_eq!(loaded.content, content());
    assert!(loaded.notice.is_none());
    assert!(store.values.is_empty());
    assert_eq!(store.writes, 0);
}

#[test]
fn successful_retry_retires_legacy_content_without_touching_progress() {
    let mut store = MemoryStore::default();
    store.cache_content(&content());
    for key in ["cards", "sentences", "explanations"] {
        store.values.insert(key.into(), "old content".into());
    }
    for key in [
        "learning_progress_v1",
        "card_scores",
        "last_data_update",
        "another-app",
    ] {
        store.values.insert(key.into(), "preserve exactly".into());
    }
    let result = choose_content(Ok(content()), &store).unwrap();
    let loaded = commit_content_load_to_store(result, &mut store, &Cell::new(false)).unwrap();
    assert!(loaded.notice.is_none());
    assert_eq!(loaded.content, content());
    assert_eq!(store.values.len(), 4);
    assert!(store
        .values
        .values()
        .all(|value| value == "preserve exactly"));
    assert_eq!(store.writes, 0);
}

#[test]
fn cancelled_or_superseded_loads_neither_commit_caches_nor_return_ui_updates() {
    let mut store = MemoryStore::default();
    store.cache_content(&content());
    let before = store.values.clone();
    let loaded = choose_content(Ok(content()), &store).unwrap();
    assert!(commit_content_load_to_store(loaded, &mut store, &Cell::new(true)).is_none());
    assert_eq!(store.values, before);
    assert_eq!(store.writes, 0);
}

#[test]
fn offline_fallback_is_read_without_writing_or_removing_it() {
    let mut store = MemoryStore::default();
    let original = r#"{"version":1,"value":{"cards":{"bok":{"base_language":"hello","sentence_indexes":[0]}},"sentences":[],"explanations":{}}}"#;
    store
        .values
        .insert("learning_content_v1".into(), original.into());
    let load = |store: &MemoryStore| {
        choose_content(Err(DownloadError::Network("vocabulary")), store).unwrap()
    };
    let cancelled = load(&store);
    assert!(commit_content_load_to_store(cancelled, &mut store, &Cell::new(true)).is_none());
    assert_eq!(store.values["learning_content_v1"], original);

    store.write_error = Some(StorageError::WriteFailed);
    let loaded = load(&store);
    let usable = commit_content_load_to_store(loaded, &mut store, &Cell::new(false)).unwrap();
    assert_eq!(usable.content, content());
    let notice = usable.notice.unwrap();
    assert!(notice.contains("Using saved learning content"));
    assert_eq!(
        notice,
        "Using saved learning content because the latest files could not be loaded."
    );
    assert_eq!(store.values["learning_content_v1"], original);

    store.write_error = None;
    let loaded = load(&store);
    let restored = commit_content_load_to_store(loaded, &mut store, &Cell::new(false)).unwrap();
    assert!(restored
        .notice
        .unwrap()
        .contains("Using saved learning content"));
    assert_eq!(store.values["learning_content_v1"], original);
    assert_eq!(store.writes, 0);
}

#[test]
fn cleanup_failure_keeps_downloaded_content_usable_and_reports_the_actual_failure() {
    let mut store = MemoryStore {
        remove_error: Some(StorageError::WriteFailed),
        ..MemoryStore::default()
    };
    store.cache_content(&content());
    let before = store.values.clone();
    let loaded = choose_content(Ok(content()), &store).unwrap();
    let loaded = commit_content_load_to_store(loaded, &mut store, &Cell::new(false)).unwrap();
    assert_eq!(loaded.content, content());
    assert!(loaded.notice.unwrap().contains("could not be removed"));
    assert_eq!(store.values, before);
    assert_eq!(store.writes, 0);
}
