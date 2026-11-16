//! Native modal dialog supplies focus containment and makes the app behind it inert.

use wasm_bindgen::JsCast;
use web_sys::{window, HtmlDialogElement, HtmlElement};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub(super) struct GuideProps {
    pub on_close: Callback<()>,
}

#[function_component(Guide)]
pub(super) fn guide(props: &GuideProps) -> Html {
    let dialog = use_node_ref();
    {
        let dialog = dialog.clone();
        let on_close = props.on_close.clone();
        use_effect_with_deps(
            move |_| {
                let opener = window()
                    .and_then(|window| window.document())
                    .and_then(|document| document.active_element())
                    .and_then(|element| element.dyn_into::<HtmlElement>().ok());
                let element = dialog.cast::<HtmlDialogElement>();
                if let Some(element) = &element {
                    if let Err(error) = element.show_modal() {
                        web_sys::console::error_2(
                            &"Could not open the guide dialog".into(),
                            &error,
                        );
                        // A failed modal must not leave the review controls locked.
                        on_close.emit(());
                    }
                }
                move || {
                    if let Some(element) = element {
                        element.close();
                    }
                    if let Some(opener) = opener {
                        if let Err(error) = opener.focus() {
                            web_sys::console::error_2(
                                &"Could not restore focus after closing the guide".into(),
                                &error,
                            );
                        }
                    }
                }
            },
            (),
        );
    }
    let close = {
        let on_close = props.on_close.clone();
        Callback::from(move |_| on_close.emit(()))
    };
    let cancel = {
        let on_close = props.on_close.clone();
        Callback::from(move |event: Event| {
            event.prevent_default();
            on_close.emit(());
        })
    };
    html! {
        <dialog ref={dialog} class="guide-dialog" aria-labelledby="guide-title"
            oncancel={cancel} onclick={close.clone()}>
            <div class="settings-modal" onclick={Callback::from(|event: MouseEvent| event.stop_propagation())}>
                <div class="modal-header">
                    <h2 id="guide-title">{"Guide"}</h2>
                    <button type="button" class="close-button" onclick={close} autofocus=true aria-label="Close guide">{"×"}</button>
                </div>
                <div class="modal-body">
                    <div class="instructions">
                        <p>{"Typing dots appear before the partner message, then before your reply. Read the partner message while your reply is being typed. The recall timer starts only when your reply appears. Recall the missing words, then use Reveal / flip or swipe left or right to stop the timer."}</p>
                        <p>{"After revealing, choose Correct (swipe up) or Incorrect (swipe down). Your message settles back into place. Scroll through earlier messages; pull up at the bottom until the next-message dot stops growing, then release, or choose Next messages to continue. Swipes can start on the conversation background. A soft green, red, or blue glow behind the moving message strengthens when you have swiped far enough. Earlier conversations disappear when a new one begins. Tap any message to toggle English for every message. Revealing keeps the tested words underlined with English hints. Flip again for word explanations beneath the reply; rating hides them, and another flip can reopen them."}</p>
                        <p>{"Keyboard: Tab to a control and press Enter or Space. Focus the conversation area and use Left/Right to reveal or flip, Up for correct or next at the bottom, and Down for incorrect. Escape closes this guide."}</p>
                        <p>{"Each word’s progress blends how long since you last saw it, how quickly you recalled it, and whether you got it right. Scores decrease over time. Quick recall after longer breaks counts toward learning."}</p>
                        <p>{"Install the app using Add to Home Screen in Safari’s Share menu, or Install / Add to Home screen in Chrome’s menu. Offline use is available after the first successful online load."}</p>
                    </div>
                </div>
            </div>
        </dialog>
    }
}
