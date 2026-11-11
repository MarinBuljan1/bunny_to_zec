use super::*;
use crate::memory_store::{content, MemoryStore};

fn progress() -> SavedProgress {
    SavedProgress {
        scores: HashMap::from([(
            "bok".into(),
            ScoreData {
                score: 500,
                last_review: 1_000,
            },
        )]),
        last_data_update: 1_100,
    }
}

#[test]
fn new_users_start_without_progress_or_warnings() {
    let loaded = read_progress(&MemoryStore::default(), 2_000).unwrap();
    assert!(loaded.progress.scores.is_empty());
    assert_eq!(loaded.progress.last_data_update, 2_000);
    assert!(loaded.warning.is_none());
}

#[test]
fn legacy_progress_migrates_on_one_successful_save_without_removing_old_keys() {
    let mut store = MemoryStore::default();
    let expected = progress();
    store.insert_json("card_scores", &expected.scores);
    store.insert_json("last_data_update", &expected.last_data_update);
    let legacy = store.values.clone();
    let restored = read_progress(&store, 2_000).unwrap();
    assert_eq!(restored.progress, expected);
    write_progress(&mut store, &restored.progress).unwrap();
    assert_eq!(store.writes, 1);
    assert_eq!(read_progress(&store, 3_000).unwrap().progress, expected);
    for (key, value) in legacy {
        assert_eq!(store.values[&key], value);
    }
    // Once migrated, an old/malformed legacy key cannot override newer progress.
    store.values.insert("card_scores".into(), "broken".into());
    assert_eq!(read_progress(&store, 3_000).unwrap().progress, expected);
}

#[test]
fn bad_legacy_timestamp_preserves_scores_and_resumes_decay_from_now() {
    let mut store = MemoryStore::default();
    store.insert_json("card_scores", &progress().scores);
    store
        .values
        .insert("last_data_update".into(), "broken".into());
    let loaded = read_progress(&store, 2_000).unwrap();
    assert_eq!(loaded.progress.scores, progress().scores);
    assert_eq!(loaded.progress.last_data_update, 2_000);
    assert!(loaded.warning.is_some());
    write_progress(&mut store, &loaded.progress).unwrap();
    assert!(read_progress(&store, 3_000).unwrap().warning.is_none());
}

