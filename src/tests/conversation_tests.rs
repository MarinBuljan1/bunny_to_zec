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
                    hr: "Kavu?".into(),
                    en: "What would you like?".into(),
                },
                learner: Message {
                    hr: text.into(),
                    en: "Coffee please.".into(),
                },
            })
            .collect()
    };
    let mut library = ConversationLibrary {
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
        difficulties: HashMap::new(),
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
    };
    library.validate_and_rank(&content()).unwrap();
    library
}

#[test]
fn index_is_verified_against_actual_learner_messages() {
    let mut library = library();
    library.validate_and_rank(&content()).unwrap();
    library.word_index.get_mut("kavu").unwrap()[0].hits += 1;
    assert!(library.validate_and_rank(&content()).is_err());
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
        assert!(review.act(ChatAction::PartnerAppears, &library, &content, 1010000, 0.0));
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
    review.act(ChatAction::PartnerAppears, &library, &content, 1001000, 0.0);
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
    let mut library: ConversationLibrary =
        serde_json::from_str(include_str!("../../assets/conversations.json")).unwrap();
    let cards = serde_json::from_str(include_str!("../../assets/croatian.json")).unwrap();
    library
        .validate_and_rank(&LearningContent {
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
    review.act(ChatAction::PartnerAppears, &library, &content, 1002000, 0.0);
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
    assert_eq!(snapshot["version"], 4);
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
    review.act(ChatAction::PartnerAppears, &library, &content, 1002000, 0.0);
    review.act(ChatAction::LearnerAppears, &library, &content, 1002000, 0.0);
    assert!(review.can_next());
    assert_eq!(review.session().unwrap().completed[0].rating, Rating::Read);
    assert!(review.progress.scores.is_empty());
    assert_eq!(typing_delay_ms("Bok!"), 1200);
    assert_eq!(typing_delay_ms(&"word ".repeat(100)), 4000);
}

#[test]
fn difficulty_counts_both_speakers_repetitions_case_and_unknown_words() {
    let content = content();
    let mut conversation = library().conversations.remove(0);
    for exchange in &mut conversation.exchanges {
        exchange.partner.hr = "KAVU, čaj!".into(); // ranks 1 and 4
        exchange.learner.hr = "Kava, molim, unknown, kavu.".into(); // 2, 3, 4, 1
        exchange.partner.en = "English never affects the rank.".into();
    }
    assert_eq!(conversation_difficulty(&conversation, &content), 15.0 / 6.0);
    for exchange in &mut conversation.exchanges {
        exchange.partner.hr = "...".into();
        exchange.learner.hr = "???".into();
    }
    assert_eq!(conversation_difficulty(&conversation, &content), 4.0);
}

#[test]
fn easier_conversations_get_more_weight_but_read_count_is_always_the_first_filter() {
    let content = content();
    let mut library = library();
    let mut progress = progress();
    for exchange in &mut library.conversations[0].exchanges {
        exchange.learner.hr = "Kavu.".into();
    }
    for exchange in &mut library.conversations[1].exchanges {
        exchange.partner.hr = "Čaj?".into();
    }
    library.word_index.get_mut("kavu").unwrap()[0].hits = 5;
    library.validate_and_rank(&content).unwrap();
    assert_eq!(library.difficulties["0001"], 1.0);
    assert_eq!(library.difficulties["0002"], 2.5);
    // Equal occurrence counts: 5/1 vs 5/2.5 gives the easy conversation 5/7 probability.
    assert_eq!(
        library
            .choose(&content, &progress, &[], 1000, 0.70)
            .unwrap()
            .id,
        "0001"
    );
    assert_eq!(
        library
            .choose(&content, &progress, &[], 1000, 0.72)
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
}

#[test]
fn both_typing_stages_are_ordered_resumable_and_excluded_from_recall() {
    let library = library();
    let content = content();
    let (mut review, _) = ConversationReview::restore(progress(), &library, &content, 1000000);
    review.act(ChatAction::Next, &library, &content, 1000000, 0.0);
    assert_eq!(review.session().unwrap().phase, SessionPhase::PartnerTyping);
    assert!(!review.act(ChatAction::LearnerAppears, &library, &content, 1000500, 0.0));
    assert!(!review.can_flip());
    let (mut review, _) = ConversationReview::restore(review.progress, &library, &content, 2000000);
    assert!(review.act(ChatAction::PartnerAppears, &library, &content, 2001000, 0.0));
    assert_eq!(review.session().unwrap().phase, SessionPhase::Typing);
    let (mut review, _) = ConversationReview::restore(review.progress, &library, &content, 3000000);
    assert!(!review.act(ChatAction::PartnerAppears, &library, &content, 3001000, 0.0));
    review.act(ChatAction::LearnerAppears, &library, &content, 3004000, 0.0);
    assert!(!review.act(ChatAction::LearnerAppears, &library, &content, 3004200, 0.0));
    review.act(ChatAction::Flip, &library, &content, 3004500, 0.0);
    assert_eq!(
        review.session().unwrap().phase,
        SessionPhase::Answer {
            elapsed_ms: 500,
            face: AnswerFace::Revealed
        }
    );
}

#[test]
fn extra_flips_show_notes_without_restarting_recall_or_regrading_a_completed_reply() {
    let library = library();
    let content = content();
    let (mut review, _) = ConversationReview::restore(progress(), &library, &content, 1000000);
    for (action, time) in [
        (ChatAction::Next, 1000000),
        (ChatAction::PartnerAppears, 1001000),
        (ChatAction::LearnerAppears, 1003000),
        (ChatAction::Flip, 1003500),
    ] {
        assert!(review.act(action, &library, &content, time, 0.0));
    }
    assert!(!review.showing_notes());
    let revealed_progress = review.progress.clone();
    review.act(ChatAction::Flip, &library, &content, 1004000, 0.0);
    assert!(review.showing_notes());
    assert_eq!(review.progress, revealed_progress);
    assert_eq!(
        review.session().unwrap().phase,
        SessionPhase::Answer {
            elapsed_ms: 500,
            face: AnswerFace::Revealed
        }
    );
    review.act(
        ChatAction::Rate(Rating::Correct),
        &library,
        &content,
        1004500,
        0.0,
    );
    assert!(!review.showing_notes());
    review.act(ChatAction::Settled, &library, &content, 1004800, 0.0);
    let saved = review.progress.clone();
    assert!(review.can_flip());
    review.act(ChatAction::Flip, &library, &content, 1005000, 0.0);
    assert!(review.showing_notes());
    review.act(ChatAction::Flip, &library, &content, 1005500, 0.0);
    assert!(!review.showing_notes());
    assert_eq!(review.progress, saved);
}

#[test]
fn legacy_saved_notes_face_can_close_without_losing_the_frozen_duration() {
    let library = library();
    let content = content();
    let (mut review, _) = ConversationReview::restore(progress(), &library, &content, 1000000);
    for (action, time) in [
        (ChatAction::Next, 1000000),
        (ChatAction::PartnerAppears, 1001000),
        (ChatAction::LearnerAppears, 1002000),
        (ChatAction::Flip, 1002500),
    ] {
        review.act(action, &library, &content, time, 0.0);
    }
    review
        .progress
        .conversations
        .session
        .as_mut()
        .unwrap()
        .phase = SessionPhase::Answer {
        elapsed_ms: 500,
        face: AnswerFace::Notes,
    };
    let (mut restored, notice) =
        ConversationReview::restore(review.progress, &library, &content, 2000000);
    assert!(notice.is_none());
    assert!(restored.showing_notes());
    restored.act(ChatAction::Flip, &library, &content, 2001000, 0.0);
    assert!(!restored.showing_notes());
    assert_eq!(
        restored.session().unwrap().phase,
        SessionPhase::Answer {
            elapsed_ms: 500,
            face: AnswerFace::Revealed
        }
    );
    let saved = restored.progress.clone();
    restored.act(ChatAction::Flip, &library, &content, 2002000, 0.0);
    assert!(restored.showing_notes());
    assert_eq!(restored.progress, saved);
}

#[test]
fn version_three_in_progress_typing_session_is_kept_and_migrates_on_save() {
    use crate::memory_store::MemoryStore;
    use crate::persistence::{read_progress, write_progress};
    let library = library();
    let content = content();
    let raw = r#"{"version":3,"value":{"scores":{"kavu":{"score":500,"last_review":1000}},"last_data_update":1100,"conversations":{"read_counts":{"0002":3},"show_english":true,"session":{"conversation_id":"0001","exchange_index":0,"learner_text":"Kavu, kavu, kavu.","phase":"Typing","words":[{"target_language":"kavu","base_language":"kavu"}],"completed":[]}}}}"#;
    let mut store = MemoryStore::default();
    store
        .values
        .insert("learning_progress_v1".into(), raw.into());
    let saved = read_progress(&store, 1200).unwrap().progress;
    let (review, notice) = ConversationReview::restore(saved.clone(), &library, &content, 1200000);
    assert!(notice.is_none());
    assert_eq!(review.progress, saved);
    assert_eq!(review.session().unwrap().phase, SessionPhase::Typing);
    assert_eq!(store.values["learning_progress_v1"], raw);
    write_progress(&mut store, &review.progress).unwrap();
    let snapshot: serde_json::Value =
        serde_json::from_str(&store.values["learning_progress_v1"]).unwrap();
    assert_eq!(snapshot["version"], 4);
    assert_eq!(read_progress(&store, 1300).unwrap().progress, saved);
}
