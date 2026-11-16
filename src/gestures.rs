//! Review-message swipes and native history scrolling with a deliberate bottom pull.
use crate::conversations::{ChatAction, Rating};
use gloo_events::{EventListener, EventListenerOptions};
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, PointerEvent, WheelEvent};
use yew::prelude::*;

pub(crate) fn at_bottom(element: &HtmlElement) -> bool {
    element.scroll_height() - element.scroll_top() - element.client_height() <= 8
}

fn review_swipe(dx: f64, dy: f64) -> Option<ChatAction> {
    // Keep the previous axis-difference threshold; diagonal dragging is not a rating.
    if (dx.abs() - dy.abs()).abs() < 50.0 {
        return None;
    }
    if dx.abs() > dy.abs() {
        Some(ChatAction::Flip)
    } else {
        Some(ChatAction::Rate(if dy < 0.0 {
            Rating::Correct
        } else {
            Rating::Incorrect
        }))
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum GestureMode {
    Disabled,
    Flip,
    Rate,
}

pub(crate) struct SwipeFeedback {
    pub action: Option<ChatAction>,
    pub strength: f64,
    pub armed: bool,
}

pub(crate) fn swipe_feedback(dx: f64, dy: f64, mode: GestureMode) -> SwipeFeedback {
    let action = if mode == GestureMode::Disabled || dx.abs().max(dy.abs()) < 0.1 {
        None
    } else if dx.abs() > dy.abs() {
        Some(ChatAction::Flip)
    } else if mode == GestureMode::Rate {
        Some(ChatAction::Rate(if dy < 0.0 {
            Rating::Correct
        } else {
            Rating::Incorrect
        }))
    } else {
        None
    };
    let strength = if action.is_some() {
        ((dx.abs() - dy.abs()).abs() / 50.0).min(1.0)
    } else {
        0.0
    };
    SwipeFeedback {
        action,
        strength,
        armed: action.is_some() && strength >= 1.0,
    }
}

#[derive(Default)]
struct BottomPull {
    distance: f64,
}
impl BottomPull {
    fn progress(&self) -> f64 {
        (self.distance / 72.0).clamp(0.0, 1.0)
    }
    fn move_by(&mut self, delta: f64, enabled: bool, bottom: bool) -> bool {
        self.distance = if enabled && (bottom || self.distance > 0.0) {
            (self.distance + delta).clamp(0.0, 72.0)
        } else {
            0.0
        };
        self.distance > 0.0
    }
    /// Reverse through the visible pull before allowing the list to move away from the bottom.
    fn consume(&mut self, delta: f64, enabled: bool, bottom: bool) -> f64 {
        let previous = self.distance;
        self.move_by(delta, enabled, bottom);
        if enabled && (previous > 0.0 || (bottom && delta > 0.0)) {
            if delta > 0.0 {
                0.0
            } else {
                delta + previous - self.distance
            }
        } else {
            delta
        }
    }
    fn finish(&mut self, enabled: bool) -> bool {
        let distance = std::mem::take(&mut self.distance);
        enabled && distance >= 72.0
    }
}

pub(crate) fn resisted_offset(distance: f64) -> f64 {
    32.0 * (distance / 32.0).tanh()
}

#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub(crate) struct ReviewHint {
    pub action: Option<ChatAction>,
    pub armed: bool,
}

fn gesture_hint(dx: f64, dy: f64, mode: GestureMode) -> ReviewHint {
    let feedback = swipe_feedback(dx, dy, mode);
    // Unarmed per-pixel changes do not affect the visible instruction.
    ReviewHint {
        action: if feedback.armed {
            review_swipe(dx, dy)
        } else {
            None
        },
        armed: feedback.armed,
    }
}

fn notes_element(event: &web_sys::Event) -> Option<HtmlElement> {
    event
        .target()
        .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
        .and_then(|e| e.closest(".message-notes").ok().flatten())
        .and_then(|e| e.dyn_into::<HtmlElement>().ok())
}

fn starts_in_scrollable_notes(event: &web_sys::Event) -> bool {
    notes_element(event).is_some_and(|notes| notes.scroll_height() > notes.client_height())
}

fn notes_gesture_mode(mode: GestureMode, overflow: bool) -> GestureMode {
    if overflow && mode == GestureMode::Rate {
        GestureMode::Flip
    } else {
        mode
    }
}

#[derive(Clone)]
pub(crate) struct ConversationScroll {
    pub preview: NodeRef,
    timeline: NodeRef,
    can_advance: bool,
    update_preview: Callback<f64>,
    motion: Rc<std::cell::RefCell<crate::scroll_motion::ScrollMotion>>,
}

/// Wheel input remains native except while revealing the next-message dot.
#[hook]
pub(crate) fn use_conversation_scroll(
    timeline: NodeRef,
    mounted: bool,
    advance: Option<Callback<ChatAction>>,
) -> ConversationScroll {
    let preview_node = use_node_ref();
    let preview = use_mut_ref(|| crate::scroll_motion::PullPreview::new(preview_node.clone()));
    let motion = use_mut_ref(crate::scroll_motion::ScrollMotion::new);
    let wheel_pull = use_mut_ref(BottomPull::default);
    let update_preview = {
        let preview = preview.clone();
        Callback::from(move |value: f64| {
            crate::scroll_motion::queue_preview(&preview, value);
        })
    };
    let available = advance.is_some();
    {
        let update = update_preview.clone();
        let preview = preview.clone();
        let wheel_pull = wheel_pull.clone();
        use_effect_with_deps(
            move |_| {
                update.emit(0.0);
                wheel_pull.borrow_mut().finish(false);
                move || crate::scroll_motion::reset_preview(&preview)
            },
            available,
        );
    }
    let latest_advance = use_mut_ref(|| advance.clone());
    *latest_advance.borrow_mut() = advance;
    {
        let node = timeline.clone();
        let motion = motion.clone();
        let update = update_preview.clone();
        let wheel_pull = wheel_pull.clone();
        use_effect_with_deps(
            move |mounted| {
                let listener =
                    mounted
                        .then(|| node.cast::<HtmlElement>())
                        .flatten()
                        .map(|element| {
                            let scroll = element.clone();
                            let pull = wheel_pull;
                            EventListener::new_with_options(
                                &element,
                                "wheel",
                                EventListenerOptions::enable_prevent_default(),
                                move |event| {
                                    let Some(event) = event.dyn_ref::<WheelEvent>() else {
                                        return;
                                    };
                                    crate::scroll_motion::cancel_scroll(&motion);
                                    if starts_in_scrollable_notes(event)
                                        || latest_advance.borrow().is_none()
                                    {
                                        pull.borrow_mut().finish(false);
                                        update.emit(0.0);
                                        return;
                                    }
                                    let pixels = match event.delta_mode() {
                                        1 => event.delta_y() * 16.0,
                                        2 => event.delta_y() * f64::from(scroll.client_height()),
                                        _ => event.delta_y(),
                                    };
                                    let bottom = at_bottom(&scroll);
                                    let intercept =
                                        pull.borrow().progress() > 0.0 || (bottom && pixels > 0.0);
                                    let remaining =
                                        pull.borrow_mut().consume(pixels * 0.6, true, bottom);
                                    if intercept {
                                        event.prevent_default();
                                        if remaining != 0.0 {
                                            crate::scroll_motion::target_scroll(
                                                &motion,
                                                scroll.clone(),
                                            );
                                            crate::scroll_motion::queue_scroll(
                                                &motion,
                                                (remaining / 0.6).round() as i32,
                                            );
                                        }
                                    }
                                    let progress = pull.borrow().progress();
                                    update.emit(progress);
                                    if progress >= 1.0 {
                                        pull.borrow_mut().finish(false);
                                        update.emit(0.0);
                                        if let Some(callback) = latest_advance.borrow().as_ref() {
                                            callback.emit(ChatAction::Next);
                                        }
                                    }
                                },
                            )
                        });
                move || drop(listener)
            },
            mounted,
        );
    }
    ConversationScroll {
        preview: preview_node,
        timeline,
        can_advance: available,
        update_preview,
        motion,
    }
}

#[cfg(test)]
mod tests {
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
    fn hint_rendering_changes_only_at_acceptance_or_direction_boundaries() {
        for distance in 0..50 {
            assert_eq!(
                gesture_hint(0.0, -f64::from(distance), GestureMode::Rate),
                ReviewHint::default()
            );
        }
        let armed = gesture_hint(0.0, -50.0, GestureMode::Rate);
        assert_eq!(armed.action, Some(ChatAction::Rate(Rating::Correct)));
        assert!(armed.armed);
        assert_eq!(gesture_hint(0.0, -80.0, GestureMode::Rate), armed);
        assert_eq!(
            gesture_hint(0.0, -49.0, GestureMode::Rate),
            ReviewHint::default()
        );
        assert_eq!(
            gesture_hint(80.0, 0.0, GestureMode::Rate).action,
            Some(ChatAction::Flip)
        );
    }
    #[test]
    fn notes_can_flip_and_only_overflowing_notes_reserve_vertical_drags_for_scrolling() {
        let short_notes = notes_gesture_mode(GestureMode::Rate, false);
        let (dx, dy) = (0.0, -60.0);
        assert!(swipe_feedback(dx, dy, short_notes).armed);
        assert_eq!(
            review_swipe(dx, dy),
            Some(ChatAction::Rate(Rating::Correct))
        );
        let long_notes = notes_gesture_mode(GestureMode::Rate, true);
        assert!(!swipe_feedback(0.0, -60.0, long_notes).armed);
        assert!(swipe_feedback(60.0, 2.0, long_notes).armed);
        assert_eq!(review_swipe(60.0, 2.0), Some(ChatAction::Flip));
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
            assert_eq!(review_swipe(dx, dy), expected);
        }
        assert!(!swipe_feedback(80.0, 80.0, GestureMode::Rate).armed);
        assert_eq!(review_swipe(80.0, 80.0), None);
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
        assert_eq!(review_swipe(49.0, 0.0), None);
        assert_eq!(review_swipe(100.0, 100.0), None);
        assert_eq!(review_swipe(50.0, 0.0), Some(ChatAction::Flip));
        assert_eq!(review_swipe(-50.0, 0.0), Some(ChatAction::Flip));
        assert_eq!(
            review_swipe(0.0, -50.0),
            Some(ChatAction::Rate(Rating::Correct))
        );
        assert_eq!(
            review_swipe(0.0, 50.0),
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
}

pub(crate) struct ReviewGestures {
    pub motion: NodeRef,
    pub hint: NodeRef,
    pub onpointerdown: Callback<PointerEvent>,
    pub onpointermove: Callback<PointerEvent>,
    pub onpointerup: Callback<PointerEvent>,
    pub cancel: Callback<PointerEvent>,
    pub tap: Callback<MouseEvent>,
    pub tap_start: Callback<PointerEvent>,
}

struct ActiveDrag {
    id: i32,
    x: f64,
    y: f64,
    mode: GestureMode,
    scroll: Option<crate::scroll_drag::ScrollDrag>,
    pull: BottomPull,
    allow_advance: bool,
    touch: bool,
}

fn move_active_drag(
    drag: &mut ActiveDrag,
    event: &PointerEvent,
    scroll: &ConversationScroll,
    motion: &Rc<std::cell::RefCell<crate::drag_motion::DragMotion>>,
) -> ReviewHint {
    let x = f64::from(event.client_x());
    let y = f64::from(event.client_y());
    if let Some(route) = drag.scroll.as_mut() {
        let can_advance = drag.allow_advance && scroll.can_advance;
        let step = route.move_to(x, y, event.time_stamp());
        // Both axes remain active. Vertical movement never clears horizontal displacement.
        let bottom = can_advance && crate::scroll_motion::at_bottom_after_pending(&scroll.motion);
        let remaining = drag.pull.consume(step.vertical, can_advance, bottom);
        if remaining != 0.0 {
            crate::scroll_motion::queue_scroll(&scroll.motion, remaining.round() as i32);
        }
        scroll.update_preview.emit(drag.pull.progress());
        crate::drag_motion::queue_motion(motion, step.horizontal, 0.0, GestureMode::Flip);
        gesture_hint(step.horizontal, 0.0, GestureMode::Flip)
    } else {
        let dx = x - drag.x;
        let dy = y - drag.y;
        crate::drag_motion::queue_motion(motion, dx, dy, drag.mode);
        gesture_hint(dx, dy, drag.mode)
    }
}

/// One owner handles scrolling, flipping and pull reversal, without transferring capture.
#[hook]
pub(crate) fn use_review_gestures(
    mode: GestureMode,
    send: Callback<ChatAction>,
    scroll: ConversationScroll,
) -> ReviewGestures {
    let node = use_node_ref();
    let motion = use_mut_ref(|| crate::drag_motion::DragMotion::new(node.clone()));
    let pointer = use_mut_ref(|| None::<ActiveDrag>);
    let suppress_click = use_mut_ref(|| false);
    let hint_node = use_node_ref();
    let hint = use_mut_ref(|| crate::gesture_hint::GestureHint::new(hint_node.clone()));
    {
        let pointer = pointer.clone();
        let motion = motion.clone();
        let hint = hint.clone();
        let scroll = scroll.clone();
        use_effect_with_deps(
            move |_| {
                *pointer.borrow_mut() = None;
                hint.borrow_mut().paint(ReviewHint::default(), false);
                crate::drag_motion::settle_motion(&motion);
                scroll.update_preview.emit(0.0);
                crate::scroll_motion::cancel_scroll(&scroll.motion);
                move || {
                    crate::drag_motion::settle_motion(&motion);
                    crate::scroll_motion::cancel_scroll(&scroll.motion);
                }
            },
            mode,
        );
    }
    let toggle = {
        let send = send.clone();
        let suppress = suppress_click.clone();
        Callback::from(move |_| {
            if !std::mem::take(&mut *suppress.borrow_mut()) {
                send.emit(ChatAction::ToggleEnglish);
            }
        })
    };
    let tap_start = {
        let suppress = suppress_click.clone();
        Callback::from(move |event: PointerEvent| {
            if event.is_primary() {
                *suppress.borrow_mut() = false;
            }
        })
    };
    let onpointerdown = {
        let pointer = pointer.clone();
        let suppress = suppress_click.clone();
        let scroll = scroll.clone();
        Callback::from(move |event: PointerEvent| {
            let control = event
                .target()
                .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                .is_some_and(|e| {
                    e.closest(".chat-header, .chat-toolbar, .chat-controls, .chat-notice")
                        .ok()
                        .flatten()
                        .is_some()
                });
            if mode == GestureMode::Disabled
                || control
                || !event.is_primary()
                || event.button() != 0
                || pointer.borrow().is_some()
            {
                return;
            }
            let notes = notes_element(&event).filter(|e| e.scroll_height() > e.client_height());
            let gesture_mode = notes_gesture_mode(mode, notes.is_some());
            let in_notes = notes.is_some();
            let pane = if gesture_mode == GestureMode::Flip {
                notes.or_else(|| scroll.timeline.cast::<HtmlElement>())
            } else {
                None
            };
            let stopped_glide = crate::scroll_motion::is_gliding(&scroll.motion);
            crate::scroll_motion::cancel_scroll(&scroll.motion);
            scroll.update_preview.emit(0.0);
            if let Some(element) = pane.as_ref() {
                crate::scroll_motion::target_scroll(&scroll.motion, element.clone());
            }
            let x = f64::from(event.client_x());
            let y = f64::from(event.client_y());
            *suppress.borrow_mut() = stopped_glide;
            *pointer.borrow_mut() = Some(ActiveDrag {
                id: event.pointer_id(),
                x,
                y,
                mode: gesture_mode,
                scroll: pane.map(|_| crate::scroll_drag::ScrollDrag::new(x, y, event.time_stamp())),
                pull: BottomPull::default(),
                allow_advance: !in_notes,
                touch: event.pointer_type() == "touch",
            });
            if let Some(element) = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
            {
                let _ = element.set_pointer_capture(event.pointer_id());
            }
        })
    };
    let onpointermove = {
        let pointer = pointer.clone();
        let motion = motion.clone();
        let suppress = suppress_click.clone();
        let hint = hint.clone();
        let scroll = scroll.clone();
        Callback::from(move |event: PointerEvent| {
            let mut active = pointer.borrow_mut();
            let Some(drag) = active.as_mut().filter(|drag| drag.id == event.pointer_id()) else {
                return;
            };
            if (f64::from(event.client_x()) - drag.x)
                .abs()
                .max((f64::from(event.client_y()) - drag.y).abs())
                > 6.0
            {
                *suppress.borrow_mut() = true;
            }
            let feedback = move_active_drag(drag, &event, &scroll, &motion);
            hint.borrow_mut()
                .paint(feedback, drag.pull.progress() >= 1.0);
        })
    };
    let onpointerup = {
        let pointer = pointer.clone();
        let motion = motion.clone();
        let send = send.clone();
        let hint = hint.clone();
        let scroll = scroll.clone();
        Callback::from(move |event: PointerEvent| {
            if pointer
                .borrow()
                .as_ref()
                .is_none_or(|drag| drag.id != event.pointer_id())
            {
                return;
            }
            let Some(mut drag) = pointer.borrow_mut().take() else {
                return;
            };
            let result = move_active_drag(&mut drag, &event, &scroll, &motion);
            crate::drag_motion::settle_motion(&motion);
            crate::scroll_motion::flush_scroll(&scroll.motion);
            hint.borrow_mut().paint(ReviewHint::default(), false);
            scroll.update_preview.emit(0.0);
            if let Some(route) = &drag.scroll {
                // One release performs one action: an armed horizontal flip takes priority.
                if let Some(action) = result.action {
                    send.emit(action);
                } else {
                    let pulled = drag.pull.progress() > 0.0;
                    if drag.pull.finish(drag.allow_advance && scroll.can_advance) {
                        send.emit(ChatAction::Next);
                    } else if drag.touch && !pulled {
                        crate::scroll_motion::glide_scroll(
                            &scroll.motion,
                            route.release_velocity(event.time_stamp()),
                        );
                    }
                }
            } else if let Some(action) = result.action {
                send.emit(action);
            }
        })
    };
    let cancel = {
        let pointer = pointer.clone();
        let motion = motion.clone();
        let suppress = suppress_click.clone();
        let hint = hint.clone();
        let scroll = scroll.clone();
        Callback::from(move |event: PointerEvent| {
            if pointer
                .borrow()
                .as_ref()
                .is_some_and(|drag| drag.id == event.pointer_id())
            {
                *pointer.borrow_mut() = None;
                *suppress.borrow_mut() = false;
                crate::drag_motion::settle_motion(&motion);
                crate::scroll_motion::cancel_scroll(&scroll.motion);
                hint.borrow_mut().paint(ReviewHint::default(), false);
                scroll.update_preview.emit(0.0);
            }
        })
    };
    ReviewGestures {
        motion: node,
        hint: hint_node,
        onpointerdown,
        onpointermove,
        onpointerup,
        cancel,
        tap: toggle,
        tap_start,
    }
}