#[test]
fn corrupt_or_future_progress_is_never_overwritten() {
    for (key, raw) in [
        (PROGRESS_KEY, "broken"),
        (PROGRESS_KEY, r#"{"version":2,"value":{}}"#),
        ("card_scores", "broken"),
    ] {
        let mut store = MemoryStore::default();
        store.values.insert(key.into(), raw.into());
        let before = store.values.clone();
        assert!(read_progress(&store, 2_000).is_err());
        assert!(write_progress(&mut store, &progress()).is_err());
        assert_eq!(store.values, before);
        assert_eq!(store.writes, 0);
    }
}

#[test]
fn failed_atomic_save_keeps_the_previous_snapshot_intact() {
    let mut store = MemoryStore::default();
    write_progress(&mut store, &progress()).unwrap();
    let before = store.values.clone();
    store.write_error = Some(StorageError::WriteFailed);
    let mut updated = progress();
    updated.scores.get_mut("bok").unwrap().score = 700;
    assert_eq!(
        write_progress(&mut store, &updated),
        Err(StorageError::WriteFailed)
    );
    assert_eq!(store.values, before);
    store.write_error = None;
    write_progress(&mut store, &updated).unwrap();
    assert_eq!(read_progress(&store, 3_000).unwrap().progress, updated);
}

#[test]
fn content_round_trips_and_legacy_content_remains_readable() {
    let expected = content();
    let mut store = MemoryStore::default();
    store.insert_json("cards", &expected.cards);
    store.insert_json("sentences", &expected.sentences);
    assert_eq!(read_content(&store).unwrap(), expected);
    write_content(&mut store, &expected).unwrap();
    assert_eq!(store.writes, 1);
    assert!(!store.values.contains_key("cards"));
    assert!(!store.values.contains_key("sentences"));
    assert_eq!(read_content(&store).unwrap(), expected);
}

#[test]
fn invalid_content_cannot_replace_a_good_cache() {
    let mut store = MemoryStore::default();
    write_content(&mut store, &content()).unwrap();
    let before = store.values.clone();
    for indices in [vec![-1], vec![1], vec![0, -1]] {
        let mut invalid = content();
        invalid.cards.get_mut("bok").unwrap().sentence_indexes = indices;
        assert!(write_content(&mut store, &invalid).is_err());
        assert_eq!(store.values, before);
    }
    assert!(validate_content(&LearningContent::default()).is_err());
    let mut word_only = content();
    word_only
        .cards
        .get_mut("bok")
        .unwrap()
        .sentence_indexes
        .clear();
    word_only.sentences.clear();
    assert!(validate_content(&word_only).is_ok());
}

#[test]
fn unavailable_storage_and_invalid_scores_return_errors_without_panicking() {
    let store = MemoryStore {
        read_error: Some(StorageError::Unavailable),
        ..MemoryStore::default()
    };
    assert!(matches!(
        read_progress(&store, 2_000),
        Err(StorageError::Unavailable)
    ));
    assert_eq!(read_content(&store), Err(StorageError::Unavailable));
    let mut invalid = progress();
    invalid.scores.get_mut("bok").unwrap().score = -1;
    assert!(write_progress(&mut MemoryStore::default(), &invalid).is_err());
}

#[test]
fn real_project_content_satisfies_validation() {
    let content = LearningContent {
        cards: serde_json::from_str(include_str!("../../assets/croatian.json")).unwrap(),
        sentences: serde_json::from_str(include_str!("../../assets/croatian_sentences.json"))
            .unwrap(),
        explanations: serde_json::from_str(include_str!("../../assets/croatian_explanations.json"))
            .unwrap(),
    };
    assert!(validate_content(&content).is_ok());
}

#[test]
fn unordered_references_and_corrupt_or_future_content_are_rejected() {
    let mut invalid = content();
    invalid.sentences.push(crate::model::CardSentences {
        base_language: "is".into(),
        target_language: "je".into(),
    });
    invalid.cards.get_mut("bok").unwrap().sentence_indexes = vec![1, 0];
    assert!(validate_content(&invalid).unwrap_err().contains("ordered"));
    for raw in ["broken", r#"{"version":2,"value":{}}"#] {
        let mut store = MemoryStore::default();
        store.values.insert(CONTENT_KEY.into(), raw.into());
        assert!(read_content(&store).is_err());
        assert_eq!(store.values[CONTENT_KEY], raw);
    }
}

#[test]
fn content_migration_only_removes_legacy_copies_after_success_and_never_touches_progress() {
    let mut store = MemoryStore::default();
    let expected = content();
    store.insert_json("cards", &expected.cards);
    store.insert_json("sentences", &expected.sentences);
    store.insert_json("explanations", &expected.explanations);
    store.insert_json("card_scores", &progress().scores);
    store.insert_json("last_data_update", &progress().last_data_update);
    let before = store.values.clone();
    store.write_error = Some(StorageError::WriteFailed);
    assert!(write_content(&mut store, &expected).is_err());
    assert_eq!(store.values, before);
    store.write_error = None;
    write_content(&mut store, &expected).unwrap();
    for key in ["cards", "sentences", "explanations"] {
        assert!(!store.values.contains_key(key));
    }
    for key in ["card_scores", "last_data_update"] {
        assert_eq!(store.values[key], before[key]);
    }
    assert_eq!(read_content(&store).unwrap(), expected);
}

#[test]
fn failed_legacy_cleanup_reports_a_warning_but_keeps_the_committed_snapshot_usable() {
    let mut store = MemoryStore {
        remove_error: Some(StorageError::WriteFailed),
        ..MemoryStore::default()
    };
    assert!(write_content(&mut store, &content()).unwrap().is_some());
    assert_eq!(read_content(&store).unwrap(), content());
}
