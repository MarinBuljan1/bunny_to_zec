use js_sys::{Date, Math, Reflect};
use std::collections::HashMap;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsCast;
use web_sys::{window, Element, EventTarget, HtmlElement, TouchEvent};
use yew::prelude::*;

mod learning;
mod model;
mod review_state;
mod sentences;
mod storage;
use learning::{
    apply_review, calculate_progress_sum, find_key_of_lowest_score, ReviewAttempt, ReviewOutcome,
    LEARNED_INTERVAL_SECONDS,
};
use model::{Card, CurrentTranslations, LearningContent, ProcessingData, SavedProgress, ScoreData};
use review_state::{
    AnimationState, DragAction, DragState, Point, ReviewAction, ReviewKind, ReviewPresentation,
    ReviewState,
};
use sentences::{choose_sentence_index, find_related_cards, split_text};

#[derive(Clone, PartialEq, Debug)]
enum SwipeDirection {
    None,
    Up,
    Down,
    Right,
    Left,
}

#[derive(Properties, Clone, PartialEq)]
struct FlashcardProps {
    swipe_direction: SwipeDirection,
    is_moving: bool,
    pos_x: f64,
    pos_y: f64,
    fast_move: bool,
    current_card: Card,
    processed_sentence: Html,
    show_explanations: bool,
    processed_explanation: Html,
}

#[function_component(Flashcard)]
fn flashcard(props: &FlashcardProps) -> Html {
    let style = format!("--pos_x: {}; --pos_y: {};", props.pos_x, props.pos_y);

    let mut class_name = match props.swipe_direction {
        SwipeDirection::Up => "card move-up",
        SwipeDirection::Down => "card move-down",
        SwipeDirection::Right => "card move-right",
        SwipeDirection::Left => "card move-left",
        SwipeDirection::None => "card",
    }
    .to_string();

    if props.is_moving {
        class_name.push_str(" moving");
    }

    if props.fast_move {
        class_name.push_str(" fast-move");
    }

    html! {
        <>
            <div class={format!("{} front-side", class_name.clone())} style={style.clone()}>
                {
                    if props.show_explanations {
                        html! { <>{ props.processed_explanation.clone() }</> }
                    } else {
                        html! {
                            <>
                                <p class="top">{ props.processed_sentence.clone() }</p>
                                <p class="bottom">{ format!("( {} )", props.current_card.base_language.clone()) }</p>
                            </>
                        }
                    }
                }
            </div>
            <div class={format!("{} back-side", class_name.clone())} style={style.clone()}>
                <p class="top">{ props.processed_sentence.clone() }</p>
                <p class="bottom">{ format!("( {} )", props.current_card.base_language.clone()) }</p>
            </div>
        </>
    }
}

#[derive(Properties, PartialEq, Clone)]
struct FlashcardViewProps {
    open_settings: Callback<()>,
}

