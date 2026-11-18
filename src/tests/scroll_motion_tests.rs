use super::*;

#[test]
fn pull_preview_sets_visible_dimensions_in_pixels_as_it_grows() {
    for (progress, space, dot) in [
        (0.0, "height: 0px", "width: 0px; height: 0px; opacity: 0"),
        (
            0.25,
            "height: 13px",
            "width: 9px; height: 9px; opacity: 0.25",
        ),
        (
            0.5,
            "height: 26px",
            "width: 18px; height: 18px; opacity: 0.5",
        ),
        (1.0, "height: 52px", "width: 36px; height: 36px; opacity: 1"),
    ] {
        let (space_style, dot_style) = preview_styles(progress);
        assert_eq!(space_style, space);
        assert_eq!(dot_style, dot);
    }
}
