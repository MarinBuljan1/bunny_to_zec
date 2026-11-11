use super::*;
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
    let next = prepare_next_review(
        &content,
        &HashMap::new(),
        &CurrentTranslations { list: vec![] },
    )
    .expect("the single available vocabulary entry must produce a review");
    assert_eq!(next.card.target_language, "bok!");
    assert_eq!(next.card.base_language, "hello!");
    assert_eq!(
        next.translations.list,
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
    let previous = CurrentTranslations {
        list: vec![Card {
            base_language: "is".into(),
            target_language: "je".into(),
        }],
    };
    let next = prepare_next_review(&content, &HashMap::new(), &previous)
        .expect("an available vocabulary entry without sentences uses the existing fallback");
    assert_eq!(next.card.target_language, "bok");
    assert_eq!(next.card.base_language, "hello");
    assert_eq!(next.translations, previous);
}

#[test]
fn derived_progress_caps_each_word_and_accumulates_beyond_i32() {
    let scores = (0..3_000)
        .map(|index| {
            (
                index.to_string(),
                ScoreData {
                    score: calculate_seconds(20, 0, 0, 0),
                    last_review: 0,
                },
            )
        })
        .collect();
    assert_eq!(
        calculate_progress_sum(&scores),
        3_000 * i64::from(calculate_seconds(10, 0, 0, 0))
    );
    assert_eq!(calculate_progress_sum(&HashMap::new()), 0);
}

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
        assert_eq!(result, expected, "Failed on input: '{}'", input);
    }
}

#[test]
fn test_get_inverted_normalised_score() {
    let test_cases = vec![
        // (Input Values, Expected Output)
        (
            vec![(1, 0), (2, 0), (3, 0), (4, 0)],
            vec![0.48, 0.24, 0.16, 0.12], // Expected normalized values
        ),
        (
            vec![(10, 0), (20, 0), (30, 0), (40, 0)],
            vec![0.48, 0.24, 0.16, 0.12], // Expected normalized values
        ),
        (
            vec![(2, 0), (5, 0), (10, 0), (20, 0)],
            vec![0.43573, 0.32244, 0.16122, 0.08061], // Expected normalized values
        ),
    ];

    for (input_values, expected_output) in test_cases {
        let result = get_inverted_normalised_score(input_values.clone());

        // Ensure result length matches expected length
        assert_eq!(
            result.len(),
            expected_output.len(),
            "Result length mismatch for input: {:?}",
            input_values
        );

        // Ensure each calculated value is within a small margin of error (floating point comparison)
        for (res, exp) in result.iter().zip(expected_output.iter()) {
            assert!(
                (res - exp).abs() < 1e-6,
                "Expected {}, got {} for input {:?}",
                exp,
                res,
                input_values
            );
        }
    }
}

#[test]
fn test_calculate_raw_score() {
    let test_cases = vec![
        // (score, last_review, ms_for_review, character_length, correct, inverted_score, expected_score, expected_last_review)
        (
            100,
            1_700_000_000 - 1000,
            5000,
            20,
            true,
            0.48,
            933,
            1_700_000_000,
        ),
        (
            200,
            1_700_000_000 - 1000,
            5000,
            20,
            true,
            0.24,
            2700,
            1_700_000_000,
        ),
        (
            300,
            1_700_000_000 - 1000,
            5000,
            20,
            true,
            0.16,
            7800,
            1_700_000_000,
        ),
        (
            400,
            1_700_000_000 - 1000,
            5000,
            20,
            true,
            0.12,
            10400,
            1_700_000_000,
        ),
        (
            100,
            1_700_000_000 - 1000,
            5000,
            20,
            false,
            0.48,
            100,
            1_699_999_480,
        ),
        (
            200,
            1_700_000_000 - 1000,
            5000,
            20,
            false,
            0.24,
            200,
            1_699_999_240,
        ),
        (
            300,
            1_700_000_000 - 1000,
            5000,
            20,
            false,
            0.16,
            300,
            1_699_999_160,
        ),
        (
            400,
            1_700_000_000 - 1000,
            5000,
            20,
            false,
            0.12,
            400,
            1_699_999_120,
        ),
    ];

    for (
        score,
        last_review,
        ms_for_review,
        character_length,
        correct,
        inverted_score,
        expected_score,
        expected_last_review,
    ) in test_cases
    {
        let (new_score, new_last_review) = calculate_raw_score(
            score,
            last_review,
            ms_for_review,
            character_length,
            correct,
            inverted_score,
            true, // Enable mock_last_review for consistent testing
        );

        assert_eq!(
            new_score,
            expected_score,
            "Incorrect score for input: {:?}",
            (
                score,
                last_review,
                ms_for_review,
                character_length,
                correct,
                inverted_score
            )
        );
        assert_eq!(
            new_last_review,
            expected_last_review,
            "Incorrect last_review for input: {:?}",
            (
                score,
                last_review,
                ms_for_review,
                character_length,
                correct,
                inverted_score
            )
        );
    }
}
