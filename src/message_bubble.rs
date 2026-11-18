//! Typing bubbles resize; info flips use transform/opacity without per-frame layout.
use gloo_timers::callback::Timeout;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::HtmlElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub(crate) struct MessageBubbleProps {
    pub class: Classes,
    pub label: &'static str,
    pub typing: bool,
    pub layout_key: String,
    #[prop_or_default]
    pub start_size: Option<(f64, f64)>,
    pub on_resized: Callback<()>,
    pub children: Children,
}

#[function_component(MessageBubble)]
pub(crate) fn message_bubble(props: &MessageBubbleProps) -> Html {
    let node = use_node_ref();
    let previous_size = use_mut_ref(|| None::<(f64, f64)>);
    let previous_typing = use_mut_ref(|| None::<bool>);
    {
        let node = node.clone();
        let resized = props.on_resized.clone();
        let start_size = props.start_size;
        let typing = props.typing;
        use_effect_with_deps(
            move |_| {
                let mut follow_growth = false;
                let animation = node.cast::<HtmlElement>().and_then(|element| {
                    let rect = element.get_bounding_client_rect();
                    let size = (rect.width(), rect.height());
                    let previous = previous_size.borrow_mut().replace(size);
                    let old_typing = previous_typing.borrow_mut().replace(typing);
                    let transition = match (old_typing, previous.or(start_size)) {
                        (None, Some(from)) if start_size.is_some() => {
                            BubbleTransition::Dot { from, to: size }
                        }
                        (None, _) => BubbleTransition::Enter,
                        (Some(old), Some(from)) if old != typing => {
                            BubbleTransition::Typing { from, to: size }
                        }
                        _ => BubbleTransition::Info,
                    };
                    follow_growth = transition != BubbleTransition::Info;
                    let reduce_motion = web_sys::window()
                        .and_then(|w| {
                            w.match_media("(prefers-reduced-motion: reduce)")
                                .ok()
                                .flatten()
                        })
                        .is_some_and(|query| query.matches());
                    if reduce_motion {
                        return None;
                    }
                    animate_transition(&element, transition)
                });
                let timer = if !follow_growth {
                    None
                } else if animation.is_some() {
                    Some(Timeout::new(170, move || resized.emit(())))
                } else {
                    resized.emit(());
                    None
                };
                move || {
                    drop(timer);
                    if let Some(animation) = animation {
                        if let Some(element) = node.cast::<HtmlElement>() {
                            let rect = element.get_bounding_client_rect();
                            *previous_size.borrow_mut() = Some((rect.width(), rect.height()));
                        }
                        cancel_animation(&animation);
                    }
                }
            },
            props.layout_key.clone(),
        );
    }
    html! { <article ref={node} class={classes!("chat-message", props.class.clone())} aria-label={props.label}>{for props.children.iter()}</article> }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum BubbleTransition {
    Enter,
    Dot { from: (f64, f64), to: (f64, f64) },
    Typing { from: (f64, f64), to: (f64, f64) },
    Info,
}

fn animation_frames(transition: BubbleTransition) -> serde_json::Value {
    match transition {
        BubbleTransition::Dot {
            from: previous,
            to: next,
        } => serde_json::json!([
            { "offset": 0, "width": format!("{}px", previous.0), "height": format!("{}px", previous.1), "borderRadius": "50%", "opacity": 1 },
            { "offset": 1, "width": format!("{}px", next.0), "height": format!("{}px", next.1), "borderRadius": "18px 18px 18px 4px", "opacity": 1 }
        ]),
        BubbleTransition::Typing {
            from: previous,
            to: next,
        } => serde_json::json!([
            { "offset": 0, "width": format!("{}px", previous.0), "height": format!("{}px", previous.1), "transform": "scale(1)", "opacity": 1 },
            { "offset": 0.2, "width": format!("{}px", previous.0), "height": format!("{}px", previous.1), "transform": "scale(0.97)", "opacity": 0.65 },
            { "offset": 1, "width": format!("{}px", next.0), "height": format!("{}px", next.1), "transform": "scale(1)", "opacity": 1 }
        ]),
        BubbleTransition::Enter => serde_json::json!([
            { "transform": "scale(0.95)", "opacity": 0 },
            { "transform": "scale(1)", "opacity": 1 }
        ]),
        BubbleTransition::Info => serde_json::json!([
            { "transform": "scale(0.99)", "opacity": 0.85 },
            { "transform": "scale(1)", "opacity": 1 }
        ]),
    }
}

fn animate_transition(element: &HtmlElement, transition: BubbleTransition) -> Option<JsValue> {
    let frames = animation_frames(transition);
    // A failed animation setup leaves the normally rendered message fully usable.
    let frames = js_sys::JSON::parse(&frames.to_string())
        .ok()?
        .dyn_into::<js_sys::Object>()
        .ok()?;
    // These bindings require an unstable flag in the pinned web-sys version;
    // call the standard browser API here without enabling unstable crate APIs.
    let animate = js_sys::Reflect::get(element, &"animate".into())
        .ok()?
        .dyn_into::<js_sys::Function>()
        .ok()?;
    let options =
        js_sys::JSON::parse(r#"{"duration":160,"easing":"cubic-bezier(0.2, 0.8, 0.2, 1)"}"#)
            .ok()?;
    animate.call2(element, &frames, &options).ok()
}

#[cfg(test)]
#[path = "tests/message_bubble_tests.rs"]
mod tests;

fn cancel_animation(animation: &JsValue) {
    if let Ok(cancel) = js_sys::Reflect::get(animation, &"cancel".into())
        .and_then(|value| value.dyn_into::<js_sys::Function>())
    {
        // Cleanup is best effort; a completed animation has already released its styles.
        let _ = cancel.call0(animation);
    }
}
