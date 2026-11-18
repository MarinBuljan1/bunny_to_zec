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
        conversations: Default::default(),
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
fn spelling_corrections_keep_the_latest_review_without_inflating_scores() {
    let mut store = MemoryStore::default();
    let scores = HashMap::from([
        (
            "covek".to_owned(),
            ScoreData {
                score: 200,
                last_review: 1100,
            },
        ),
        (
            "čovjek".to_owned(),
            ScoreData {
                score: 900,
                last_review: 1000,
            },
        ),
        (
            "pomoči".to_owned(),
            ScoreData {
                score: 300,
                last_review: 1200,
            },
        ),
        (
            "pomoći".to_owned(),
            ScoreData {
                score: 500,
                last_review: 1200,
            },
        ),
        (
            "o.k.".to_owned(),
            ScoreData {
                score: 700,
                last_review: 1000,
            },
        ),
    ]);
    store.insert_json("card_scores", &scores);
    store.insert_json("last_data_update", &1300);
    let before = store.values.clone();
    let restored = read_progress(&store, 1400).unwrap().progress;
    assert_eq!(restored.scores.len(), 3);
    assert_eq!(restored.scores["čovjek"], scores["covek"]);
    assert_eq!(restored.scores["pomoći"], scores["pomoći"]);
    assert_eq!(restored.scores["okej"], scores["o.k."]);
    assert_eq!(restored.last_data_update, 1300);
    assert_eq!(store.values, before); // Reading does not rewrite the stored data.
    write_progress(&mut store, &restored).unwrap();
    assert_eq!(read_progress(&store, 1500).unwrap().progress, restored);
    assert_eq!(store.values["card_scores"], before["card_scores"]);
}

#[test]
fn spelling_corrections_also_apply_to_snapshots_and_stale_saves() {
    let original = SavedProgress {
        scores: HashMap::from([(
            "o.k.".to_owned(),
            ScoreData {
                score: 123,
                last_review: 1000,
            },
        )]),
        last_data_update: 1100,
        conversations: Default::default(),
    };
    let mut store = MemoryStore::default();
    store.insert_json(
        PROGRESS_KEY,
        &Snapshot {
            version: 2,
            value: &original,
        },
    );
    let restored = read_progress(&store, 1200).unwrap().progress;
    assert_eq!(restored.scores["okej"], original.scores["o.k."]);
    write_progress(&mut store, &original).unwrap();
    let saved: Snapshot<SavedProgress> = serde_json::from_str(&store.values[PROGRESS_KEY]).unwrap();
    assert!(!saved.value.scores.contains_key("o.k."));
    assert_eq!(saved.value, restored);
}

#[test]
fn every_reviewed_alias_migrates_to_a_current_word_without_changing_credit() {
    let cards: HashMap<String, serde_json::Value> =
        serde_json::from_str(include_str!("../../assets/croatian.json")).unwrap();
    for correction in &VOCABULARY_CORRECTIONS.corrections {
        assert!(!cards.contains_key(&correction.original));
        assert!(cards.contains_key(&correction.corrected));
        let mut progress = progress();
        let reviewed = ScoreData {
            score: 456,
            last_review: 1234,
        };
        progress
            .scores
            .insert(correction.original.clone(), reviewed.clone());
        correct_progress_spellings(&mut progress);
        assert!(!progress.scores.contains_key(&correction.original));
        assert_eq!(progress.scores[&correction.corrected], reviewed);
    }
}

