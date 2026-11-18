use super::*;

#[test]
fn vertical_movement_does_not_reset_horizontal_card_position() {
    let mut drag = ScrollDrag::new(100.0, 300.0, 0.0);
    let diagonal = drag.move_to(160.0, 200.0, 16.0);
    assert_eq!(diagonal.horizontal, 60.0);
    assert_eq!(diagonal.vertical, 100.0);
    let vertical = drag.move_to(160.0, 250.0, 32.0);
    assert_eq!(vertical.horizontal, 60.0);
    assert_eq!(vertical.vertical, -50.0);
    let back = drag.move_to(100.0, 250.0, 48.0);
    assert_eq!(back.horizontal, 0.0);
    assert_eq!(back.vertical, 0.0);
}

#[test]
fn both_axes_respond_to_one_pixel_movements_without_a_mode_threshold() {
    let mut drag = ScrollDrag::new(0.0, 0.0, 0.0);
    let first = drag.move_to(1.0, -1.0, 16.0);
    assert_eq!(first.horizontal, 1.0);
    assert_eq!(first.vertical, 1.0);
    let next = drag.move_to(1.0, 0.0, 32.0);
    assert_eq!(next.horizontal, 1.0);
    assert_eq!(next.vertical, -1.0);
}

#[test]
fn repeated_direction_changes_do_not_rebase_either_axis() {
    let mut drag = ScrollDrag::new(0.0, 0.0, 0.0);
    for (index, (x, y, expected_vertical)) in [
        (60.0, 0.0, 0.0),
        (60.0, -80.0, 80.0),
        (-20.0, -80.0, 0.0),
        (-20.0, -10.0, -70.0),
    ]
    .into_iter()
    .enumerate()
    {
        let step = drag.move_to(x, y, (index + 1) as f64 * 16.0);
        assert_eq!(step.horizontal, x);
        assert_eq!(step.vertical, expected_vertical);
    }
}

#[test]
fn release_does_not_restart_motion_after_a_pause_or_a_flip() {
    let mut drag = ScrollDrag::new(0.0, 100.0, 0.0);
    drag.move_to(0.0, 50.0, 16.0);
    assert!(drag.release_velocity(20.0) > 0.0);
    assert_eq!(drag.release_velocity(120.0), 0.0);
    drag.move_to(60.0, 50.0, 136.0);
    assert_eq!(drag.release_velocity(140.0), 0.0);
}

#[test]
fn momentum_follows_reversals_and_does_not_reuse_velocity_after_a_pause() {
    let mut drag = ScrollDrag::new(0.0, 1000.0, 0.0);
    drag.move_to(0.0, 0.0, 16.0);
    drag.move_to(0.0, 80.0, 32.0);
    assert!(drag.release_velocity(40.0) < 0.0);
    drag.move_to(0.0, 81.0, 200.0);
    assert!(drag.release_velocity(205.0).abs() < 0.02);
}
