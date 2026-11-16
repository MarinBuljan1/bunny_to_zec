//! Release hints update independently of Yew while a pointer is held.
use crate::conversations::{ChatAction, Rating};
use web_sys::HtmlElement;
use yew::NodeRef;

pub(crate) struct GestureHint {
    node: NodeRef,
    previous: Option<(&'static str, bool)>,
}

impl GestureHint {
    pub fn new(node: NodeRef) -> Self {
        Self {
            node,
            previous: None,
        }
    }

    pub fn paint(&mut self, action: Option<ChatAction>, next_ready: bool) {
        let instruction = release_instruction(action, next_ready);
        let active = instruction.is_some();
        let text = instruction.unwrap_or("");
        if self.previous == Some((text, active)) {
            return;
        }
        let Some(element) = self.node.cast::<HtmlElement>() else {
            return;
        };
        let Some(normal) = element.first_element_child() else {
            return;
        };
        let Some(release) = element.last_element_child() else {
            return;
        };
        release.set_text_content(Some(text));
        let result = element
            .set_attribute("data-armed", if active { "true" } else { "false" })
            .and_then(|_| {
                normal.set_attribute("aria-hidden", if active { "true" } else { "false" })
            })
            .and_then(|_| {
                release.set_attribute("aria-hidden", if active { "false" } else { "true" })
            });
        if let Err(error) = result {
            // Hints are supplementary; buttons and review acceptance remain usable.
            web_sys::console::error_2(&"Could not update the swipe instruction".into(), &error);
        } else {
            self.previous = Some((text, active));
        }
    }
}

fn release_instruction(action: Option<ChatAction>, next_ready: bool) -> Option<&'static str> {
    match action {
        Some(ChatAction::Flip) => Some("Release to flip"),
        Some(ChatAction::Rate(Rating::Correct)) => Some("Release to mark remembered"),
        Some(ChatAction::Rate(_)) => Some("Release to review again"),
        _ if next_ready => Some("Release for next messages"),
        _ => None,
    }
}

#[cfg(test)]
#[path = "tests/gesture_hint_tests.rs"]
mod tests;
