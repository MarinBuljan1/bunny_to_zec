use super::*;
#[test]
fn sideways_movement_keeps_the_pull_and_vertical_motion_keeps_the_card_offset() {
    let mut drag = crate::scroll_drag::ScrollDrag::new(0.0, 100.0, 0.0);
    let mut pull = BottomPull::default();
    let diagonal = drag.move_to(60.0, 0.0, 16.0);
    assert_eq!(pull.consume(diagonal.vertical, true, true), 0.0);
    assert_eq!(pull.progress(), 1.0);
    let sideways = drag.move_to(80.0, 0.0, 32.0);
    assert_eq!(pull.consume(sideways.vertical, true, true), 0.0);
    assert_eq!(pull.progress(), 1.0);
    let reverse = drag.move_to(80.0, 8.0, 48.0);
    assert_eq!(reverse.horizontal, 80.0);
    assert_eq!(pull.consume(reverse.vertical, true, true), 0.0);
    assert_eq!(pull.progress(), 64.0 / 72.0);
}
#[test]
fn release_action_changes_only_at_acceptance_or_direction_boundaries() {
    for distance in 0..50 {
        assert_eq!(
            armed_swipe(0.0, -f64::from(distance), GestureMode::Rate),
            None
        );
    }
    let armed = armed_swipe(0.0, -50.0, GestureMode::Rate);
    assert_eq!(armed, Some(ChatAction::Rate(Rating::Correct)));
    assert_eq!(armed_swipe(0.0, -80.0, GestureMode::Rate), armed);
    assert_eq!(armed_swipe(0.0, -49.0, GestureMode::Rate), None);
    assert_eq!(
        armed_swipe(80.0, 0.0, GestureMode::Rate),
        Some(ChatAction::Flip)
    );
}
#[test]
fn notes_can_flip_and_only_overflowing_notes_reserve_vertical_drags_for_scrolling() {
    let short_notes = notes_gesture_mode(GestureMode::Rate, false);
    let (dx, dy) = (0.0, -60.0);
    assert!(swipe_feedback(dx, dy, short_notes).armed);
    assert_eq!(
        armed_swipe(dx, dy, GestureMode::Rate),
        Some(ChatAction::Rate(Rating::Correct))
    );
    let long_notes = notes_gesture_mode(GestureMode::Rate, true);
    assert!(!swipe_feedback(0.0, -60.0, long_notes).armed);
    assert!(swipe_feedback(60.0, 2.0, long_notes).armed);
    assert_eq!(
        armed_swipe(60.0, 2.0, GestureMode::Rate),
        Some(ChatAction::Flip)
    );
    assert!(notes_gesture_mode(GestureMode::Disabled, true) == GestureMode::Disabled);
}
#[test]
fn pull_dot_stops_growing_at_the_threshold_and_shrinks_when_the_pull_is_reversed() {
    let mut pull = BottomPull::default();
    pull.move_by(72.0, true, true);
    assert_eq!(pull.progress(), 1.0);
    pull.move_by(100.0, true, true);
    assert_eq!(pull.progress(), 1.0);
    pull.move_by(-36.0, true, true);
    assert_eq!(pull.progress(), 0.5);
    assert!(!pull.finish(true));
    assert_eq!(pull.progress(), 0.0);
}
#[test]
fn reversing_a_full_pull_consumes_the_dot_before_scrolling_history() {
    let mut pull = BottomPull::default();
    assert_eq!(pull.consume(300.0, true, true), 0.0);
    assert_eq!(pull.progress(), 1.0);
    assert_eq!(pull.consume(-8.0, true, true), 0.0);
    assert_eq!(pull.progress(), 64.0 / 72.0);
    // A transient bottom measurement must not discard a pull already in progress.
    assert_eq!(pull.consume(-4.0, true, false), 0.0);
    assert_eq!(pull.progress(), 60.0 / 72.0);
    assert_eq!(pull.consume(-70.0, true, false), -10.0);
    assert_eq!(pull.progress(), 0.0);
    assert!(!pull.finish(true));
}
#[test]
fn a_drag_can_change_direction_before_release() {
    let positions = [
        (80.0, 2.0, Some(ChatAction::Flip)),
        (10.0, -80.0, Some(ChatAction::Rate(Rating::Correct))),
        (5.0, 80.0, Some(ChatAction::Rate(Rating::Incorrect))),
        (-80.0, 5.0, Some(ChatAction::Flip)),
    ];
    for (dx, dy, expected) in positions {
        let feedback = swipe_feedback(dx, dy, GestureMode::Rate);
        assert!(feedback.armed);
        assert_eq!(feedback.action, expected);
        assert_eq!(armed_swipe(dx, dy, GestureMode::Rate), expected);
    }
    assert!(!swipe_feedback(80.0, 80.0, GestureMode::Rate).armed);
    assert_eq!(armed_swipe(80.0, 80.0, GestureMode::Rate), None);
}
#[test]
fn resisted_movement_stays_continuous_and_bounded_beyond_the_rating_threshold() {
    assert_eq!(resisted_offset(0.0), 0.0);
    assert!((resisted_offset(5.0) - 5.0).abs() < 0.05);
    assert!(resisted_offset(50.0) > 28.0);
    assert!(resisted_offset(51.0) > resisted_offset(50.0));
    assert!(resisted_offset(10000.0) <= 32.0);
    assert_eq!(resisted_offset(-50.0), -resisted_offset(50.0));
}
#[test]
fn pull_progress_and_acceptance_use_the_same_distance() {
    let mut pull = BottomPull::default();
    pull.move_by(36.0, true, true);
    assert_eq!(pull.progress(), 0.5);
    pull.move_by(36.0, true, true);
    assert_eq!(pull.progress(), 1.0);
    assert!(pull.finish(true));
    assert_eq!(pull.progress(), 0.0);
    pull.move_by(71.0, true, true);
    assert!(pull.progress() < 1.0);
    assert!(!pull.finish(true));
}
#[test]
fn gesture_feedback_arms_at_the_same_threshold_as_the_action() {
    let partial = swipe_feedback(0.0, -49.0, GestureMode::Rate);
    assert_eq!(partial.action, Some(ChatAction::Rate(Rating::Correct)));
    assert!(!partial.armed);
    assert_eq!(partial.strength, 0.98);
    let complete = swipe_feedback(0.0, -50.0, GestureMode::Rate);
    assert!(complete.armed);
    assert_eq!(complete.strength, 1.0);
    assert!(!swipe_feedback(200.0, 200.0, GestureMode::Rate).armed);
    assert!(swipe_feedback(-50.0, 0.0, GestureMode::Flip).armed);
    assert!(swipe_feedback(0.0, 80.0, GestureMode::Flip)
        .action
        .is_none());
    assert!(!swipe_feedback(100.0, 0.0, GestureMode::Disabled).armed);
    assert!(swipe_feedback(0.0, 0.0, GestureMode::Rate).action.is_none());
}
#[test]
fn review_swipes_keep_the_threshold_and_reject_ambiguous_diagonals() {
    assert_eq!(armed_swipe(49.0, 0.0, GestureMode::Rate), None);
    assert_eq!(armed_swipe(100.0, 100.0, GestureMode::Rate), None);
    assert_eq!(
        armed_swipe(50.0, 0.0, GestureMode::Rate),
        Some(ChatAction::Flip)
    );
    assert_eq!(
        armed_swipe(-50.0, 0.0, GestureMode::Rate),
        Some(ChatAction::Flip)
    );
    assert_eq!(
        armed_swipe(0.0, -50.0, GestureMode::Rate),
        Some(ChatAction::Rate(Rating::Correct))
    );
    assert_eq!(
        armed_swipe(0.0, 50.0, GestureMode::Rate),
        Some(ChatAction::Rate(Rating::Incorrect))
    );
}
#[test]
fn only_distance_beyond_bottom_advances_and_cancel_resets_it() {
    let mut pull = BottomPull::default();
    assert!(!pull.move_by(300.0, true, false));
    assert!(!pull.finish(true));
    assert!(pull.move_by(40.0, true, true));
    assert!(!pull.finish(true));
    pull.move_by(80.0, true, true);
    assert!(pull.finish(true));
    assert!(!pull.finish(true));
    pull.move_by(80.0, true, true);
    assert!(!pull.finish(false));
    assert!(!pull.finish(true));
}
#[test]
fn reversing_direction_or_rating_cannot_advance() {
    let mut pull = BottomPull::default();
    pull.move_by(50.0, true, true);
    pull.move_by(-30.0, true, true);
    assert!(!pull.finish(true));
    assert!(!pull.move_by(200.0, false, true));
    assert!(!pull.finish(true));
}

#[test]
fn growing_preview_does_not_reset_a_pull_when_the_layout_moves_the_bottom() {
    let mut pull = BottomPull::default();
    assert_eq!(pull.consume(18.0, true, true), 0.0);
    assert_eq!(pull.progress(), 0.25);
    // The preview's increasing height can temporarily change the DOM bottom reading.
    assert_eq!(pull.consume(18.0, true, false), 0.0);
    assert_eq!(pull.progress(), 0.5);
    assert_eq!(pull.consume(36.0, true, false), 0.0);
    assert_eq!(pull.progress(), 1.0);
    assert_eq!(pull.consume(-18.0, true, false), 0.0);
    assert_eq!(pull.progress(), 0.75);
    // Only the part left after fully shrinking the preview scrolls history.
    assert_eq!(pull.consume(-64.0, true, false), -10.0);
    assert_eq!(pull.progress(), 0.0);
    assert!(!pull.finish(true));
}
