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
    /// Reverse through the visible pull before allowing the list to move away from the bottom.
    fn consume(&mut self, delta: f64, enabled: bool, bottom: bool) -> f64 {
        let previous = self.distance;
        self.distance = if enabled && (bottom || self.distance > 0.0) {
            (self.distance + delta).clamp(0.0, 72.0)
        } else {
            0.0
        };
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

/// The same feedback calculation controls both the glow and release acceptance.
fn armed_swipe(dx: f64, dy: f64, mode: GestureMode) -> Option<ChatAction> {
    let feedback = swipe_feedback(dx, dy, mode);
    feedback.action.filter(|_| feedback.armed)
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
    let preview = use_mut_ref(|| {
        crate::scroll_motion::PullPreview::new(preview_node.clone(), timeline.clone())
    });
    let motion = use_mut_ref(crate::scroll_motion::ScrollMotion::default);
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

pub(crate) struct ReviewGestures {
    pub motion: NodeRef,
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
) -> Option<ChatAction> {
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
        armed_swipe(step.horizontal, 0.0, GestureMode::Flip)
    } else {
        let dx = x - drag.x;
        let dy = y - drag.y;
        crate::drag_motion::queue_motion(motion, dx, dy, drag.mode);
        armed_swipe(dx, dy, drag.mode)
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
    {
        let pointer = pointer.clone();
        let motion = motion.clone();
        let scroll = scroll.clone();
        use_effect_with_deps(
            move |_| {
                *pointer.borrow_mut() = None;
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
                    e.closest(".chat-header, .chat-notices")
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
            move_active_drag(drag, &event, &scroll, &motion);
        })
    };
    let onpointerup = {
        let pointer = pointer.clone();
        let motion = motion.clone();
        let send = send.clone();
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
            scroll.update_preview.emit(0.0);
            if let Some(route) = &drag.scroll {
                // One release performs one action: an armed horizontal flip takes priority.
                if let Some(action) = result {
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
            } else if let Some(action) = result {
                send.emit(action);
            }
        })
    };
    let cancel = {
        let pointer = pointer.clone();
        let motion = motion.clone();
        let suppress = suppress_click.clone();
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
                scroll.update_preview.emit(0.0);
            }
        })
    };
    ReviewGestures {
        motion: node,
        onpointerdown,
        onpointermove,
        onpointerup,
        cancel,
        tap: toggle,
        tap_start,
    }
}

#[cfg(test)]
#[path = "tests/gesture_tests.rs"]
mod tests;
