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
        DownloadError::InvalidJson("sentences"),
    ] {
        let loaded = choose_content(Err(error), &store).unwrap();
        assert_eq!(loaded.content, content());
        assert!(loaded.notice.is_some());
        assert!(matches!(loaded.origin, ContentOrigin::Saved));
    }
    assert_eq!(store.values, before);
}

#[test]
fn invalid_downloads_do_not_replace_valid_cached_content() {
    let mut store = MemoryStore::default();
    persistence::write_content(&mut store, &content()).unwrap();
    let before = store.values.clone();
    let mut invalid = content();
    invalid.cards.get_mut("bok").unwrap().sentence_indexes = vec![999];
    let loaded = choose_content(Ok(invalid), &store).unwrap();
    let state = commit_content_load_to_store(Ok(loaded), &mut store, &Cell::new(false)).unwrap();
    assert!(matches!(state, ContentLoadState::Ready(_)));
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
    let state =
        commit_content_load_to_store(Err(message), &mut MemoryStore::default(), &Cell::new(false))
            .unwrap();
    assert!(matches!(state, ContentLoadState::Failed(_)));
}

#[test]
fn cache_write_failure_keeps_downloaded_content_usable_with_a_warning() {
    let mut store = MemoryStore {
        write_error: Some(StorageError::WriteFailed),
        ..MemoryStore::default()
    };
    let loaded = choose_content(Ok(content()), &store).unwrap();
    let state = commit_content_load_to_store(Ok(loaded), &mut store, &Cell::new(false)).unwrap();
    let ContentLoadState::Ready(loaded) = state else {
        panic!("valid content should remain usable");
    };
    assert_eq!(loaded.content, content());
    assert!(loaded.notice.unwrap().contains("keep learning"));
    assert_eq!(store.writes, 0);
}

#[test]
fn successful_retry_can_replace_a_failed_state() {
    let mut store = MemoryStore::default();
    let result = choose_content(Ok(content()), &store);
    assert!(matches!(
        commit_content_load_to_store(result, &mut store, &Cell::new(false)),
        Some(ContentLoadState::Ready(_))
    ));
    assert_eq!(persistence::read_content(&store).unwrap(), content());
}

#[test]
fn cancelled_or_superseded_loads_neither_commit_caches_nor_return_ui_updates() {
    let mut store = MemoryStore::default();
    let loaded = choose_content(Ok(content()), &store).unwrap();
    assert!(commit_content_load_to_store(Ok(loaded), &mut store, &Cell::new(true)).is_none());
    assert!(store.values.is_empty());
    assert_eq!(store.writes, 0);
}
