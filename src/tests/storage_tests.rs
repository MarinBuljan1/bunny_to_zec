use super::*;
use crate::memory_store::{content, MemoryStore};

#[test]
fn network_http_and_parse_errors_use_the_last_valid_complete_cache() {
    let mut store = MemoryStore::default();
    persistence::write_content(&mut store, &content()).unwrap();
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
    persistence::write_content(&mut store, &content()).unwrap();
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
fn cache_write_failure_keeps_downloaded_content_usable_with_a_warning() {
    let mut store = MemoryStore {
        write_error: Some(StorageError::WriteFailed),
        ..MemoryStore::default()
    };
    let loaded = choose_content(Ok(content()), &store).unwrap();
    let loaded = commit_content_load_to_store(loaded, &mut store, &Cell::new(false)).unwrap();
    assert_eq!(loaded.content, content());
    assert!(loaded.notice.unwrap().contains("keep learning"));
    assert_eq!(store.writes, 0);
}

#[test]
fn successful_retry_can_replace_a_failed_state() {
    let mut store = MemoryStore::default();
    let result = choose_content(Ok(content()), &store).unwrap();
    assert!(commit_content_load_to_store(result, &mut store, &Cell::new(false)).is_some());
    assert_eq!(persistence::read_content(&store).unwrap(), content());
}

#[test]
fn cancelled_or_superseded_loads_neither_commit_caches_nor_return_ui_updates() {
    let mut store = MemoryStore::default();
    let loaded = choose_content(Ok(content()), &store).unwrap();
    assert!(commit_content_load_to_store(loaded, &mut store, &Cell::new(true)).is_none());
    assert!(store.values.is_empty());
    assert_eq!(store.writes, 0);
}

#[test]
fn offline_content_migrates_only_after_validation_and_keeps_warnings_on_failure() {
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
    assert!(notice.contains("keep learning"));
    assert_eq!(store.values["learning_content_v1"], original);

    store.write_error = None;
    let loaded = load(&store);
    let migrated = commit_content_load_to_store(loaded, &mut store, &Cell::new(false)).unwrap();
    assert!(migrated
        .notice
        .unwrap()
        .contains("Using saved learning content"));
    let snapshot: serde_json::Value =
        serde_json::from_str(&store.values["learning_content_v1"]).unwrap();
    assert_eq!(snapshot["version"], 2);
    assert!(snapshot["value"].get("sentences").is_none());
}