#[function_component(FlashcardView)]
fn flashcard_view(props: &FlashcardViewProps) -> Html {
    let content = use_state(LearningContent::default);
    let progress = use_state(|| SavedProgress {
        scores: HashMap::new(),
        last_data_update: get_timestamp_s(),
    });
    let review = use_reducer(|| ReviewState::welcome(get_timestamp_ms()));
    let drag = use_reducer(DragState::default);
    let animation = use_state(AnimationState::default);

    {
        let content = content.clone();
        let progress = progress.clone();
        use_effect_with_deps(
            move |_| {
                wasm_bindgen_futures::spawn_local(async move {
                    let loaded_content = storage::load_learning_content().await;
                    progress.set(storage::load_progress(get_timestamp_s()));
                    content.set(loaded_content);
                });
                || ()
            },
            (),
        );
    }

    let on_swipe_completed = {
        let drag = drag.clone();
        let animation = animation.clone();
        let review = review.clone();
        let content = content.clone();
        let progress = progress.clone();
        Callback::from(move |end: Point| {
            if let Some(direction) = drag.completed_direction(end, &animation.direction()) {
                animation.set(AnimationState::Swiping(direction.clone()));
                match direction {
                    SwipeDirection::Up | SwipeDirection::Down => {
                        let updated_scores = save_review_result(&progress, &review, direction);
                        animate_next_review(&content, &review, &animation, updated_scores);
                    }
                    SwipeDirection::Left | SwipeDirection::Right => {
                        if review.is_question() {
                            review.dispatch(ReviewAction::Reveal(get_timestamp_ms()));
                            let review = review.clone();
                            set_timeout(
                                move || review.dispatch(ReviewAction::ShowExplanations),
                                400,
                            );
                        }
                    }
                    SwipeDirection::None => {}
                }
            }
            drag.dispatch(DragAction::End);
        })
    };

    let on_touch_event = {
        let drag = drag.clone();
        let on_swipe_completed = on_swipe_completed.clone();
        Callback::from(move |event: TouchEvent| match event.type_().as_str() {
            "touchstart" => {
                if event
                    .target()
                    .as_ref()
                    .is_some_and(should_enable_drag_for_event_target)
                {
                    let touch = get_first_touch(&event, "touches");
                    drag.dispatch(DragAction::Start(Point {
                        x: touch.client_x() as f64,
                        y: touch.client_y() as f64,
                    }));
                }
            }
            "touchmove" if drag.is_moving() => {
                let touch = get_first_touch(&event, "touches");
                move_drag(
                    &drag,
                    Point {
                        x: touch.client_x() as f64,
                        y: touch.client_y() as f64,
                    },
                );
            }
            "touchend" if drag.is_moving() => {
                let touch = get_first_touch(&event, "changedTouches");
                on_swipe_completed.emit(Point {
                    x: touch.client_x() as f64,
                    y: touch.client_y() as f64,
                });
            }
            _ => {}
        })
    };

    let on_mouse_event = {
        let drag = drag.clone();
        Callback::from(move |event: MouseEvent| {
            event.prevent_default();
            let position = Point {
                x: event.client_x() as f64,
                y: event.client_y() as f64,
            };
            match event.type_().as_str() {
                "mousedown" => {
                    if event
                        .target()
                        .as_ref()
                        .is_some_and(should_enable_drag_for_event_target)
                    {
                        drag.dispatch(DragAction::Start(position));
                    }
                }
                "mousemove" if drag.is_moving() => move_drag(&drag, position),
                "mouseup" if drag.is_moving() => on_swipe_completed.emit(position),
                _ => {}
            }
        })
    };

    let drag_direction = drag.direction();
    let delta_xy = drag.intensity();
    let bg_color = match drag_direction {
        SwipeDirection::Up => "34, 197, 94",
        SwipeDirection::Down => "239, 68, 68",
        SwipeDirection::Right => "59, 130, 246",
        SwipeDirection::Left => "59, 130, 246",
        SwipeDirection::None => "88, 101, 122",
    }
    .to_string();
    let bg_direction = match drag_direction {
        SwipeDirection::Up => "180deg",
        SwipeDirection::Down => "0deg",
        SwipeDirection::Right => "270deg",
        SwipeDirection::Left => "90deg",
        SwipeDirection::None => "0deg",
    }
    .to_string();
    let edge_alpha = (delta_xy * 0.8).min(0.8);
    let shadow_boost = (delta_xy * 0.9).min(0.9);
    let style = format!(
        "--edge-rgb: {bg_color}; --edge-alpha: {edge_alpha:.3}; --edge-angle: {bg_direction}; --shadow-boost: {shadow_boost:.3};"
    );

    let is_initial_state = review.kind == ReviewKind::Welcome;
    let progress_sum = use_memo(
        |progress| calculate_progress_sum(&progress.scores),
        progress.clone(),
    );
    let progress_card_number = if is_initial_state {
        0.0
    } else {
        *progress_sum as f64 / LEARNED_INTERVAL_SECONDS as f64
    };

    let skill_levels = [
        (100.0, "Beginner"),
        (300.0, "Novice"),
        (1000.0, "Conversational"),
        (4000.0, "Intermediate"),
        (10000.0, "Advanced"),
        (100000.0, "Fluent"),
    ];

    let mut skill_level = "Beginner";
    let mut next_skill_level = 100.0;

    for (level, name) in skill_levels.iter() {
        if progress_card_number < *level {
            skill_level = name;
            next_skill_level = *level;
            break;
        }
    }

    let progress_percentage = (progress_card_number / next_skill_level) * 100.0;
    let rounded_progress_card_number = (progress_card_number * 10.0).round() / 10.0;

    let open_settings_cb = {
        let cb = props.open_settings.clone();
        Callback::from(move |_| cb.emit(()))
    };

    let progress_bars = (0..2).map(|i| {
        html! {
            <div class={format!("progress-bar bar-{}", i + 1)}
                 style={format!("--progress:{progress_percentage}%")}
                 onclick={open_settings_cb.clone()}>
                {
                    if is_initial_state {
                        html! { <p>{"Guide"}</p> }
                    } else {
                        html! {
                            <>
                                <p>{format!("{}", skill_level)}</p>
                                <p>{format!("{}/{}", rounded_progress_card_number, next_skill_level)}</p>
                            </>
                        }
                    }
                }
            </div>
        }
    }).collect::<Html>();

    html! {
        <>
            {progress_bars}
            <div class="flashcard-view"
                ontouchstart={on_touch_event.clone()}
                ontouchmove={on_touch_event.clone()}
                ontouchend={on_touch_event}
                onmousedown={on_mouse_event.clone()}
                onmousemove={on_mouse_event.clone()}
                onmouseup={on_mouse_event}
                style={style.clone()}>
                <Flashcard
                    swipe_direction={animation.direction()}
                    is_moving={drag.is_moving()}
                    pos_x={drag.rotation()}
                    pos_y={drag.offset().y}
                    fast_move={*animation == AnimationState::Repositioning}
                    current_card={review.presentation.card.clone()}
                    processed_sentence={review.presentation.sentence.clone()}
                    show_explanations={review.show_explanations()}
                    processed_explanation={review.presentation.explanation.clone()}
                />
            </div>
        </>
    }
}

