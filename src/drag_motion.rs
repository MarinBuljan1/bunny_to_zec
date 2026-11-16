//! Frame-owned visual updates keep pointer movement out of the conversation renderer.
use crate::conversations::{ChatAction, Rating};
use crate::gestures::{resisted_offset, swipe_feedback, GestureMode};
use gloo_render::{request_animation_frame, AnimationFrame};
use std::cell::RefCell;
use std::rc::Rc;
use web_sys::HtmlElement;
use yew::NodeRef;

pub(crate) struct DragMotion {
    node: NodeRef,
    latest: (f64, f64, GestureMode),
    frame: Option<AnimationFrame>,
}

impl DragMotion {
    pub fn new(node: NodeRef) -> Self {
        Self {
            node,
            latest: (0.0, 0.0, GestureMode::Disabled),
            frame: None,
        }
    }
}

/// Coalesce high-frequency pointer events into one visual update per browser frame.
pub(crate) fn queue_motion(motion: &Rc<RefCell<DragMotion>>, dx: f64, dy: f64, mode: GestureMode) {
    {
        let mut motion = motion.borrow_mut();
        motion.latest = (dx, dy, mode);
        if motion.frame.is_some() {
            return;
        }
    }
    // The frame must not keep its owner alive after the view unmounts.
    let owner = Rc::downgrade(motion);
    let frame = request_animation_frame(move |_| {
        if let Some(owner) = owner.upgrade() {
            let mut motion = owner.borrow_mut();
            motion.frame.take();
            draw_motion(&motion, false);
        }
    });
    motion.borrow_mut().frame = Some(frame);
}

/// Cancel pending movement before settling, on release, cancellation or view teardown.
pub(crate) fn settle_motion(motion: &Rc<RefCell<DragMotion>>) {
    let mut motion = motion.borrow_mut();
    motion.frame.take();
    draw_motion(&motion, true);
}

fn draw_motion(motion: &DragMotion, settling: bool) {
    let Some(element) = motion.node.cast::<HtmlElement>() else {
        return;
    };
    let (dx, dy, mode) = motion.latest;
    let feedback = swipe_feedback(dx, dy, mode);
    let color = match feedback.action {
        Some(ChatAction::Rate(Rating::Correct)) => "34, 197, 94",
        Some(ChatAction::Rate(_)) => "239, 68, 68",
        Some(ChatAction::Flip) => "59, 130, 246",
        _ => "88, 101, 122",
    };
    let (x, y, opacity, duration) = if settling {
        (0.0, 0.0, 0.0, 170)
    } else {
        (
            resisted_offset(dx),
            if mode == GestureMode::Rate {
                resisted_offset(dy)
            } else {
                0.0
            },
            feedback.strength * 0.45,
            0,
        )
    };
    // This hook owns the active wrapper's inline visual properties; Yew owns its contents.
    let style = format!("--drag-x: {x}px; --drag-y: {y}px; --drag-duration: {duration}ms; --glow-rgb: {color}; --glow-opacity: {opacity}");
    if let Err(error) = element.set_attribute("style", &style) {
        // Failure leaves the message usable through keyboard controls.
        web_sys::console::error_2(&"Could not update message movement".into(), &error);
    }
}
