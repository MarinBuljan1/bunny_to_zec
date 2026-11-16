use super::*;

#[test]
fn glow_follows_the_action_axis_and_reverses_with_the_drag() {
    let left = glow_offset(Some(ChatAction::Flip), -80.0, 10.0);
    let right = glow_offset(Some(ChatAction::Flip), 80.0, -10.0);
    assert!(left.0 < 0.0);
    assert!(right.0 > 0.0);
    assert_eq!(left.1, 0.0);
    assert_eq!(right.1, 0.0);
    assert_eq!(left.0, -right.0);

    let up = glow_offset(Some(ChatAction::Rate(Rating::Correct)), 10.0, -80.0);
    let down = glow_offset(Some(ChatAction::Rate(Rating::Incorrect)), -10.0, 80.0);
    assert_eq!(up.0, 0.0);
    assert_eq!(down.0, 0.0);
    assert!(up.1 < 0.0);
    assert!(down.1 > 0.0);
    assert_eq!(up.1, -down.1);
    assert_eq!(glow_offset(None, 10.0, 20.0), (0.0, 0.0));
}