#[function_component(App)]
fn app() -> Html {
    let show_settings = use_state(|| false);

    html! {
            <div class="app-container">
                <FlashcardView open_settings={{
                    let show_settings = show_settings.clone();
                    Callback::from(move |_| show_settings.set(true))
                }} />

                { if *show_settings { html! { <SettingsModal on_close={{
                    let show_settings = show_settings.clone();
                    Callback::from(move |_| show_settings.set(false))
                }} /> } } else { html!{} } }
            </div>
    }
}

#[derive(Properties, PartialEq, Clone)]
struct SettingsModalProps {
    on_close: Callback<()>,
}

#[function_component(SettingsModal)]
fn settings_modal(props: &SettingsModalProps) -> Html {
    html! {
        <div class="modal-overlay" onclick={{
            let cb = props.on_close.clone();
            Callback::from(move |_| cb.emit(()))
        }}>
            <div class="settings-modal" onclick={
                Callback::from(|e: MouseEvent| e.stop_propagation())
            }>
                <div class="modal-header">
                    <h3>{"Guide"}</h3>
                    <button class="close-button" onclick={{
                        let cb = props.on_close.clone();
                        Callback::from(move |_| cb.emit(()))
                    }} aria-label="Close">
                        { html!{ <i class="fas fa-times"></i> } }
                    </button>
                </div>
                <div class="modal-body">
                    <div class="instructions">
                        <div class="instruction-row">
                            <i class="fas fa-arrow-up instruction-icon" aria-hidden="true"></i>
                            <p class="instruction-text">
                                {"Swipe up or down to show the next card. A short timer starts as soon as it appears."}
                            </p>
                        </div>
                        <div class="instruction-row">
                            <i class="fas fa-arrows-left-right instruction-icon" aria-hidden="true"></i>
                            <p class="instruction-text">
                                {"When you think of the missing word, swipe left or right to reveal the answer and explanations. The timer stops the moment it's revealed."}
                            </p>
                        </div>
                        <div class="instruction-row">
                            <i class="fas fa-check instruction-icon" aria-hidden="true"></i>
                            <p class="instruction-text">
                                {"Ready to move on? Swipe up if you were correct, or down if not."}
                            </p>
                        </div>
                        <div class="instruction-row">
                            <i class="fas fa-chart-line instruction-icon" aria-hidden="true"></i>
                            <p class="instruction-text">
                                {"Each word’s progress score blends three things: how long since you last saw it, how quickly you recalled it, and whether you got it right. This drops over time."}
                            </p>
                        </div>
                        <div class="instruction-row philosophy">
                            <i class="fas fa-bolt instruction-icon" aria-hidden="true"></i>
                            <p class="instruction-text">
                                {"Real conversations are fast. So we consider words ‘learnt’ when you can recall them quickly especially after longer breaks."}
                            </p>
                        </div>
                        <div class="instruction-row app-install">
                            <i class="fas fa-mobile-screen-button instruction-icon" aria-hidden="true"></i>
                            <p class="instruction-text">
                                {"Install the app:"}
                                <br/>
                                {"• iPhone/iPad (Safari): Tap the Share icon, then \"Add to Home Screen\", then \"Add\"."}
                                <br/>
                                {"• Android (Chrome): Tap the three-dots menu, then \"Add to Home screen\" or \"Install\"."}
                                <br/>
                                {"The app opens full screen and can work offline after the first load."}
                            </p>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    yew::Renderer::<App>::new().render();
}

fn get_first_touch(event: &TouchEvent, property: &str) -> web_sys::Touch {
    let touches = Reflect::get(event, &property.into()).unwrap();
    let touch = Reflect::get(&touches, &0.into()).unwrap();
    touch.dyn_into::<web_sys::Touch>().unwrap()
}

fn move_drag(drag: &UseReducerHandle<DragState>, position: Point) {
    let window = web_sys::window().unwrap();
    let viewport_width = window.inner_width().unwrap().as_f64().unwrap();
    drag.dispatch(DragAction::Move {
        position,
        viewport_width,
    });
}

fn save_review_result(
    progress: &UseStateHandle<SavedProgress>,
    review: &ReviewState,
    direction: SwipeDirection,
) -> HashMap<String, ScoreData> {
    let attempt = ReviewAttempt {
        elapsed_ms: review.duration(),
        // Preserve the original byte-length measurement, including accented text.
        character_length: (review.presentation.card.base_language.len()
            + review.presentation.card.target_language.len()) as i32,
        outcome: if direction == SwipeDirection::Up {
            ReviewOutcome::Correct
        } else {
            ReviewOutcome::Incorrect
        },
    };
    let Some(updated) = apply_review(
        progress,
        &review.presentation.translations.list,
        &attempt,
        get_timestamp_s(),
    ) else {
        return progress.scores.clone();
    };
    // Continue safely in memory if persistence is unavailable. R5 will add user-facing recovery.
    if let Err(error) = storage::save_progress(&updated) {
        web_sys::console::error_1(&format!("Progress could not be saved; this session can continue, but changes may be lost on reload: {error}").into());
    }
    let updated_scores = updated.scores.clone();
    progress.set(updated);
    updated_scores
}

fn animate_next_review(
    content: &UseStateHandle<LearningContent>,
    review: &UseReducerHandle<ReviewState>,
    animation: &UseStateHandle<AnimationState>,
    updated_scores: HashMap<String, ScoreData>,
) {
    let content = content.clone();
    let review = review.clone();
    let animation = animation.clone();
    set_timeout(
        move || {
            animation.set(AnimationState::Repositioning);
            if let Some(presentation) = prepare_next_review(
                &content,
                &updated_scores,
                &review.presentation.translations,
                get_timestamp_s(),
                Math::random(),
            ) {
                review.dispatch(ReviewAction::Present(presentation));
            } else {
                web_sys::console::log_1(&"No valid card found with the lowest score".into());
            }
            set_timeout(
                move || {
                    animation.set(AnimationState::Settled);
                    review.dispatch(ReviewAction::HideExplanations);
                    set_timeout(
                        move || review.dispatch(ReviewAction::StartTimer(get_timestamp_ms())),
                        100,
                    );
                },
                50,
            );
        },
        400,
    );
}

/// Prepare presentation once when advancing, preserving selection and sentence rules.
fn prepare_next_review(
    content: &LearningContent,
    scores: &HashMap<String, ScoreData>,
    current_translations: &CurrentTranslations,
    now_s: i32,
    sentence_sample: f64,
) -> Option<ReviewPresentation> {
    let key = find_key_of_lowest_score(&content.cards, scores, &current_translations.list, now_s)?;
    let card = &content.cards[&key];
    if let Some(index) = choose_sentence_index(
        &card.sentence_indexes,
        current_translations.list.len() as i32,
        sentence_sample,
    ) {
        let sentence = &content.sentences[index as usize];
        let (related_cards, tokens) = find_related_cards(
            scores,
            &content.cards,
            split_text(&sentence.target_language),
            &key,
        );
        let explanation =
            construct_processed_explanation(&related_cards, &content.explanations, scores);
        Some(ReviewPresentation {
            card: Card {
                base_language: sentence.base_language.clone(),
                target_language: sentence.target_language.clone(),
            },
            translations: CurrentTranslations {
                list: related_cards,
            },
            sentence: construct_processed_sentence(tokens),
            explanation,
        })
    } else {
        Some(ReviewPresentation {
            card: Card {
                base_language: card.base_language.clone(),
                target_language: key,
            },
            // Preserve the existing no-sentence fallback; changing its review words is a separate decision.
            translations: current_translations.clone(),
            sentence: html! { <span>{ card.base_language.clone() }</span> },
            explanation: html! { <span>{ " " }</span> },
        })
    }
}

fn set_timeout<F>(callback: F, delay_ms: i32)
where
    F: FnOnce() + 'static,
{
    let closure = Closure::once(callback);

    window()
        .unwrap()
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            delay_ms,
        )
        .unwrap();

    // Forget the closure to avoid it being dropped prematurely
    closure.forget();
}

