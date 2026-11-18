use super::*;
use crate::model::CardList;

#[test]
fn repeated_forms_and_shared_translations_produce_one_entry_per_form() {
    let mut content = LearningContent::default();
    for form in ["je", "jeste", "jel"] {
        content.cards.insert(
            form.into(),
            CardList {
                base_language: "is".into(),
            },
        );
    }
    content.explanations.insert(
        "is".into(),
        HashMap::from([
            ("je".into(), "Standard form.".into()),
            ("jeste".into(), "Emphatic form.".into()),
            ("jel".into(), "Question form.".into()),
        ]),
    );
    let word = Card {
        target_language: "je".into(),
        base_language: "is".into(),
    };
    let scores = HashMap::from([(
        "jeste".into(),
        ScoreData {
            score: 1,
            last_review: 1,
        },
    )]);
    let notes = word_notes(&[word.clone(), word], &content, &scores);
    assert_eq!(
        notes
            .iter()
            .map(|note| note.form.as_str())
            .collect::<Vec<_>>(),
        ["je", "jeste"]
    );
    assert_eq!(notes[0].explanation.as_deref(), Some("Standard form."));
    assert_eq!(notes[1].translation, "is");
}

#[test]
fn a_form_without_grammar_notes_still_has_one_translation_entry() {
    let notes = word_notes(
        &[Card {
            target_language: "bok".into(),
            base_language: "hello".into(),
        }],
        &LearningContent::default(),
        &HashMap::new(),
    );
    assert_eq!(
        notes,
        vec![WordNote {
            form: "bok".into(),
            translation: "hello".into(),
            explanation: None
        }]
    );
}
