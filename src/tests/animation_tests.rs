use super::*;

#[test]
fn advance_runs_the_original_400_50_100_millisecond_stages_once() {
    let mut state = AnimationState::default();
    assert!(state.accept_swipe(SwipeDirection::Up, 1_000));
    let leaving = state.step().unwrap();
    assert_eq!(leaving.kind, AnimationStepKind::Advance(SwipeDirection::Up));
    assert_eq!(leaving.delay_ms(), 400);
    assert!(!state.accept_swipe(SwipeDirection::Down, 1_001));
    assert_eq!(state.step(), Some(leaving.clone()));
    assert_eq!(state.elapse(leaving.generation, 1_400), Some(leaving.kind));
    let reposition = state.step().unwrap();
    assert!(state.fast_move());
    assert_eq!(state.direction(), SwipeDirection::Down);
    assert_eq!(reposition.delay_ms(), 50);
    assert!(!state.accepts_swipes());
    state.elapse(reposition.generation, 1_450);
    let starting = state.step().unwrap();
    assert_eq!(starting.kind, AnimationStepKind::StartQuestion);
    assert_eq!(starting.delay_ms(), 100);
    assert_eq!(state.direction(), SwipeDirection::None);
    assert!(!state.fast_move());
    assert!(!state.accepts_swipes());
    state.elapse(starting.generation, 1_550);
    assert!(state.step().is_none());
    assert!(state.accepts_swipes());
}

#[test]
fn quick_flip_back_keeps_the_first_reveal_timer() {
    let mut state = AnimationState::default();
    state.accept_swipe(SwipeDirection::Left, 1_000);
    state.begin_reveal(1_000);
    let pending = state.step().unwrap();
    assert!(state.accept_swipe(SwipeDirection::None, 1_100));
    assert_eq!(state.direction(), SwipeDirection::None);
    assert_eq!(state.step(), Some(pending.clone()));
    assert_eq!(pending.started_at_ms, 1_000);
    state.elapse(pending.generation, 1_400);
    assert!(state.step().is_none());
}

#[test]
fn advancing_supersedes_pending_reveal_and_rejects_its_late_callback() {
    let mut state = AnimationState::default();
    state.accept_swipe(SwipeDirection::Left, 1_000);
    state.begin_reveal(1_000);
    let reveal = state.step().unwrap();
    assert!(state.accept_swipe(SwipeDirection::Down, 1_100));
    let leaving = state.step().unwrap();
    assert_eq!(state.elapse(reveal.generation, 1_400), None);
    assert_eq!(state.step(), Some(leaving));
}

#[test]
fn duplicate_and_obsolete_elapsed_events_cannot_skip_stages() {
    let mut state = AnimationState::default();
    state.accept_swipe(SwipeDirection::Up, 1_000);
    let leaving = state.step().unwrap();
    state.elapse(leaving.generation, 1_400);
    let reposition = state.step().unwrap();
    assert_eq!(state.elapse(leaving.generation, 1_401), None);
    assert_eq!(state.step(), Some(reposition.clone()));
    assert!(state.can_present(reposition.generation));
    assert!(!state.can_present(leaving.generation));
    state.elapse(reposition.generation, 1_450);
    assert!(!state.can_present(reposition.generation));
}

#[test]
fn completed_reveal_has_no_pending_timer_and_late_events_do_nothing() {
    let mut state = AnimationState::default();
    state.accept_swipe(SwipeDirection::Left, 1_000);
    state.begin_reveal(1_000);
    let reveal = state.step().unwrap();
    state.elapse(reveal.generation, 1_400);
    assert!(state.step().is_none());
    assert_eq!(state.elapse(reveal.generation, 9_000), None);
    assert!(state.step().is_none());
    assert_eq!(state.direction(), SwipeDirection::Left);
}