fn get_timestamp_s() -> i32 {
    let now = Date::now();
    let duration = now / 1000.0; // Convert to seconds
    duration as i32
}

fn get_timestamp_ms() -> i64 {
    let now = Date::now();
    now as i64
}

fn construct_processed_sentence(processed_sentence_data: Vec<ProcessingData>) -> Html {
    html! {
        { for processed_sentence_data.iter().map(|word| {
            if word.found {
                let char_difference = word.translation.len() as isize + 2 - word.word.len() as isize;
                let spacing_value = if char_difference > 0 { char_difference / 2 } else { 0 } as usize;
                let spaces = std::iter::repeat(html! { <span>{"\u{00A0}"}</span> })
                    .take(spacing_value)
                    .collect::<Html>();

                html! {
                    <span class="hidden-word" data-translation={ word.translation.clone() }>
                        { spaces.clone() }{ &word.word }{ spaces }
                    </span>
                }
            } else {
                html! { <span class="visible-word">{ &word.word }</span> }
            }
        })}
    }
}

fn construct_processed_explanation(
    related_cards: &Vec<Card>,
    explanations: &HashMap<String, HashMap<String, String>>,
    updated_scores: &HashMap<String, ScoreData>,
) -> Html {
    let mut explanation_nodes: Vec<Html> = vec![];
    for card in related_cards {
        let mut found = false;
        if let Some(inner) = explanations.get(&card.base_language) {
            for (target, text) in inner {
                if !found {
                    found = true;
                    explanation_nodes.push(html! {
                        <p class="explanation base-word">
                            <strong>{ format!("{}", &card.base_language) }</strong>
                        </p>
                    });
                }
                if target == &card.target_language || updated_scores.contains_key(target) {
                    explanation_nodes.push(html! {
                        <p class="explanation target-word">
                            <strong>{ format!("{}: ", target) }</strong>
                            { text }
                        </p>
                    });
                }
            }
        }
    }

    html! {
        <div class="explanation-div">
            { for explanation_nodes }
        </div>
    }
}

fn find_ancestor_with_class(mut element: Option<Element>, class_name: &str) -> Option<HtmlElement> {
    while let Some(el) = element {
        if let Some(html_el) = el.dyn_ref::<HtmlElement>() {
            if html_el.class_list().contains(class_name) {
                return Some(html_el.clone());
            }
        }
        element = el.parent_element();
    }
    None
}

fn should_enable_drag_for_event_target(target: &EventTarget) -> bool {
    if let Ok(elem) = target.clone().dyn_into::<web_sys::Element>() {
        if let Some(_) = target.dyn_ref::<HtmlElement>() {
            return find_ancestor_with_class(Some(elem.clone()), "flashcard-view").is_some()
                && !find_ancestor_with_class(Some(elem.clone()), "explanation-div")
                    .map(|card_elem| card_elem.scroll_height() > card_elem.client_height())
                    .unwrap_or(false);
        }
    }
    false
}

#[cfg(test)]
#[path = "tests/presentation_tests.rs"]
mod tests;
