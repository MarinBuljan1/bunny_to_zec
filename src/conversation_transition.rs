//! Temporary DOM snapshots make conversation changes feel like continued scrolling.
//! Yew owns the live timeline; the snapshot host contains only inert, short-lived clones.
use gloo_timers::callback::Timeout;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::HtmlElement;
use yew::prelude::*;

pub(crate) struct ConversationTransition {
    pub outgoing: NodeRef,
    pub capture: Callback<()>,
}

#[hook]
pub(crate) fn use_conversation_transition(
    timeline: NodeRef,
    turn: Option<(String, usize)>,
) -> ConversationTransition {
    let outgoing = use_node_ref();
    let pending = use_mut_ref(|| None::<HtmlElement>);
    {
        let timeline = timeline.clone();
        let pending = pending.clone();
        use_effect_with_deps(
            move |_| {
                let animation = pending.borrow_mut().take().and_then(|snapshot| {
                    let Some(timeline) = timeline.cast::<HtmlElement>() else {
                        snapshot.remove();
                        return None;
                    };
                    let started = snapshot
                        .set_attribute("data-conversation-exit", "true")
                        .and_then(|_| timeline.set_attribute("data-conversation-enter", "true"));
                    if let Err(error) = started {
                        // Optional animation failure must not block the next conversation.
                        web_sys::console::error_2(
                            &"Could not animate conversation change".into(),
                            &error,
                        );
                        finish_transition(&snapshot, &timeline);
                        return None;
                    }
                    let old = snapshot.clone();
                    let live = timeline.clone();
                    let timer = Timeout::new(380, move || finish_transition(&old, &live));
                    Some((timer, snapshot, timeline))
                });
                move || {
                    if let Some((timer, snapshot, timeline)) = animation {
                        drop(timer);
                        finish_transition(&snapshot, &timeline);
                    }
                }
            },
            turn,
        );
    }
    let capture = {
        let outgoing = outgoing.clone();
        Callback::from(move |_| {
            let reduced_motion = web_sys::window()
                .and_then(|window| {
                    window
                        .match_media("(prefers-reduced-motion: reduce)")
                        .ok()
                        .flatten()
                })
                .is_some_and(|query| query.matches());
            if reduced_motion {
                return;
            }
            let (Some(timeline), Some(host)) = (
                timeline.cast::<HtmlElement>(),
                outgoing.cast::<HtmlElement>(),
            ) else {
                return;
            };
            match snapshot_timeline(&timeline, &host) {
                Ok(snapshot) => {
                    if let Some(previous) = pending.borrow_mut().replace(snapshot) {
                        previous.remove();
                    }
                }
                Err(error) => web_sys::console::error_2(
                    &"Could not capture outgoing conversation".into(),
                    &error,
                ),
            }
        })
    };
    ConversationTransition { outgoing, capture }
}

fn snapshot_timeline(timeline: &HtmlElement, host: &HtmlElement) -> Result<HtmlElement, JsValue> {
    let snapshot = timeline
        .clone_node_with_deep(true)?
        .dyn_into::<HtmlElement>()?;
    snapshot.set_attribute("class", "chat-timeline conversation-snapshot")?;
    snapshot.remove_attribute("tabindex")?;
    host.append_child(&snapshot)?;
    snapshot.set_scroll_top(timeline.scroll_top());
    Ok(snapshot)
}

fn finish_transition(snapshot: &HtmlElement, timeline: &HtmlElement) {
    snapshot.remove();
    // Cleanup is best effort; the snapshot is already removed and the live UI remains usable.
    let _ = timeline.remove_attribute("data-conversation-enter");
}
