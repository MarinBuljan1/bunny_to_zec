use super::*;
use crate::model::CardList;
use indexmap::IndexMap;

fn content() -> LearningContent {
    LearningContent {
        cards: ["kavu", "kava", "molim", "čaj"]
            .into_iter()
            .map(|word| {
                (
                    word.to_owned(),
                    CardList {
                        base_language: word.into(),
                        sentence_indexes: vec![],
                    },
                )
            })
            .collect::<IndexMap<_, _>>(),
        ..Default::default()
    }
}
fn progress() -> SavedProgress {
    SavedProgress {
        scores: HashMap::new(),
        last_data_update: 1000,
        conversations: Default::default(),
    }
}
fn library() -> ConversationLibrary {
    let exchanges = |text: &str| {
        (0..5)
            .map(|_| Exchange {
                partner: Message {
                    hr: "Što želite?".into(),
                    en: "What would you like?".into(),
                },
                learner: Message {
                    hr: text.into(),
                    en: "Coffee please.".into(),
                },
            })
            .collect()
    };
    ConversationLibrary {
        schema_version: 1,
        source_language: "hr".into(),
        translation_language: "en".into(),
        conversations: vec![
            Conversation {
                id: "0001".into(),
                title: "Cafe".into(),
                exchanges: exchanges("Kavu, kavu, kavu."),
            },
            Conversation {
                id: "0002".into(),
                title: "Shop".into(),
                exchanges: exchanges("Kavu."),
            },
        ],
        word_index: HashMap::from([(
            "kavu".into(),
            vec![
                Hit {
                    c_id: "0001".into(),
                    hits: 15,
                },
                Hit {
                    c_id: "0002".into(),
                    hits: 5,
                },
            ],
        )]),
    }
}