#[test]
fn valid_homographs_and_split_phrase_history_are_not_reassigned() {
    let mut progress = progress();
    let original_words = ["sef", "vise", "biću", "biće", "bille", "nemogu", "samnom"];
    for word in original_words {
        progress.scores.insert(
            word.into(),
            ScoreData {
                score: 456,
                last_review: 1234,
            },
        );
    }
    let expected = progress.clone();
    correct_progress_spellings(&mut progress);
    assert_eq!(progress, expected);
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
        (PROGRESS_KEY, r#"{"version":5,"value":{}}"#),
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
fn version_one_progress_migrates_atomically_and_old_readers_reject_version_four() {
    let mut store = MemoryStore::default();
    // Literal old-format JSON is independent of the widened Rust model.
    let original = r#"{"version":1,"value":{"scores":{"bok":{"score":2147483647,"last_review":1700000000}},"last_data_update":1700000001}}"#;
    store.values.insert(PROGRESS_KEY.into(), original.into());
    let mut restored = read_progress(&store, 1_700_000_100).unwrap().progress;
    assert_eq!(restored.scores["bok"].score, i64::from(i32::MAX));
    assert_eq!(store.values[PROGRESS_KEY], original); // Reading alone does not migrate.
    restored.scores.get_mut("bok").unwrap().score += 1;
    restored.last_data_update = 2_200_000_000;
    store.write_error = Some(StorageError::WriteFailed);
    assert_eq!(
        write_progress(&mut store, &restored),
        Err(StorageError::WriteFailed)
    );
    assert_eq!(store.values[PROGRESS_KEY], original);
    store.write_error = None;
    write_progress(&mut store, &restored).unwrap();
    let envelope: serde_json::Value = serde_json::from_str(&store.values[PROGRESS_KEY]).unwrap();
    assert_eq!(envelope["version"], 4);
    assert_eq!(
        read_progress(&store, 2_200_000_001).unwrap().progress,
        restored
    );
    // The previous reader checked its supported version before decoding/writing.
    assert!(matches!(
        read_snapshot::<SavedProgress>(&store, PROGRESS_KEY, &[1]),
        Err(StorageError::UnsupportedVersion(PROGRESS_KEY))
    ));
}

#[test]
fn full_i64_values_round_trip_exactly_without_javascript_number_conversion() {
    let mut store = MemoryStore::default();
    let expected = SavedProgress {
        scores: HashMap::from([(
            "bok".into(),
            ScoreData {
                score: i64::MAX,
                last_review: i64::MAX - 1,
            },
        )]),
        last_data_update: i64::MAX,
        conversations: Default::default(),
    };
    write_progress(&mut store, &expected).unwrap();
    assert_eq!(read_progress(&store, i64::MAX).unwrap().progress, expected);
    assert!(store.values[PROGRESS_KEY].contains("9223372036854775807"));
}

#[test]
fn widened_progress_rejects_negative_and_out_of_range_values_without_overwriting() {
    for value in ["-1", "9223372036854775808"] {
        for field in ["score", "last_review", "last_data_update"] {
            let mut store = MemoryStore::default();
            let raw = r#"{"version":2,"value":{"scores":{"bok":{"score":500,"last_review":1000}},"last_data_update":1100}}"#;
            let valid = match field {
                "score" => "\"score\":500",
                "last_review" => "\"last_review\":1000",
                _ => "\"last_data_update\":1100",
            };
            let invalid = raw.replace(valid, &format!("\"{field}\":{value}"));
            store.values.insert(PROGRESS_KEY.into(), invalid.clone());
            assert!(read_progress(&store, 2_000).is_err());
            assert!(write_progress(&mut store, &progress()).is_err());
            assert_eq!(store.values[PROGRESS_KEY], invalid);
            assert_eq!(store.writes, 0);
        }
    }
}

#[test]
fn historical_content_snapshots_and_legacy_keys_remain_readable() {
    let expected = content();
    let mut store = MemoryStore::default();
    store.insert_json("cards", &expected.cards);
    store.insert_json(
        "sentences",
        &serde_json::json!([{"base_language": "hello!", "target_language": "bok!"}]),
    );
    assert_eq!(read_content(&store).unwrap(), expected);
    store.cache_content(&expected);
    assert_eq!(read_content(&store).unwrap(), expected);
    assert_eq!(store.writes, 0);
}

#[test]
fn empty_vocabulary_and_blank_forms_are_rejected() {
    let mut empty = content();
    empty.cards.clear();
    assert!(validate_content(&empty).is_err());
    let mut invalid = content();
    let entry = invalid.cards.shift_remove("bok").unwrap();
    invalid.cards.insert(" ".into(), entry);
    assert!(validate_content(&invalid).is_err());
    assert!(validate_content(&LearningContent::default()).is_err());
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
        explanations: serde_json::from_str(include_str!("../../assets/croatian_explanations.json"))
            .unwrap(),
    };
    assert!(validate_content(&content).is_ok());
}

#[test]
fn corrupt_or_future_content_is_rejected() {
    for raw in [
        "broken",
        r#"{"version":2,"value":{}}"#,
        r#"{"version":3,"value":{}}"#,
    ] {
        let mut store = MemoryStore::default();
        store.values.insert(CONTENT_KEY.into(), raw.into());
        assert!(read_content(&store).is_err());
        assert_eq!(store.values[CONTENT_KEY], raw);
    }
}

#[test]
fn obsolete_content_cleanup_never_touches_progress() {
    let mut store = MemoryStore::default();
    let expected = content();
    store.insert_json("cards", &expected.cards);
    store.insert_json(
        "sentences",
        &serde_json::json!([{"base_language": "hello!", "target_language": "bok!"}]),
    );
    store.insert_json("explanations", &expected.explanations);
    store.cache_content(&expected);
    store.insert_json("card_scores", &progress().scores);
    store.insert_json("last_data_update", &progress().last_data_update);
    let before = store.values.clone();
    // Quota failure on set_item must not prevent removal of obsolete content.
    store.write_error = Some(StorageError::WriteFailed);
    remove_legacy_content(&mut store).unwrap();
    for key in [CONTENT_KEY, "cards", "sentences", "explanations"] {
        assert!(!store.values.contains_key(key));
    }
    for key in ["card_scores", "last_data_update"] {
        assert_eq!(store.values[key], before[key]);
    }
    assert_eq!(store.writes, 0);
}

#[test]
fn failed_legacy_cleanup_returns_an_error_without_writing_a_snapshot() {
    let mut store = MemoryStore {
        remove_error: Some(StorageError::WriteFailed),
        ..MemoryStore::default()
    };
    store.cache_content(&content());
    assert_eq!(
        remove_legacy_content(&mut store),
        Err(StorageError::WriteFailed)
    );
    assert_eq!(read_content(&store).unwrap(), content());
    assert_eq!(store.writes, 0);
}

#[test]
fn version_one_content_reads_without_sentence_data_or_progress_changes() {
    let original = r#"{"version":1,"value":{"cards":{"bok":{"base_language":"hello","sentence_indexes":[0]}},"sentences":[{"base_language":"hello!","target_language":"bok!"}],"explanations":{}}}"#;
    let mut store = MemoryStore::default();
    store.values.insert(CONTENT_KEY.into(), original.into());
    let mut saved = progress();
    saved.conversations.read_counts.insert("0001".into(), 3);
    saved.conversations.show_english = true;
    write_progress(&mut store, &saved).unwrap();
    let previous_progress = store.values[PROGRESS_KEY].clone();
    let restored = read_content(&store).unwrap();
    assert_eq!(restored, content());
    assert_eq!(store.values[CONTENT_KEY], original);
    assert_eq!(store.values[PROGRESS_KEY], previous_progress);
    assert_eq!(read_content(&store).unwrap(), restored);
    assert_eq!(store.values[CONTENT_KEY], original);
}

#[test]
fn legacy_vocabulary_read_preserves_order_and_needs_no_sentence_cache() {
    let mut store = MemoryStore::default();
    store.values.insert("cards".into(), r#"{"second":{"base_language":"two","sentence_indexes":[999]},"first":{"base_language":"one","sentence_indexes":[]}}"#.into());
    store
        .values
        .insert("sentences".into(), "broken obsolete cache".into());
    let restored = read_content(&store).unwrap();
    assert_eq!(
        restored
            .cards
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["second", "first"]
    );
    assert!(store.values.contains_key("sentences"));
    assert_eq!(read_content(&store).unwrap(), restored);
}
