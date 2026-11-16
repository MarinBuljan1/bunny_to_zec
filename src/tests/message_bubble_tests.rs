use super::*;

#[test]
fn info_flips_only_animate_transform_and_opacity() {
    let frames = animation_frames(BubbleTransition::Info);
    for frame in frames.as_array().unwrap() {
        let properties = frame.as_object().unwrap();
        assert!(!properties.contains_key("width"));
        assert!(!properties.contains_key("height"));
        assert!(properties.contains_key("transform"));
        assert!(properties.contains_key("opacity"));
    }
}

#[test]
fn dot_growth_retains_the_typing_bubble_resize_transition() {
    let frames = animation_frames(BubbleTransition::Dot {
        from: (36.0, 36.0),
        to: (90.0, 52.0),
    });
    assert_eq!(frames[0]["width"], "36px");
    assert_eq!(frames[0]["borderRadius"], "50%");
    assert_eq!(frames[1]["width"], "90px");
    assert_eq!(frames[1]["height"], "52px");
}
