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
                        <p>{"Swipe up or down to start or skip a question. Use Reveal / flip (or swipe left or right) when you recall the word; this stops the recall timer immediately."}</p>
                        <p>{"After revealing, choose Correct (swipe up) or Incorrect (swipe down) to rate your answer and continue."}</p>
                        <p>{"Keyboard: Tab to a control and press Enter or Space. Focus the card area and use Left/Right to reveal or flip, Up for correct/next, and Down for incorrect/next. Escape closes this guide."}</p>
                        <p>{"Each word’s progress blends how long since you last saw it, how quickly you recalled it, and whether you got it right. Scores decrease over time. Quick recall after longer breaks counts toward learning."}</p>
                        <p>{"Install the app using Add to Home Screen in Safari’s Share menu, or Install / Add to Home screen in Chrome’s menu. Offline use is available after the first successful online load."}</p>
                    </div>
                </div>
            </div>
        </dialog>
    }
}
