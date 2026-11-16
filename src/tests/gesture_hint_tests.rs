use super::*;

#[test]
fn release_instruction_matches_action_priority_and_clears_when_unarmed() {
    assert_eq!(release_instruction(ReviewHint::default(), false), None);
    assert_eq!(
        release_instruction(ReviewHint::default(), true),
        Some("Release for next messages")
    );
    assert_eq!(
        release_instruction(
            ReviewHint {
                action: Some(ChatAction::Flip),
                armed: true
            },
            true
        ),
        Some("Release to flip")
    );
    assert_eq!(
        release_instruction(
            ReviewHint {
                action: Some(ChatAction::Rate(Rating::Correct)),
                armed: true
            },
            false
        ),
        Some("Release to mark remembered")
    );
    assert_eq!(
        release_instruction(
            ReviewHint {
                action: Some(ChatAction::Flip),
                armed: false
            },
            false
        ),
        None
    );
}
