//! Frame-owned scrolling, touch-release momentum, and pull-preview painting.
use gloo_render::{request_animation_frame, AnimationFrame};
use std::cell::RefCell;
use std::rc::Rc;
use web_sys::HtmlElement;
use yew::NodeRef;

pub(crate) struct ScrollMotion {
    element: Option<HtmlElement>,
    delta: i32,
    frame: Option<AnimationFrame>,
    velocity: f64,
    time: Option<f64>,
    remainder: f64,
}

impl ScrollMotion {
    pub fn new() -> Self {
        Self {
            element: None,
            delta: 0,
            frame: None,
            velocity: 0.0,
            time: None,
            remainder: 0.0,
        }
    }
}

pub(crate) fn target_scroll(motion: &Rc<RefCell<ScrollMotion>>, element: HtmlElement) {
    cancel_scroll(motion);
    motion.borrow_mut().element = Some(element);
}

pub(crate) fn is_gliding(motion: &Rc<RefCell<ScrollMotion>>) -> bool {
    motion.borrow().velocity.abs() > 0.02
}

/// Gesture decisions use the queued position too, before the next paint applies it.
pub(crate) fn at_bottom_after_pending(motion: &Rc<RefCell<ScrollMotion>>) -> bool {
    let motion = motion.borrow();
    let Some(element) = &motion.element else {
        return false;
    };
    let top = element.scroll_top().saturating_add(motion.delta).max(0);
    i64::from(element.scroll_height()) - i64::from(top) - i64::from(element.client_height()) <= 8
}

pub(crate) fn queue_scroll(motion: &Rc<RefCell<ScrollMotion>>, delta: i32) {
    {
        let mut motion = motion.borrow_mut();
        motion.delta = motion.delta.saturating_add(delta);
    }
    schedule_scroll(motion);
}

fn schedule_scroll(motion: &Rc<RefCell<ScrollMotion>>) {
    if motion.borrow().frame.is_some() {
        return;
    }
    let owner = Rc::downgrade(motion);
    let frame = request_animation_frame(move |time| {
        if let Some(owner) = owner.upgrade() {
            let continuing = {
                let mut motion = owner.borrow_mut();
                motion.frame.take();
                let dt = motion.time.map(|previous| time - previous).unwrap_or(16.0);
                if dt > 80.0 {
                    motion.velocity = 0.0;
                }
                let dt = dt.clamp(1.0, 32.0);
                motion.time = Some(time);
                motion.remainder += motion.velocity * dt;
                let glide = motion.remainder.round() as i32;
                motion.remainder -= f64::from(glide);
                motion.delta = motion.delta.saturating_add(glide);
                let moved = apply_scroll(&mut motion);
                motion.velocity *= (-dt / 240.0).exp();
                if !moved && glide != 0 {
                    motion.velocity = 0.0;
                }
                motion.velocity.abs() > 0.02
            };
            if continuing {
                schedule_scroll(&owner);
            }
        }
    });
    motion.borrow_mut().frame = Some(frame);
}

/// Apply the final mouse displacement before accepting a release.
pub(crate) fn flush_scroll(motion: &Rc<RefCell<ScrollMotion>>) {
    let mut motion = motion.borrow_mut();
    motion.frame.take();
    apply_scroll(&mut motion);
    motion.velocity = 0.0;
    motion.time = None;
    motion.remainder = 0.0;
}

fn apply_scroll(motion: &mut ScrollMotion) -> bool {
    let delta = std::mem::take(&mut motion.delta);
    let Some(element) = &motion.element else {
        return false;
    };
    let previous = element.scroll_top();
    if delta != 0 {
        element.set_scroll_top(previous.saturating_add(delta));
    }
    element.scroll_top() != previous
}

/// Touch scrolling glides after release, without allowing momentum to trigger Next.
pub(crate) fn glide_scroll(motion: &Rc<RefCell<ScrollMotion>>, velocity: f64) {
    flush_scroll(motion);
    motion.borrow_mut().velocity = velocity;
    if velocity.abs() > 0.02 {
        schedule_scroll(motion);
    }
}

pub(crate) fn cancel_scroll(motion: &Rc<RefCell<ScrollMotion>>) {
    let mut motion = motion.borrow_mut();
    motion.frame.take();
    motion.delta = 0;
    motion.velocity = 0.0;
    motion.time = None;
    motion.remainder = 0.0;
}

pub(crate) struct PullPreview {
    node: NodeRef,
    timeline: NodeRef,
    progress: f64,
    frame: Option<AnimationFrame>,
}

impl PullPreview {
    pub fn new(node: NodeRef, timeline: NodeRef) -> Self {
        Self {
            node,
            timeline,
            progress: 0.0,
            frame: None,
        }
    }
}

pub(crate) fn queue_preview(preview: &Rc<RefCell<PullPreview>>, progress: f64) {
    {
        let mut preview = preview.borrow_mut();
        let progress = progress.clamp(0.0, 1.0);
        if preview.progress == progress {
            return;
        }
        preview.progress = progress;
        if preview.frame.is_some() {
            return;
        }
    }
    let owner = Rc::downgrade(preview);
    let frame = request_animation_frame(move |_| {
        if let Some(owner) = owner.upgrade() {
            let mut preview = owner.borrow_mut();
            preview.frame.take();
            paint_preview(&preview);
        }
    });
    preview.borrow_mut().frame = Some(frame);
}

pub(crate) fn reset_preview(preview: &Rc<RefCell<PullPreview>>) {
    let mut preview = preview.borrow_mut();
    preview.frame.take();
    preview.progress = 0.0;
    paint_preview(&preview);
}

fn paint_preview(preview: &PullPreview) {
    let Some(element) = preview.node.cast::<HtmlElement>() else {
        return;
    };
    let Some(dot) = element.first_element_child() else {
        return;
    };
    let progress = preview.progress;
    let timeline = preview.timeline.cast::<HtmlElement>();
    let scroll_before = timeline
        .as_ref()
        .map(|timeline| (timeline.scroll_height(), timeline.scroll_top()));
    let (space_style, dot_style) = preview_styles(progress);
    // Failed visual updates leave keyboard advancement and the review state usable.
    let result = element
        .set_attribute("style", &space_style)
        .and_then(|_| dot.set_attribute("style", &dot_style))
        .and_then(|_| {
            element.set_attribute(
                "aria-hidden",
                if progress == 0.0 { "true" } else { "false" },
            )
        })
        .and_then(|_| {
            element.set_attribute(
                "aria-label",
                if progress >= 1.0 {
                    "Release for next messages"
                } else {
                    "Pull for next messages"
                },
            )
        });
    if let (Some(timeline), Some((height, top))) = (timeline, scroll_before) {
        let height_change = timeline.scroll_height() - height;
        if height_change != 0 {
            // Keep the same distance from the bottom as the preview resizes. Using
            // the previous top also preserves reverse-scroll deltas queued this frame.
            timeline.set_scroll_top(top.saturating_add(height_change));
        }
    }
    if let Err(error) = result {
        web_sys::console::error_2(&"Could not update the next-message dot".into(), &error);
    }
}

/// Set concrete dimensions on the DOM nodes rather than relying on CSS-variable
/// arithmetic to give an otherwise zero-sized preview its visible size.
fn preview_styles(progress: f64) -> (String, String) {
    let space = 52.0 * progress;
    let diameter = 36.0 * progress;
    (
        format!("height: {space}px"),
        format!("width: {diameter}px; height: {diameter}px; opacity: {progress}"),
    )
}

#[cfg(test)]
#[path = "tests/scroll_motion_tests.rs"]
mod tests;
