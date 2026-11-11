use super::*;

fn exercise(word: &str) -> ReviewPresentation {
    ReviewPresentation {
        card: Card {
            base_language: "translation".into(),
            target_language: word.into(),
        },
        translations: CurrentTranslations {
            list: vec![Card {
                base_language: "translation".into(),
                target_language: word.into(),
            }],
        },
        sentence: html! { <span>{word.to_owned()}</span> },
        explanation: html! { "explanation" },
    }
}

fn elapse(state: Rc<ReviewState>, at_ms: i64) -> Rc<ReviewState> {
    let generation = state
        .animation
        .step()
        .expect("test transition must have a pending timer")
        .generation;
    state.reduce(ReviewAction::AnimationElapsed { generation, at_ms })
}

fn present_next(state: Rc<ReviewState>, presentation: ReviewPresentation) -> Rc<ReviewState> {
    let generation = state
        .animation
        .step()
        .expect("test presentation must occur during repositioning")
        .generation;
    state.reduce(ReviewAction::PresentNext {
        generation,
        presentation,
    })
}

fn question(presentation: ReviewPresentation) -> Rc<ReviewState> {
    let state = Rc::new(ReviewState::welcome(0)).reduce(ReviewAction::Swipe {
        direction: SwipeDirection::Up,
        at_ms: 1_450,
    });
    let state = present_next(elapse(state, 1_850), presentation);
    elapse(elapse(state, 1_900), 2_000)
}

#[test]
fn skipping_an_unrevealed_question_has_no_review_duration() {
    let state = question(exercise("bok"));
    assert!(state.is_question());
    assert_eq!(state.duration(), 0);
    assert!(!state.show_explanations());
}

#[test]
fn revealing_stops_timing_before_explanations_appear() {
    let state = question(exercise("bok")).reduce(ReviewAction::Swipe {
        direction: SwipeDirection::Left,
        at_ms: 3_500,
    });
    assert!(!state.is_question());
    assert_eq!(state.duration(), 1_500);
    assert!(!state.show_explanations());
    let state = elapse(state, 3_900);
    assert_eq!(state.duration(), 1_500);
    assert!(state.show_explanations());
}

#[test]
fn flipping_an_answer_again_does_not_change_duration() {
    let state = question(exercise("bok")).reduce(ReviewAction::Swipe {
        direction: SwipeDirection::Left,
        at_ms: 3_500,
    });
    let state = elapse(state, 3_900)
        .reduce(ReviewAction::Swipe {
            direction: SwipeDirection::None,
            at_ms: 4_000,
        })
        .reduce(ReviewAction::Swipe {
            direction: SwipeDirection::Right,
            at_ms: 5_000,
        });
    assert_eq!(state.duration(), 1_500);
    assert!(state.show_explanations());
    assert!(state.animation.step().is_none());
}

#[test]
fn a_zero_duration_reveal_is_still_an_answer() {
    let state = question(exercise("bok"))
        .reduce(ReviewAction::Swipe {
            direction: SwipeDirection::Left,
            at_ms: 2_000,
        })
        .reduce(ReviewAction::Swipe {
            direction: SwipeDirection::None,
            at_ms: 2_100,
        });
    assert!(!state.is_question());
    assert_eq!(state.duration(), 0);
}

#[test]
fn presenting_a_card_replaces_all_review_content_together() {
    let state = question(exercise("bok")).reduce(ReviewAction::Swipe {
        direction: SwipeDirection::Up,
        at_ms: 4_000,
    });
    let next = exercise("je");
    let state = present_next(elapse(state, 4_400), next.clone());
    assert!(state.presentation == next);
    assert!(matches!(state.kind, ReviewKind::Exercise));
}

#[test]
fn welcome_identity_does_not_depend_on_card_text() {
    let mut presentation = exercise("Swipe Up or Down");
    presentation.card.base_language = "Swipe to Begin".into();
    let state = question(presentation);
    assert!(matches!(state.kind, ReviewKind::Exercise));
    let welcome = Rc::new(ReviewState::welcome(0)).reduce(ReviewAction::Swipe {
        direction: SwipeDirection::Left,
        at_ms: 1_000,
    });
    assert!(matches!(welcome.kind, ReviewKind::Welcome));
}

#[test]
fn next_review_hides_explanations_then_starts_a_fresh_timer() {
    let state = question(exercise("bok")).reduce(ReviewAction::Swipe {
        direction: SwipeDirection::Left,
        at_ms: 3_500,
    });
    let state = elapse(state, 3_900).reduce(ReviewAction::Swipe {
        direction: SwipeDirection::Up,
        at_ms: 4_000,
    });
    let state = present_next(elapse(state, 4_400), exercise("je"));
    let state = elapse(state, 4_450);
    assert!(!state.show_explanations());
    assert!(!state.animation.accepts_swipes());
    let state = elapse(state, 4_550);
    assert!(state.is_question());
    assert_eq!(state.duration(), 0);
    let state = state.reduce(ReviewAction::Swipe {
        direction: SwipeDirection::Left,
        at_ms: 5_300,
    });
    assert_eq!(state.duration(), 750);
}

