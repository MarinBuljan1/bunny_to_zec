use super::*;

#[test]
fn learning_status_and_conversation_gates_share_exact_boundaries() {
    for (words, name, band, next_threshold) in [
        (0.0, "Beginner", 1, 100.0),
        (99.999, "Beginner", 1, 100.0),
        (100.0, "Novice", 2, 300.0),
        (299.999, "Novice", 2, 300.0),
        (300.0, "Conversational", 3, 1000.0),
        (999.999, "Conversational", 3, 1000.0),
        (1000.0, "Intermediate", 4, 4000.0),
        (4000.0, "Advanced", 4, 10000.0),
        (10000.0, "Fluent", 4, 100000.0),
        (100000.0, "Fluent", 4, 100000.0),
    ] {
        let level = learning_level(words);
        assert_eq!(
            (
                level.name,
                level.maximum_conversation_band,
                level.next_threshold
            ),
            (name, band, next_threshold)
        );
    }
}

#[test]
fn derived_progress_caps_each_word_and_accumulates_beyond_i32() {
    let scores = (0..3_000)
        .map(|index| {
            (
                index.to_string(),
                ScoreData {
                    score: 2 * LEARNED_INTERVAL_SECONDS,
                    last_review: 0,
                },
            )
        })
        .collect();
    assert_eq!(
        calculate_progress_sum(&scores),
        3_000 * LEARNED_INTERVAL_SECONDS
    );
    assert_eq!(calculate_progress_sum(&HashMap::new()), 0);
}

