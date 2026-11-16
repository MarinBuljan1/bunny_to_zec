use wasm_bindgen::prelude::wasm_bindgen;
use yew::prelude::*;

mod chat;
mod conversation_transition;
mod conversations;
mod drag_motion;
mod gestures;
mod guide;
mod learning;
mod message_bubble;
mod model;
mod persistence;
mod scroll_drag;
mod scroll_motion;
mod sentences;
mod storage;
mod word_notes;
use guide::Guide;

#[function_component(App)]
fn app() -> Html {
    let show_settings = use_state(|| false);

    html! {
            <div class="app-container">
                <chat::ConversationView guide_open={*show_settings} open_settings={{
                    let show_settings = show_settings.clone();
                    Callback::from(move |_| show_settings.set(true))
                }} />

                { if *show_settings { html! { <Guide on_close={{
                    let show_settings = show_settings.clone();
                    Callback::from(move |_| show_settings.set(false))
                }} /> } } else { html!{} } }
            </div>
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    yew::Renderer::<App>::new().render();
}

#[cfg(test)]
#[path = "tests/memory_store.rs"]
mod memory_store;