#[test]
fn old_reveal_callbacks_cannot_reveal_a_new_question() {
    let state = question(exercise("bok")).reduce(ReviewAction::Swipe {
        direction: SwipeDirection::Left,
        at_ms: 3_500,
    });
    let old_reveal = state.animation.step().unwrap().generation;
    let state = state.reduce(ReviewAction::Swipe {
        direction: SwipeDirection::Up,
        at_ms: 3_600,
    });
    let state = present_next(elapse(state, 4_000), exercise("je"));
    let state = elapse(elapse(state, 4_050), 4_150).reduce(ReviewAction::AnimationElapsed {
        generation: old_reveal,
        at_ms: 9_000,
    });
    assert!(state.is_question());
    assert!(!state.show_explanations());
    assert_eq!(state.duration(), 0);
    assert_eq!(state.presentation.card.target_language, "je");
}

#[test]
fn old_presentation_cannot_replace_the_current_card() {
    let state = question(exercise("bok")).reduce(ReviewAction::Swipe {
        direction: SwipeDirection::Up,
        at_ms: 3_000,
    });
    let state = elapse(state, 3_400);
    let old_generation = state.animation.step().unwrap().generation;
    let state = present_next(state, exercise("je"));
    let state = elapse(elapse(state, 3_450), 3_550).reduce(ReviewAction::PresentNext {
        generation: old_generation,
        presentation: exercise("old"),
    });
    assert_eq!(state.presentation.card.target_language, "je");
}

#[test]
fn rapid_swipes_during_advance_do_not_change_the_accepted_review_or_timer() {
    let state = question(exercise("bok"))
        .reduce(ReviewAction::Swipe {
            direction: SwipeDirection::Left,
            at_ms: 3_000,
        })
        .reduce(ReviewAction::Swipe {
            direction: SwipeDirection::Up,
            at_ms: 3_100,
        });
    let accepted = state.animation.step();
    let duration = state.duration();
    let state = state
        .reduce(ReviewAction::Swipe {
            direction: SwipeDirection::Down,
            at_ms: 3_101,
        })
        .reduce(ReviewAction::Swipe {
            direction: SwipeDirection::Right,
            at_ms: 3_102,
        });
    assert_eq!(state.animation.step(), accepted);
    assert_eq!(state.duration(), duration);
    assert_eq!(state.animation.direction(), SwipeDirection::Up);
}

#[test]
fn revealing_after_an_unanswered_advance_is_ignored() {
    let state = question(exercise("bok")).reduce(ReviewAction::Swipe {
        direction: SwipeDirection::Up,
        at_ms: 3_000,
    });
    let accepted = state.animation.step();
    let state = state.reduce(ReviewAction::Swipe {
        direction: SwipeDirection::Left,
        at_ms: 3_001,
    });
    assert_eq!(state.animation.step(), accepted);
    assert!(state.is_question());
    assert_eq!(state.duration(), 0);
}

#[test]
fn no_next_presentation_preserves_the_card_and_restarts_its_timer() {
    let state = question(exercise("bok")).reduce(ReviewAction::Swipe {
        direction: SwipeDirection::Up,
        at_ms: 3_000,
    });
    // This follows the existing no-eligible-card fallback without PresentNext.
    let state = elapse(elapse(elapse(state, 3_400), 3_450), 3_550);
    assert_eq!(state.presentation.card.target_language, "bok");
    assert!(state.animation.accepts_swipes());
    let state = state.reduce(ReviewAction::Swipe {
        direction: SwipeDirection::Left,
        at_ms: 4_000,
    });
    assert_eq!(state.duration(), 450);
}

#[test]
fn drag_visuals_are_derived_and_reset_together() {
    let drag = Rc::new(DragState::Idle)
        .reduce(DragAction::Start {
            pointer_id: 7,
            origin: Point { x: 10.0, y: 20.0 },
        })
        .reduce(DragAction::Move {
            pointer_id: 7,
            position: Point { x: 110.0, y: 45.0 },
            viewport_width: 400.0,
        });
    assert!(drag.is_moving());
    assert_eq!(drag.offset().x, 100.0);
    assert_eq!(drag.offset().y, 25.0);
    assert_eq!(drag.rotation(), 22.5);
    assert_eq!(drag.intensity(), 1.0);
    assert_eq!(drag.direction(), SwipeDirection::Right);

    let drag = drag.reduce(DragAction::End(7));
    assert!(!drag.is_moving());
    assert_eq!(drag.offset().x, 0.0);
    assert_eq!(drag.offset().y, 0.0);
    assert_eq!(drag.rotation(), 0.0);
    assert_eq!(drag.intensity(), 0.0);
    assert_eq!(drag.direction(), SwipeDirection::None);
}