#[test]
fn index_is_verified_against_actual_learner_messages() {
    let mut library = library();
    library.validate(&content()).unwrap();
    library.word_index.get_mut("kavu").unwrap()[0].hits += 1;
    assert!(library.validate(&content()).is_err());
}
#[test]
fn selection_weights_occurrences_only_among_least_read_conversations() {
    let library = library();
    let content = content();
    let mut progress = progress();
    assert_eq!(
        library
            .choose(&content, &progress, &[], 1000, 0.74)
            .unwrap()
            .id,
        "0001"
    );
    assert_eq!(
        library
            .choose(&content, &progress, &[], 1000, 0.75)
            .unwrap()
            .id,
        "0002"
    );
    progress.conversations.read_counts.insert("0001".into(), 1);
    assert_eq!(
        library
            .choose(&content, &progress, &[], 1000, 0.0)
            .unwrap()
            .id,
        "0002"
    );
    // Uncovered earlier-priority vocabulary cannot block the conversation library.
    progress.scores.insert(
        "čaj".into(),
        ScoreData {
            score: 0,
            last_review: 0,
        },
    );
    assert_eq!(
        library
            .choose(&content, &progress, &[], 1000, 0.0)
            .unwrap()
            .id,
        "0002"
    );
}
#[test]
fn masks_known_forms_and_one_unseen_form_without_merging_grammar_or_case() {
    let content = content();
    let mut progress = progress();
    let words = review_words("Kavu, kavu, KAVA, molim.", &content, &progress, 1000);
    assert_eq!(
        words
            .iter()
            .map(|w| w.target_language.as_str())
            .collect::<Vec<_>>(),
        ["kavu"]
    );
    progress.scores.insert(
        "kavu".into(),
        ScoreData {
            score: 863999,
            last_review: 1000,
        },
    );
    assert_eq!(
        review_words("Kavu, kava.", &content, &progress, 1000).len(),
        1
    );
    progress.scores.get_mut("kavu").unwrap().score = 864000;
    assert_eq!(
        review_words("Kavu, kava.", &content, &progress, 1000).len(),
        2
    );
}
#[test]
fn typing_does_not_count_as_recall_and_completion_is_counted_only_once() {
    let library = library();
    let content = content();
    let (mut review, _) = ConversationReview::restore(progress(), &library, &content, 1000000);
    for index in 0..5 {
        assert!(review.act(ChatAction::Next, &library, &content, 1000000, 0.0));
        assert!(!review.act(ChatAction::Flip, &library, &content, 1001000, 0.0));
        assert!(review.act(ChatAction::LearnerAppears, &library, &content, 1010000, 0.0));
        assert!(!review.act(
            ChatAction::Rate(Rating::Correct),
            &library,
            &content,
            1010100,
            0.0
        ));
        assert!(review.act(ChatAction::Flip, &library, &content, 1010500, 0.0));
        assert_eq!(
            review.session().unwrap().phase,
            SessionPhase::Answer {
                elapsed_ms: 500,
                face: AnswerFace::Revealed
            }
        );
        assert!(review.act(
            ChatAction::Rate(Rating::Correct),
            &library,
            &content,
            1011000,
            0.0
        ));
        let saved = review.progress.clone();
        assert!(!review.act(
            ChatAction::Rate(Rating::Correct),
            &library,
            &content,
            1011001,
            0.0
        ));
        assert_eq!(review.progress, saved);
        assert_eq!(
            review
                .progress
                .conversations
                .read_counts
                .get("0001")
                .copied()
                .unwrap_or(0),
            u64::from(index == 4)
        );
        assert!(review.act(ChatAction::Settled, &library, &content, 1012000, 0.0));
    }
    assert_eq!(review.progress.scores.len(), 1);
    review.act(ChatAction::Next, &library, &content, 1013000, 0.0);
    assert_eq!(review.session().unwrap().conversation_id, "0002");
    assert!(review.session().unwrap().completed.is_empty());
}
#[test]
fn resuming_question_restarts_clock_and_resuming_answer_keeps_frozen_duration() {
    let library = library();
    let content = content();
    let (mut review, _) = ConversationReview::restore(progress(), &library, &content, 1000000);
    review.act(ChatAction::Next, &library, &content, 1000000, 0.0);
    review.act(ChatAction::LearnerAppears, &library, &content, 1001000, 0.0);
    let (mut resumed, _) =
        ConversationReview::restore(review.progress, &library, &content, 2000000);
    resumed.act(ChatAction::Flip, &library, &content, 2000400, 0.0);
    let (resumed, _) = ConversationReview::restore(resumed.progress, &library, &content, 9000000);
    assert_eq!(
        resumed.session().unwrap().phase,
        SessionPhase::Answer {
            elapsed_ms: 400,
            face: AnswerFace::Revealed
        }
    );
}
#[test]
fn actual_corpus_matches_the_runtime_tokenizer_and_vocabulary() {
    let library: ConversationLibrary =
        serde_json::from_str(include_str!("../../assets/conversations.json")).unwrap();
    let cards = serde_json::from_str(include_str!("../../assets/croatian.json")).unwrap();
    library
        .validate(&LearningContent {
            cards,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(library.conversations.len(), 1000);
}

#[test]
fn conversation_progress_and_scores_commit_atomically_and_resume_without_regrading() {
    use crate::memory_store::MemoryStore;
    use crate::persistence::{read_progress, write_progress, StorageError};
    let library = library();
    let content = content();
    let (mut review, _) = ConversationReview::restore(progress(), &library, &content, 1000000);
    let mut store = MemoryStore::default();
    write_progress(&mut store, &review.progress).unwrap();
    let before = store.values.clone();
    review.act(ChatAction::Next, &library, &content, 1000000, 0.0);
    review.act(ChatAction::LearnerAppears, &library, &content, 1002000, 0.0);
    review.act(ChatAction::Flip, &library, &content, 1002500, 0.0);
    review.act(
        ChatAction::Rate(Rating::Correct),
        &library,
        &content,
        1002600,
        0.0,
    );
    review.act(ChatAction::ToggleEnglish, &library, &content, 1002600, 0.0);
    store.write_error = Some(StorageError::WriteFailed);
    assert!(write_progress(&mut store, &review.progress).is_err());
    assert_eq!(store.values, before);
    store.write_error = None;
    write_progress(&mut store, &review.progress).unwrap();
    let progress = read_progress(&store, 2000).unwrap().progress;
    assert_eq!(progress, review.progress);
    let (mut restored, notice) = ConversationReview::restore(progress, &library, &content, 2000000);
    assert!(notice.is_none());
    assert!(restored.can_next());
    assert!(!restored.act(
        ChatAction::Rate(Rating::Correct),
        &library,
        &content,
        2001000,
        0.0
    ));
    assert_eq!(restored.progress, review.progress);
}

#[test]
fn version_two_scores_migrate_without_losing_any_values() {
    use crate::memory_store::MemoryStore;
    use crate::persistence::{read_progress, write_progress};
    let mut store = MemoryStore::default();
    let raw = r#"{"version":2,"value":{"scores":{"kavu":{"score":500,"last_review":1000}},"last_data_update":1100}}"#;
    store
        .values
        .insert("learning_progress_v1".into(), raw.into());
    let progress = read_progress(&store, 1200).unwrap().progress;
    assert!(progress.conversations.session.is_none());
    assert!(progress.conversations.read_counts.is_empty());
    assert_eq!(store.values["learning_progress_v1"], raw);
    write_progress(&mut store, &progress).unwrap();
    let snapshot: serde_json::Value =
        serde_json::from_str(&store.values["learning_progress_v1"]).unwrap();
    assert_eq!(snapshot["version"], 3);
    assert_eq!(read_progress(&store, 1200).unwrap().progress, progress);
}

#[test]
fn obsolete_or_invalid_session_preserves_scores_and_read_counts() {
    let library = library();
    let content = content();
    let (mut review, _) = ConversationReview::restore(progress(), &library, &content, 1000000);
    review.act(ChatAction::Next, &library, &content, 1000000, 0.0);
    review.progress.scores.insert(
        "molim".into(),
        ScoreData {
            score: 2000,
            last_review: 1000,
        },
    );
    review
        .progress
        .conversations
        .read_counts
        .insert("0002".into(), 4);
    // A stored target cannot score an unrelated word that is absent from the reply.
    review
        .progress
        .conversations
        .session
        .as_mut()
        .unwrap()
        .words
        .push(Card {
            target_language: "molim".into(),
            base_language: "molim".into(),
        });
    let (restored, notice) =
        ConversationReview::restore(review.progress.clone(), &library, &content, 2000000);
    assert!(notice.is_some());
    assert!(restored.session().is_none());
    assert_eq!(restored.progress.scores, review.progress.scores);
    assert_eq!(
        restored.progress.conversations.read_counts,
        review.progress.conversations.read_counts
    );
}

#[test]
fn read_only_messages_advance_without_grading_and_delay_has_readable_bounds() {
    let mut library = library();
    let content = content();
    library.conversations[0].exchanges[0].learner.hr = "Dobar dan.".into();
    let (mut review, _) = ConversationReview::restore(progress(), &library, &content, 1000000);
    review.act(ChatAction::Next, &library, &content, 1000000, 0.0);
    assert!(review.session().unwrap().words.is_empty());
    review.act(ChatAction::LearnerAppears, &library, &content, 1002000, 0.0);
    assert!(review.can_next());
    assert_eq!(review.session().unwrap().completed[0].rating, Rating::Read);
    assert!(review.progress.scores.is_empty());
    assert_eq!(typing_delay_ms("Bok!"), 1200);
    assert_eq!(typing_delay_ms(&"word ".repeat(100)), 4000);
}
