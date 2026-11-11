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

#[test]
fn skipping_an_unrevealed_question_has_no_review_duration() {
    let state = Rc::new(ReviewState::welcome(1_000))
        .reduce(ReviewAction::Present(exercise("bok")))
        .reduce(ReviewAction::StartTimer(2_000));

    assert!(state.is_question());
    assert_eq!(state.duration(), 0);
    assert!(!state.show_explanations());
}

#[test]
fn revealing_stops_timing_before_explanations_appear() {
    let state = Rc::new(ReviewState::welcome(1_000))
        .reduce(ReviewAction::Present(exercise("bok")))
        .reduce(ReviewAction::StartTimer(2_000))
        .reduce(ReviewAction::Reveal(3_500));

    assert!(!state.is_question());
    assert_eq!(state.duration(), 1_500);
    assert!(!state.show_explanations());

    let state = state.reduce(ReviewAction::ShowExplanations);
    assert_eq!(state.duration(), 1_500);
    assert!(state.show_explanations());
}

#[test]
fn flipping_an_answer_again_does_not_change_duration() {
    let state = Rc::new(ReviewState::welcome(1_000))
        .reduce(ReviewAction::Reveal(2_000))
        .reduce(ReviewAction::ShowExplanations)
        .reduce(ReviewAction::Reveal(9_000));

    assert_eq!(state.duration(), 1_000);
    assert!(state.show_explanations());
    assert!(matches!(state.kind, ReviewKind::Welcome));
}

#[test]
fn a_zero_duration_reveal_is_still_an_answer() {
    let state = Rc::new(ReviewState::welcome(1_000))
        .reduce(ReviewAction::Reveal(1_000))
        .reduce(ReviewAction::Reveal(5_000));
    assert!(!state.is_question());
    assert_eq!(state.duration(), 0);
}

#[test]
fn presenting_a_card_replaces_all_review_content_together() {
    let state = Rc::new(ReviewState::welcome(0)).reduce(ReviewAction::Present(exercise("bok")));
    assert!(matches!(state.kind, ReviewKind::Exercise));
    assert_eq!(state.presentation.card.target_language, "bok");
    assert_eq!(
        state.presentation.translations.list[0].target_language,
        "bok"
    );
    assert!(state.presentation.sentence == exercise("bok").sentence);
    assert!(state.presentation.explanation == exercise("bok").explanation);

    let next = exercise("je");
    let state = state.reduce(ReviewAction::Present(next.clone()));
    assert!(state.presentation == next);
}

#[test]
fn welcome_identity_does_not_depend_on_card_text() {
    let mut presentation = exercise("Swipe Up or Down");
    presentation.card.base_language = "Swipe to Begin".into();
    let state = Rc::new(ReviewState::welcome(0)).reduce(ReviewAction::Present(presentation));
    assert!(matches!(state.kind, ReviewKind::Exercise));
}

#[test]
fn next_review_hides_explanations_then_starts_a_fresh_timer() {
    let state = Rc::new(ReviewState::welcome(1_000))
        .reduce(ReviewAction::Reveal(2_000))
        .reduce(ReviewAction::ShowExplanations)
        .reduce(ReviewAction::Present(exercise("je")))
        .reduce(ReviewAction::HideExplanations);
    assert!(!state.show_explanations());

    let state = state.reduce(ReviewAction::StartTimer(3_000));
    assert!(state.is_question());
    assert_eq!(state.duration(), 0);
    let state = state.reduce(ReviewAction::Reveal(3_750));
    assert_eq!(state.duration(), 750);
}

#[test]
fn delayed_explanations_do_not_reveal_an_unanswered_question() {
    let state = Rc::new(ReviewState::welcome(1_000)).reduce(ReviewAction::ShowExplanations);
    assert!(state.is_question());
    assert!(!state.show_explanations());
}

#[test]
fn drag_visuals_are_derived_and_reset_together() {
    let drag = Rc::new(DragState::Idle)
        .reduce(DragAction::Start(Point { x: 10.0, y: 20.0 }))
        .reduce(DragAction::Move {
            position: Point { x: 110.0, y: 45.0 },
            viewport_width: 400.0,
        });
    assert!(drag.is_moving());
    assert_eq!(drag.offset().x, 100.0);
    assert_eq!(drag.offset().y, 25.0);
    assert_eq!(drag.rotation(), 22.5);
    assert_eq!(drag.intensity(), 1.0);
    assert_eq!(drag.direction(), SwipeDirection::Right);

    let drag = drag.reduce(DragAction::End);
    assert!(!drag.is_moving());
    assert_eq!(drag.offset().x, 0.0);
    assert_eq!(drag.offset().y, 0.0);
    assert_eq!(drag.rotation(), 0.0);
    assert_eq!(drag.intensity(), 0.0);
    assert_eq!(drag.direction(), SwipeDirection::None);
}

#[test]
fn completed_swipes_preserve_thresholds_directions_and_flip_back() {
    let drag = Rc::new(DragState::Idle).reduce(DragAction::Start(Point::default()));
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
            drag.completed_direction(end, &SwipeDirection::None),
            expected
        );
    }
    assert_eq!(
        drag.completed_direction(Point { x: 50.0, y: 0.0 }, &SwipeDirection::Left),
        Some(SwipeDirection::None),
    );
}