#[test]
fn test_review_weights() {
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
        let result = review_weights(
            &input_values
                .iter()
                .map(|&(score, last_review)| ScoreData { score, last_review })
                .collect::<Vec<_>>(),
        );

        // Ensure result length matches expected length
        assert_eq!(
            result.len(),
            expected_output.len(),
            "Result length mismatch for input: {input_values:?}"
        );

        // Ensure each calculated value is within a small margin of error (floating point comparison)
        for (res, exp) in result.iter().zip(expected_output.iter()) {
            assert!(
                (res - exp).abs() < 1e-6,
                "Expected {exp}, got {res} for input {input_values:?}"
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
        let updated = calculate_raw_score(
            &ScoreData { score, last_review },
            &ReviewAttempt {
                elapsed_ms: ms_for_review,
                character_length,
                outcome: if correct {
                    ReviewOutcome::Correct
                } else {
                    ReviewOutcome::Incorrect
                },
            },
            inverted_score,
            1_700_000_000,
        );
        let (new_score, new_last_review) = (updated.score, updated.last_review);

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

const NOW: i64 = 1_700_000_000;

#[test]
fn scores_cross_the_old_i32_limit_without_changing_selection_or_review_math() {
    let score = i64::from(i32::MAX);
    let progress = SavedProgress {
        scores: HashMap::from([(
            "bok".into(),
            ScoreData {
                score,
                last_review: NOW - 1,
            },
        )]),
        last_data_update: NOW,
        conversations: Default::default(),
    };
    let attempt = ReviewAttempt {
        elapsed_ms: 1_500,
        character_length: 0,
        outcome: ReviewOutcome::Correct,
    };
    let updated = apply_review(&progress, &[review_word("bok")], &attempt, NOW).unwrap();
    assert_eq!(updated.scores["bok"].score, score + 1);
    assert_eq!(
        calculate_final_score(&progress.scores["bok"], NOW),
        i128::from(score + REVIEW_PRIORITY_INTERVAL_SECONDS - 1)
    );
}

#[test]
fn post_2038_correct_and_incorrect_reviews_preserve_seconds_and_decay() {
    let now = 2_200_000_000; // After the old Unix-second limit.
    let progress = SavedProgress {
        scores: HashMap::from([
            (
                "bok".into(),
                ScoreData {
                    score: 100,
                    last_review: now - 1_000,
                },
            ),
            (
                "other".into(),
                ScoreData {
                    score: 1_000,
                    last_review: now - 10,
                },
            ),
        ]),
        last_data_update: now - 100,
        conversations: Default::default(),
    };
    for (outcome, expected_score, expected_review) in [
        (ReviewOutcome::Correct, 1_100, now),
        (ReviewOutcome::Incorrect, 100, now),
    ] {
        let attempt = ReviewAttempt {
            elapsed_ms: 1_500,
            character_length: 0,
            outcome,
        };
        let updated = apply_review(&progress, &[review_word("bok")], &attempt, now).unwrap();
        assert_eq!(
            updated.scores["bok"],
            ScoreData {
                score: expected_score,
                last_review: expected_review
            }
        );
        assert_eq!(updated.scores["other"].score, 900);
        assert_eq!(updated.last_data_update, now);
    }
}

#[test]
fn maximum_i64_values_do_not_wrap_or_flatten_selection_priorities() {
    let progress = SavedProgress {
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
    let attempt = ReviewAttempt {
        elapsed_ms: 1_500,
        character_length: 0,
        outcome: ReviewOutcome::Correct,
    };
    let updated = apply_review(&progress, &[review_word("bok")], &attempt, i64::MAX).unwrap();
    assert_eq!(updated.scores["bok"].score, i64::MAX);
    assert_eq!(updated.scores["bok"].last_review, i64::MAX);
    let recently_reviewed = ScoreData {
        score: i64::MAX,
        last_review: i64::MAX,
    };
    let older = ScoreData {
        score: i64::MAX,
        last_review: 1,
    };
    assert!(
        calculate_final_score(&recently_reviewed, i64::MAX)
            > calculate_final_score(&older, i64::MAX)
    );
    assert_eq!(
        calculate_progress_sum(&updated.scores),
        LEARNED_INTERVAL_SECONDS
    );

    // A very long interval also cannot overflow score growth, and decay floors at zero.
    let long_interval = SavedProgress {
        scores: HashMap::from([(
            "bok".into(),
            ScoreData {
                score: 50,
                last_review: 1,
            },
        )]),
        last_data_update: 1,
        conversations: Default::default(),
    };
    let updated = apply_review(&long_interval, &[review_word("bok")], &attempt, i64::MAX).unwrap();
    assert_eq!(updated.scores["bok"].score, i64::MAX);
    assert_eq!(
        apply_review(&long_interval, &[], &attempt, i64::MAX)
            .unwrap()
            .scores["bok"]
            .score,
        0
    );
}

#[test]
fn clock_rollback_does_not_inflate_decay_or_subtract_earned_scores() {
    let progress = SavedProgress {
        scores: HashMap::from([
            (
                "bok".into(),
                ScoreData {
                    score: 100,
                    last_review: NOW + 100,
                },
            ),
            (
                "other".into(),
                ScoreData {
                    score: 200,
                    last_review: NOW,
                },
            ),
        ]),
        last_data_update: NOW + 200,
        conversations: Default::default(),
    };
    for outcome in [ReviewOutcome::Correct, ReviewOutcome::Incorrect] {
        let attempt = ReviewAttempt {
            elapsed_ms: 1_500,
            character_length: 0,
            outcome,
        };
        let updated = apply_review(&progress, &[review_word("bok")], &attempt, NOW).unwrap();
        assert_eq!(updated.scores["bok"].score, 100);
        assert_eq!(updated.scores["other"].score, 200);
    }
}

#[test]
fn recall_duration_boundary_and_divisor_clamps_follow_the_production_formula() {
    let progress = SavedProgress {
        scores: HashMap::new(),
        last_data_update: NOW,
        conversations: Default::default(),
    };
    for (elapsed_ms, character_length, expected_score) in [
        (2, 0, 8_640_000),       // First duration that scores, with the minimum divisor.
        (1_500, 100, 8_640_000), // Byte-length adjustment also hits the minimum.
        (150_000, 0, 86_400),    // Slow answers use the maximum divisor of ten.
    ] {
        let attempt = ReviewAttempt {
            elapsed_ms,
            character_length,
            outcome: ReviewOutcome::Correct,
        };
        let updated = apply_review(&progress, &[review_word("bok")], &attempt, NOW).unwrap();
        assert_eq!(updated.scores["bok"].score, expected_score);
    }
}

fn review_word(target: &str) -> Card {
    Card {
        base_language: "translation".into(),
        target_language: target.into(),
    }
}

#[test]
fn review_decays_other_words_but_uses_original_scores_for_reviewed_words() {
    let progress = SavedProgress {
        scores: HashMap::from([
            (
                "bok".into(),
                ScoreData {
                    score: 100,
                    last_review: NOW - 1_000,
                },
            ),
            (
                "je".into(),
                ScoreData {
                    score: 10_000,
                    last_review: NOW - 5,
                },
            ),
            (
                "low".into(),
                ScoreData {
                    score: 50,
                    last_review: NOW - 5,
                },
            ),
        ]),
        last_data_update: NOW - 100,
        conversations: Default::default(),
    };
    let attempt = ReviewAttempt {
        elapsed_ms: 1_500,
        character_length: 0,
        outcome: ReviewOutcome::Correct,
    };
    let updated = apply_review(&progress, &[review_word("bok")], &attempt, NOW).unwrap();
    assert_eq!(
        updated.scores["bok"],
        ScoreData {
            score: 1_100,
            last_review: NOW
        }
    );
    assert_eq!(
        updated.scores["je"],
        ScoreData {
            score: 9_900,
            last_review: NOW - 5
        }
    );
    assert_eq!(updated.scores["low"].score, 0);
    assert_eq!(updated.last_data_update, NOW);
    assert_eq!(progress.scores["bok"].score, 100);
    assert_eq!(progress.scores["je"].score, 10_000);
    assert_eq!(progress.last_data_update, NOW - 100);
}

#[test]
fn skipped_reviews_do_not_apply_decay_or_produce_progress_to_save() {
    let progress = SavedProgress {
        scores: HashMap::new(),
        last_data_update: NOW - 100,
        conversations: Default::default(),
    };
    for elapsed_ms in [-1, 0, 1] {
        let attempt = ReviewAttempt {
            elapsed_ms,
            character_length: 0,
            outcome: ReviewOutcome::Correct,
        };
        assert!(apply_review(&progress, &[review_word("bok")], &attempt, NOW).is_none());
    }
}

#[test]
fn incorrect_multiword_review_retains_scores_and_distributes_review_timestamps() {
    let progress = SavedProgress {
        scores: HashMap::from([
            (
                "bok".into(),
                ScoreData {
                    score: 100,
                    last_review: NOW - 1_000,
                },
            ),
            (
                "je".into(),
                ScoreData {
                    score: 100,
                    last_review: NOW - 1_000,
                },
            ),
        ]),
        last_data_update: NOW - 100,
        conversations: Default::default(),
    };
    let attempt = ReviewAttempt {
        elapsed_ms: 3_000,
        character_length: 0,
        outcome: ReviewOutcome::Incorrect,
    };
    let updated = apply_review(
        &progress,
        &[review_word("bok"), review_word("je")],
        &attempt,
        NOW,
    )
    .unwrap();
    for target in ["bok", "je"] {
        assert_eq!(
            updated.scores[target],
            ScoreData {
                score: 100,
                last_review: NOW - 500
            }
        );
    }
}

#[test]
fn unseen_words_use_ten_day_initial_interval_for_correct_answers() {
    let progress = SavedProgress {
        scores: HashMap::new(),
        last_data_update: NOW,
        conversations: Default::default(),
    };
    for (outcome, expected_score) in [
        (ReviewOutcome::Correct, LEARNED_INTERVAL_SECONDS),
        (ReviewOutcome::Incorrect, 0),
    ] {
        let attempt = ReviewAttempt {
            elapsed_ms: 1_500,
            character_length: 0,
            outcome,
        };
        let updated = apply_review(&progress, &[review_word("bok")], &attempt, NOW).unwrap();
        assert_eq!(
            updated.scores["bok"],
            ScoreData {
                score: expected_score,
                last_review: NOW
            }
        );
    }
}

#[test]
fn empty_review_word_list_still_decays_existing_progress_as_before() {
    let progress = SavedProgress {
        scores: HashMap::from([(
            "bok".into(),
            ScoreData {
                score: 100,
                last_review: NOW - 1_000,
            },
        )]),
        last_data_update: NOW - 20,
        conversations: Default::default(),
    };
    let attempt = ReviewAttempt {
        elapsed_ms: 1_500,
        character_length: 0,
        outcome: ReviewOutcome::Correct,
    };
    let updated = apply_review(&progress, &[], &attempt, NOW).unwrap();
    assert_eq!(updated.scores["bok"].score, 80);
    assert_eq!(updated.last_data_update, NOW);
}

#[test]
fn normalisation_handles_empty_and_zero_scores() {
    assert!(review_weights(&[]).is_empty());
    let scores = vec![
        ScoreData {
            score: 0,
            last_review: 0
        };
        2
    ];
    assert_eq!(review_weights(&scores), vec![0.5, 0.5]);
}

#[test]
fn priorities_use_explicit_time_and_the_unseen_word_threshold() {
    let scores: HashMap<String, ScoreData> = HashMap::from([(
        "known".into(),
        ScoreData {
            score: 10,
            last_review: NOW,
        },
    )]);
    assert_eq!(
        calculate_final_score(&scores["known"], NOW + REVIEW_PRIORITY_INTERVAL_SECONDS - 1),
        11
    );
    assert_eq!(
        calculate_final_score(&scores["known"], NOW + REVIEW_PRIORITY_INTERVAL_SECONDS),
        10
    );
    assert_eq!(
        calculate_final_score(
            &ScoreData {
                score: 99,
                last_review: 0
            },
            NOW
        ),
        i128::from(LEARNED_INTERVAL_SECONDS)
    );
}