#[test]
fn completed_swipes_preserve_thresholds_directions_and_flip_back() {
    let drag = Rc::new(DragState::Idle).reduce(DragAction::Start {
        pointer_id: 7,
        origin: Point::default(),
    });
    let examples = [
        (Point { x: 49.0, y: 0.0 }, None),
        (Point { x: 100.0, y: 100.0 }, None),
        (Point { x: 0.0, y: -50.0 }, Some(SwipeDirection::Up)),
        (Point { x: 0.0, y: 50.0 }, Some(SwipeDirection::Down)),
        (Point { x: -50.0, y: 0.0 }, Some(SwipeDirection::Left)),
        (Point { x: 50.0, y: 0.0 }, Some(SwipeDirection::Right)),
    ];
    for (end, expected) in examples {
        assert_eq!(
            drag.completed_direction(7, end, &SwipeDirection::None),
            expected
        );
    }
    assert_eq!(
        drag.completed_direction(7, Point { x: 50.0, y: 0.0 }, &SwipeDirection::Left),
        Some(SwipeDirection::None),
    );
}

#[test]
fn another_pointer_cannot_replace_or_move_an_active_drag() {
    let drag = Rc::new(DragState::Idle)
        .reduce(DragAction::Start {
            pointer_id: 7,
            origin: Point { x: 10.0, y: 20.0 },
        })
        .reduce(DragAction::Start {
            pointer_id: 8,
            origin: Point { x: 500.0, y: 500.0 },
        })
        .reduce(DragAction::Move {
            pointer_id: 8,
            position: Point { x: 900.0, y: 900.0 },
            viewport_width: 400.0,
        });
    assert!(drag.is_active_pointer(7));
    assert!(!drag.is_active_pointer(8));
    assert_eq!(drag.offset().x, 0.0);
    assert_eq!(drag.offset().y, 0.0);
    let drag = drag.reduce(DragAction::Move {
        pointer_id: 7,
        position: Point { x: 110.0, y: 45.0 },
        viewport_width: 400.0,
    });
    assert_eq!(drag.offset().x, 100.0);
    assert_eq!(drag.offset().y, 25.0);
}

#[test]
fn another_pointer_cannot_complete_or_cancel_an_active_drag() {
    let drag = Rc::new(DragState::Idle)
        .reduce(DragAction::Start {
            pointer_id: 7,
            origin: Point::default(),
        })
        .reduce(DragAction::End(8));
    assert!(drag.is_active_pointer(7));
    assert_eq!(
        drag.completed_direction(8, Point { x: 0.0, y: -100.0 }, &SwipeDirection::None),
        None
    );
    assert_eq!(
        drag.completed_direction(7, Point { x: 0.0, y: -100.0 }, &SwipeDirection::None),
        Some(SwipeDirection::Up)
    );
}

#[test]
fn cancelled_drag_cannot_later_complete_a_swipe_and_next_pointer_can_start() {
    let drag = Rc::new(DragState::Idle)
        .reduce(DragAction::Start {
            pointer_id: 7,
            origin: Point::default(),
        })
        .reduce(DragAction::Move {
            pointer_id: 7,
            position: Point { x: 0.0, y: -100.0 },
            viewport_width: 400.0,
        })
        .reduce(DragAction::End(7));
    assert!(!drag.is_moving());
    assert_eq!(drag.intensity(), 0.0);
    assert_eq!(
        drag.completed_direction(7, Point { x: 0.0, y: -100.0 }, &SwipeDirection::None),
        None
    );
    let drag = drag.reduce(DragAction::Start {
        pointer_id: 8,
        origin: Point { x: 200.0, y: 200.0 },
    });
    assert!(drag.is_active_pointer(8));
    assert_eq!(drag.offset().x, 0.0);
}

#[test]
fn captured_pointer_can_complete_beyond_the_original_viewport() {
    let drag = Rc::new(DragState::Idle).reduce(DragAction::Start {
        pointer_id: 7,
        origin: Point { x: 100.0, y: 100.0 },
    });
    assert_eq!(
        drag.completed_direction(
            7,
            Point {
                x: -200.0,
                y: 100.0
            },
            &SwipeDirection::None
        ),
        Some(SwipeDirection::Left)
    );
    assert_eq!(
        drag.completed_direction(
            7,
            Point {
                x: 100.0,
                y: -200.0
            },
            &SwipeDirection::None
        ),
        Some(SwipeDirection::Up)
    );
}
