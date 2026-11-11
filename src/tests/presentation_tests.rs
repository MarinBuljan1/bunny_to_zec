use super::*;
use crate::model::{CardList, CardSentences};
use indexmap::indexmap;

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
