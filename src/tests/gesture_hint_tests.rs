use super::*;

#[test]
fn release_instruction_matches_action_priority_and_clears_when_unarmed() {
    assert_eq!(release_instruction(None, false), None);
    assert_eq!(
        release_instruction(None, true),
        Some("Release for next messages")
    );
    assert_eq!(
        release_instruction(Some(ChatAction::Flip), true),
        Some("Release to flip")
    );
    assert_eq!(
        release_instruction(Some(ChatAction::Rate(Rating::Correct)), false),
        Some("Release to mark remembered")
    );
    assert_eq!(
        release_instruction(Some(ChatAction::Rate(Rating::Incorrect)), false),
        Some("Release to review again")
    );
}
