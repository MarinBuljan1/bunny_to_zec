use super::*;
use indexmap::indexmap;

#[test]
fn test_find_related_cards() {
    let updated_scores: HashMap<String, ScoreData> = vec![
        (
            "hola".to_string(),
            ScoreData {
                score: 5,
                last_review: 0,
            },
        ),
        (
            "mundo".to_string(),
            ScoreData {
                score: 3,
                last_review: 0,
            },
        ),
    ]
    .into_iter()
    .collect();

    let words = vec!["hola".to_string(), "óxido".to_string(), "mundo".to_string()];

    let cards = indexmap! {
        "hola".to_string() => CardList { base_language: "hello".to_string(), sentence_indexes: vec![0], },
        "mundo".to_string() => CardList { base_language: "world".to_string(), sentence_indexes: vec![0], },
        "óxido".to_string() => CardList { base_language: "rust".to_string(), sentence_indexes: vec![0], },
    };

    let current_card = "mundo".to_string();

    let (related_cards, processed_sentence_data) =
        find_related_cards(&updated_scores, &cards, words, &current_card);

    // Expected related cards (must match words in `cards`)
    let expected_related_cards = vec![
        Card {
            base_language: "hello".to_string(),
            target_language: "hola".to_string(),
        },
        Card {
            base_language: "world".to_string(),
            target_language: "mundo".to_string(),
        },
    ];

    // Expected processed sentence data
    let expected_processed_sentence_data = vec![
        ProcessingData {
            word: "hola".to_string(),
            found: true,
            translation: "hello".to_string(),
        },
        ProcessingData {
            word: "óxido".to_string(),
            found: false,
            translation: "".to_string(),
        },
        ProcessingData {
            word: "mundo".to_string(),
            found: true,
            translation: "world".to_string(),
        },
    ];

    // Check related cards match expected results
    assert_eq!(
        related_cards, expected_related_cards,
        "Related cards do not match expected values."
    );

    // Check processed sentence data
    assert_eq!(
        processed_sentence_data, expected_processed_sentence_data,
        "Processed sentence data is incorrect."
    );
}

#[test]
fn test_split_text() {
    let examples_split_text = vec![
        (
            "Hello, world! Rust is great.",
            vec![
                "Hello", ",", " ", "world", "!", " ", "Rust", " ", "is", " ", "great", ".",
            ],
        ),
        (
            "Hey! How's it going?",
            vec!["Hey", "!", " ", "How's", " ", "it", " ", "going", "?"],
        ),
        ("No punctuation", vec!["No", " ", "punctuation"]),
        ("A, B, C!", vec!["A", ",", " ", "B", ",", " ", "C", "!"]),
        (
            "  Spacing  Test  ",
            vec![" ", " ", "Spacing", " ", " ", "Test", " ", " "],
        ),
    ];

    for (input, expected) in examples_split_text {
        let result = split_text(input);
        assert_eq!(result, expected, "Failed on input: '{input}'");
    }
}

#[test]
fn sentence_selection_preserves_empty_fallback_prefix_and_random_boundaries() {
    assert_eq!(choose_sentence_index(&[], 5, 0.5), None);
    assert_eq!(choose_sentence_index(&[5, 10], 0, 0.5), Some(5));
    assert_eq!(choose_sentence_index(&[1, 2, 3], 4, 0.0), Some(1));
    assert_eq!(choose_sentence_index(&[1, 2, 3], 4, 0.999), Some(3));
    // Eligibility stops at the first out-of-range entry, rather than filtering the whole list.
    assert_eq!(choose_sentence_index(&[1, 10, 2], 5, 0.999), Some(1));
}

#[test]
fn repeated_words_only_hide_the_first_eligible_occurrence() {
    let cards = indexmap! {
        "bok".into() => CardList { base_language: "hello".into(), sentence_indexes: vec![] },
    };
    let (related, tokens) = find_related_cards(
        &HashMap::new(),
        &cards,
        vec!["bok".into(), " ".into(), "bok".into()],
        "bok",
    );
    assert_eq!(related.len(), 1);
    assert!(tokens[0].found);
    assert!(!tokens[2].found);
}
