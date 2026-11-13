//! Review-message swipes and native history scrolling with a deliberate bottom pull.
use crate::conversations::{ChatAction, Rating};
use gloo_events::{EventListener, EventListenerOptions};
use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, PointerEvent, TouchEvent, WheelEvent};
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

#[derive(Default)]
struct BottomPull {
    distance: f64,
}
impl BottomPull {
    fn move_by(&mut self, delta: f64, enabled: bool, bottom: bool) -> bool {
        self.distance = if enabled && bottom {
            (self.distance + delta).max(0.0)
        } else {
            0.0
        };
        self.distance > 0.0
    }
    fn finish(&mut self, enabled: bool) -> bool {
        let distance = std::mem::take(&mut self.distance);
        enabled && distance >= 72.0
    }
}

/// Listeners keep native touch scrolling until the reader pulls beyond the bottom.
#[hook]
pub(crate) fn use_conversation_scroll(
    timeline: NodeRef,
    mounted: bool,
    advance: Option<Callback<ChatAction>>,
) {
    let latest_advance = use_mut_ref(|| advance.clone());
    *latest_advance.borrow_mut() = advance;
    use_effect_with_deps(
        move |mounted| {
            let mut listeners = Vec::new();
            if let Some(element) = mounted.then(|| timeline.cast::<HtmlElement>()).flatten() {
                let last_y = Rc::new(Cell::new(None::<i32>));
                let pull = Rc::new(std::cell::RefCell::new(BottomPull::default()));
                let y = last_y.clone();
                let distance = pull.clone();
                listeners.push(EventListener::new(&element, "touchstart", move |event| {
                    y.set(
                        event
                            .dyn_ref::<TouchEvent>()
                            .filter(|e| e.touches().length() == 1)
                            .and_then(|e| e.touches().item(0))
                            .map(|t| t.client_y()),
                    );
                    distance.borrow_mut().finish(false);
                }));
                let y = last_y.clone();
                let distance = pull.clone();
                let callback = latest_advance.clone();
                let scroll = element.clone();
                listeners.push(EventListener::new_with_options(
                    &element,
                    "touchmove",
                    EventListenerOptions::enable_prevent_default(),
                    move |event| {
                        let Some(event) = event.dyn_ref::<TouchEvent>() else {
                            return;
                        };
                        if event.touches().length() != 1 {
                            y.set(None);
                            distance.borrow_mut().finish(false);
                            return;
                        }
                        let Some(touch) = event.touches().item(0) else {
                            return;
                        };
                        let current = touch.client_y();
                        let previous = y.replace(Some(current)).unwrap_or(current);
                        if distance.borrow_mut().move_by(
                            f64::from(previous - current),
                            callback.borrow().is_some(),
                            at_bottom(&scroll),
                        ) {
                            event.prevent_default();
                        }
                    },
                ));
                let distance = pull.clone();
                let callback = latest_advance.clone();
                listeners.push(EventListener::new(&element, "touchend", move |_| {
                    if distance.borrow_mut().finish(callback.borrow().is_some()) {
                        if let Some(callback) = callback.borrow().as_ref() {
                            callback.emit(ChatAction::Next);
                        }
                    }
                }));
                listeners.push(EventListener::new(&element, "touchcancel", move |_| {
                    last_y.set(None);
                    pull.borrow_mut().finish(false);
                }));

                // Mouse dragging scrolls history too; active review bubbles own their gestures.
                let drag = Rc::new(Cell::new(None::<(i32, i32)>));
                let distance = Rc::new(std::cell::RefCell::new(BottomPull::default()));
                let pointer = drag.clone();
                let pull = distance.clone();
                listeners.push(EventListener::new(&element, "pointerdown", move |event| {
                    let Some(event) = event.dyn_ref::<PointerEvent>() else {
                        return;
                    };
                    let reviewing = event
                        .target()
                        .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                        .is_some_and(|e| e.closest(".reviewing").ok().flatten().is_some());
                    if event.pointer_type() == "mouse"
                        && event.is_primary()
                        && event.button() == 0
                        && !reviewing
                    {
                        pointer.set(Some((event.pointer_id(), event.client_y())));
                        pull.borrow_mut().finish(false);
                    }
                }));
                let pointer = drag.clone();
                let pull = distance.clone();
                let scroll = element.clone();
                let callback = latest_advance.clone();
                listeners.push(EventListener::new(&element, "pointermove", move |event| {
                    let Some(event) = event.dyn_ref::<PointerEvent>() else {
                        return;
                    };
                    let Some((id, y)) = pointer.get().filter(|(id, _)| *id == event.pointer_id())
                    else {
                        return;
                    };
                    let delta = y - event.client_y();
                    if delta.abs() < 3 {
                        return;
                    }
                    let bottom = at_bottom(&scroll);
                    pull.borrow_mut().move_by(
                        f64::from(delta),
                        callback.borrow().is_some(),
                        bottom,
                    );
                    scroll.set_scroll_top(scroll.scroll_top() + delta);
                    pointer.set(Some((id, event.client_y())));
                    // Capture only after movement so a simple tap reaches its message button.
                    let _ = scroll.set_pointer_capture(id);
                }));
                let pointer = drag.clone();
                let pull = distance.clone();
                let callback = latest_advance.clone();
                listeners.push(EventListener::new(&element, "pointerup", move |event| {
                    if event.dyn_ref::<PointerEvent>().is_some_and(|event| {
                        pointer
                            .get()
                            .is_some_and(|(id, _)| id == event.pointer_id())
                    }) {
                        pointer.set(None);
                        if pull.borrow_mut().finish(callback.borrow().is_some()) {
                            if let Some(callback) = callback.borrow().as_ref() {
                                callback.emit(ChatAction::Next);
                            }
                        }
                    }
                }));
                listeners.push(EventListener::new(
                    &element,
                    "lostpointercapture",
                    move |_| {
                        drag.set(None);
                        distance.borrow_mut().finish(false);
                    },
                ));
                let accumulated = Cell::new(0.0_f64);
                let last_wheel = Cell::new(0.0_f64);
                let callback = latest_advance.clone();
                let scroll = element.clone();
                listeners.push(EventListener::new_with_options(
                    &element,
                    "wheel",
                    EventListenerOptions::enable_prevent_default(),
                    move |event| {
                        let Some(event) = event.dyn_ref::<WheelEvent>() else {
                            return;
                        };
                        if callback.borrow().is_some()
                            && at_bottom(&scroll)
                            && event.delta_y() > 0.0
                        {
                            event.prevent_default();
                            if event.time_stamp() - last_wheel.get() > 500.0 {
                                accumulated.set(0.0);
                            }
                            last_wheel.set(event.time_stamp());
                            let pixels = match event.delta_mode() {
                                1 => event.delta_y() * 16.0,
                                2 => event.delta_y() * f64::from(scroll.client_height()),
                                _ => event.delta_y(),
                            };
                            accumulated.set(accumulated.get() + pixels);
                            if accumulated.get() >= 120.0 {
                                accumulated.set(0.0);
                                if let Some(callback) = callback.borrow().as_ref() {
                                    callback.emit(ChatAction::Next);
                                }
                            }
                        } else {
                            accumulated.set(0.0);
                        }
                    },
                ));
            }
            move || drop(listeners)
        },
        mounted,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
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
    pub offset: (f64, f64),
    pub onpointerdown: Callback<PointerEvent>,
    pub onpointermove: Callback<PointerEvent>,
    pub onpointerup: Callback<PointerEvent>,
    pub cancel: Callback<PointerEvent>,
    pub tap: Callback<MouseEvent>,
    pub tap_start: Callback<PointerEvent>,
}

#[hook]
pub(crate) fn use_review_gestures(blocked: bool, send: Callback<ChatAction>) -> ReviewGestures {
    let offset = use_state(|| (0.0_f64, 0.0_f64));
    let pointer = use_mut_ref(|| None::<(i32, f64, f64)>);
    let suppress_click = use_mut_ref(|| false);
    {
        let pointer = pointer.clone();
        let offset = offset.clone();
        use_effect_with_deps(
            move |blocked| {
                if *blocked {
                    *pointer.borrow_mut() = None;
                    offset.set((0.0, 0.0));
                }
                || ()
            },
            blocked,
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

        Callback::from(move |event: PointerEvent| {
            if blocked || !event.is_primary() || event.button() != 0 || pointer.borrow().is_some() {
                return;
            }
            *suppress.borrow_mut() = false;
            *pointer.borrow_mut() = Some((
                event.pointer_id(),
                f64::from(event.client_x()),
                f64::from(event.client_y()),
            ));
            // Capture the original element so a tap still clicks its message button.
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
        let offset = offset.clone();
        let suppress = suppress_click.clone();
        Callback::from(move |event: PointerEvent| {
            if let Some((id, x, y)) = *pointer.borrow() {
                if event.pointer_id() != id {
                    return;
                }
                let dx = f64::from(event.client_x()) - x;
                let dy = f64::from(event.client_y()) - y;
                if dx.abs().max(dy.abs()) > 6.0 {
                    *suppress.borrow_mut() = true;
                }
                offset.set((dx.clamp(-32.0, 32.0), dy.clamp(-32.0, 32.0)));
            }
        })
    };
    let onpointerup = {
        let pointer = pointer.clone();
        let offset = offset.clone();
        let send = send.clone();
        Callback::from(move |event: PointerEvent| {
            let Some((id, x, y)) = *pointer.borrow() else {
                return;
            };
            if event.pointer_id() != id {
                return;
            }
            *pointer.borrow_mut() = None;
            offset.set((0.0, 0.0));
            let dx = f64::from(event.client_x()) - x;
            let dy = f64::from(event.client_y()) - y;
            if let Some(action) = review_swipe(dx, dy) {
                send.emit(action);
            }
        })
    };
    let cancel = {
        let pointer = pointer.clone();
        let offset = offset.clone();
        let suppress = suppress_click.clone();
        Callback::from(move |event: PointerEvent| {
            if pointer
                .borrow()
                .is_some_and(|(id, _, _)| id == event.pointer_id())
            {
                *pointer.borrow_mut() = None;
                offset.set((0.0, 0.0));
                *suppress.borrow_mut() = false;
            }
        })
    };
    ReviewGestures {
        offset: *offset,
        onpointerdown,
        onpointermove,
        onpointerup,
        cancel,
        tap: toggle,
        tap_start,
    }
}
