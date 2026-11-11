use super::*;
use crate::model::{CardList, CardSentences};
use indexmap::indexmap;

fn finish_step(state: Rc<ReviewState>, at_ms: i64) -> Rc<ReviewState> {
    let step = state.animation.step().unwrap();
    state.reduce(ReviewAction::AnimationElapsed {
        generation: step.generation,
        at_ms,
    })
}

#[test]
fn complete_review_saves_and_reloads_the_accepted_result_once() {
    use crate::persistence::{read_progress, write_progress};
    for now_s in [1_700_000_000, 2_200_000_000] {
        let content = crate::memory_store::content();
        let mut store = crate::memory_store::MemoryStore::default();
        let now_ms = now_s * 1_000;
        let progress = read_progress(&store, now_s).unwrap().progress;
        let presentation =
            prepare_next_review(&content, &progress.scores, &[], now_s, 0.0).unwrap();
        let state = Rc::new(ReviewState::welcome(now_ms)).reduce(ReviewAction::Swipe {
            direction: SwipeDirection::Up,
            at_ms: now_ms,
        });
        // Starting the app cannot score the welcome screen.
        assert!(
            calculate_review_result(&progress, &state, &state.animation.step().unwrap()).is_none()
        );
        let state = finish_step(state, now_ms + 400);
        let generation = state.animation.step().unwrap().generation;
        let state = state.reduce(ReviewAction::PresentNext {
            generation,
            presentation,
        });
        let state = finish_step(finish_step(state, now_ms + 450), now_ms + 550);
        let state = state.reduce(ReviewAction::Swipe {
            direction: SwipeDirection::Left,
            at_ms: now_ms + 2_050,
        });
        let reveal = state.animation.step().unwrap();
        assert!(calculate_review_result(&progress, &state, &reveal).is_none());
        // Rate before the explanation timer finishes; duration was already frozen.
        let state = state.reduce(ReviewAction::Swipe {
            direction: SwipeDirection::Up,
            at_ms: now_ms + 2_100,
        });
        let accepted = state.clone();
        let step = state.animation.step().unwrap();
        let state = state.reduce(ReviewAction::Swipe {
            direction: SwipeDirection::Down,
            at_ms: now_ms + 9_000,
        });
        let state = state.reduce(ReviewAction::AnimationElapsed {
            generation: reveal.generation,
            at_ms: now_ms + 9_100,
        });
        assert!(Rc::ptr_eq(&state, &accepted));
        let updated = calculate_review_result(&progress, &state, &step).unwrap();
        // "hello!bok!" is 10 UTF-8 bytes: divisor = 1.5/1.5 - 10*0.02 = 0.8.
        assert_eq!(updated.scores["bok"].score, 1_080_000);
        assert_eq!(updated.scores["bok"].last_review, now_s + 2);
        assert_eq!(updated.last_data_update, now_s + 2);
        write_progress(&mut store, &updated).unwrap();
        assert_eq!(store.writes, 1);
        assert_eq!(read_progress(&store, now_s + 60).unwrap().progress, updated);
        let state = finish_step(state, now_ms + 2_500);
        assert!(
            calculate_review_result(&updated, &state, &state.animation.step().unwrap()).is_none()
        );
    }
}

#[test]
fn prepared_review_keeps_the_selected_sentence_and_words_together() {
    let content = LearningContent {
        cards: indexmap! {
            "bok".into() => CardList { base_language: "hello".into(), sentence_indexes: vec![1] },
        },
        sentences: vec![
            CardSentences {
                base_language: "unused".into(),
                target_language: "unused".into(),
            },
            CardSentences {
                base_language: "hello!".into(),
                target_language: "bok!".into(),
            },
        ],
        explanations: HashMap::new(),
    };
    // Index 1 uses the existing first-index fallback, so no browser RNG is needed.
    let next = prepare_next_review(&content, &HashMap::new(), &[], 1_700_000_000, 0.5)
        .expect("the single available vocabulary entry must produce a review");
    assert_eq!(next.card.target_language, "bok!");
    assert_eq!(next.card.base_language, "hello!");
    assert_eq!(
        next.review_words,
        vec![Card {
            base_language: "hello".into(),
            target_language: "bok".into(),
        }]
    );
}

#[test]
fn no_sentence_fallback_preserves_previous_review_words() {
    let content = LearningContent {
        cards: indexmap! {
            "bok".into() => CardList { base_language: "hello".into(), sentence_indexes: vec![] },
        },
        ..LearningContent::default()
    };
    let previous = vec![Card {
        base_language: "is".into(),
        target_language: "je".into(),
    }];
    let next = prepare_next_review(&content, &HashMap::new(), &previous, 1_700_000_000, 0.5)
        .expect("an available vocabulary entry without sentences uses the existing fallback");
    assert_eq!(next.card.target_language, "bok");
    assert_eq!(next.card.base_language, "hello");
    assert_eq!(next.review_words, previous);
}
