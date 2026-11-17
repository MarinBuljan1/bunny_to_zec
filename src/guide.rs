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
                        {instruction_row("", "M4 4h16v12H9l-5 4V4z M8 8h8 M8 12h5", "Read the conversation", html!{
                            <p>{"Choose Start conversation. Typing dots appear before each message. Read your partner’s message, then recall the missing Croatian words in your reply. Your recall timer starts only when your reply appears."}</p>
                        })}
                        {instruction_row("", "M4 12h16 M8 8l-4 4 4 4 M16 8l4 4-4 4", "Swipe sideways to reveal", html!{
                            <p>{"Swipe left or right to fill the blanks and stop the timer. The English word hints and underlines stay visible. Flip again to show or hide word explanations beneath your reply."}</p>
                        })}
                        {instruction_row("", "M5 12l4 4L19 6", "Swipe up or down to rate", html!{
                            <p>{"After revealing, swipe up if you were correct, or down to practise again. A green or red border records your rating. You can swipe on the conversation background too; the coloured glow shows the action and grows stronger as you reach the threshold."}</p>
                        })}
                        {instruction_row("", "M12 20V4 M6 10l6-6 6 6", "Pull up to keep chatting", html!{
                            <p>{"Scroll through earlier messages. At the bottom, pull up until the next-message dot stops growing, then release. A divider marks the end of a conversation; the old conversation slides away as the next begins."}</p>
                        })}
                        {instruction_row("", "M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0z M3 12h18 M12 3c4 4 4 14 0 18-4-4-4-14 0-18z", "English on or off", html!{
                            <p>{"Tap EN in the header, or tap any message, to toggle English for all messages. A line through EN means translations are off."}</p>
                        })}
                        {instruction_row("", "M3 5h18v14H3z M6 9h1 M10 9h1 M14 9h1 M18 9h0 M6 13h1 M10 13h1 M14 13h4 M7 16h10", "Keyboard controls", html!{
                            <p>{"Tab to a control and press Enter or Space. Focus the conversation area: Left/Right reveals or flips, Up rates correct or advances at the bottom, and Down rates incorrect. Arrows inside long explanations scroll them. Escape closes this guide."}</p>
                        })}
                        {instruction_row("philosophy", "M4 4v16h16 M7 15l4-5 4 3 5-7", "Learn for real conversations", html!{
                            <p>{"Your score blends time since the last review, recall speed, and whether you were correct. Scores decrease over time. Quick recall after longer breaks counts towards learning, and the header bar fills as you approach the next level."}</p>
                        })}
                        {instruction_row("app-install", "M7 2h10v20H7z M10 5h4 M11 19h2", "Install the app", html!{
                            <>
                            <ul>
                                <li><strong>{"iPhone / iPad: "}</strong>{"In Safari, open Share, then choose Add to Home Screen and Add."}</li>
                                <li><strong>{"Android: "}</strong>{"In Chrome, open the three-dots menu and choose Install or Add to Home screen."}</li>
                            </ul>
                            <p>{"The app opens full screen and can work offline after the first successful online load."}</p>
                            </>
                        })}
                    </div>
                </div>
            </div>
        </dialog>
    }
}

/// Inline icons keep the guide available offline without an icon-font dependency.
fn instruction_row(
    modifier: &'static str,
    icon: &'static str,
    title: &'static str,
    content: Html,
) -> Html {
    html! {
        <section class={classes!("instruction-row", modifier)}>
            <span class="instruction-icon" aria-hidden="true">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"
                    stroke-linecap="round" stroke-linejoin="round" focusable="false">
                    <path d={icon} />
                </svg>
            </span>
            <div class="instruction-text"><h3>{title}</h3>{content}</div>
        </section>
    }
}
