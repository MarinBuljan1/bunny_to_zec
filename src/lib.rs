use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use indexmap::IndexMap;
use js_sys::{Date, Math, Reflect};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::collections::HashMap;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsCast;
use web_sys::{window, Element, EventTarget, HtmlElement, TouchEvent};
use yew::prelude::*;

mod review_state;
use review_state::{
    AnimationState, DragAction, DragState, Point, ReviewAction, ReviewKind, ReviewPresentation,
    ReviewState,
};

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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Card {
    base_language: String,
    target_language: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct CurrentTranslations {
    list: Vec<Card>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct CardList {
    base_language: String,
    sentence_indexes: Vec<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct CardSentences {
    target_language: String,
    base_language: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
struct ScoreData {
    score: i32,
    last_review: i32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
struct ProcessingData {
    word: String,
    found: bool,
    translation: String,
}

#[derive(Default, PartialEq)]
struct LearningContent {
    cards: IndexMap<String, CardList>,
    sentences: Vec<CardSentences>,
    explanations: HashMap<String, HashMap<String, String>>,
}

#[derive(PartialEq)]
struct SavedProgress {
    scores: HashMap<String, ScoreData>,
    last_data_update: i32,
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
                    // Fetch cards
                    let fetched_cards: IndexMap<String, CardList> =
                        match Request::get("assets/croatian.json").send().await {
                            Ok(response) => {
                                let cards: IndexMap<String, CardList> =
                                    response.json().await.expect("Failed to parse JSON");

                                LocalStorage::set("cards", &cards).expect("Failed to store cards");
                                cards
                            }
                            Err(_) => LocalStorage::get::<IndexMap<String, CardList>>("cards")
                                .unwrap_or_else(|_| IndexMap::new()),
                        };

                    // Fetch sentences
                    let card_sentences: Vec<CardSentences> =
                        match Request::get("assets/croatian_sentences.json").send().await {
                            Ok(response) => {
                                let sentences: Vec<CardSentences> =
                                    response.json().await.expect("Failed to parse JSON");

                                LocalStorage::set("sentences", &sentences)
                                    .expect("Failed to store sentences");
                                sentences
                            }
                            Err(_) => LocalStorage::get::<Vec<CardSentences>>("sentences")
                                .unwrap_or_else(|_| vec![]),
                        };

                    // Fetch explanations
                    let fetched_explanations: HashMap<String, HashMap<String, String>> =
                        match Request::get("assets/croatian_explanations.json")
                            .send()
                            .await
                        {
                            Ok(response) => {
                                let explanations: HashMap<String, HashMap<String, String>> =
                                    response
                                        .json()
                                        .await
                                        .expect("Failed to parse explanations JSON");

                                LocalStorage::set("explanations", &explanations)
                                    .expect("Failed to store explanations");
                                explanations
                            }
                            Err(_) => {
                                LocalStorage::get("explanations").unwrap_or_else(|_| HashMap::new())
                            }
                        };

                    progress.set(SavedProgress {
                        scores: LocalStorage::get("card_scores").unwrap_or_default(),
                        last_data_update: LocalStorage::get("last_data_update")
                            .unwrap_or_else(|_| get_timestamp_s()),
                    });
                    content.set(LearningContent {
                        cards: fetched_cards,
                        sentences: card_sentences,
                        explanations: fetched_explanations,
                    });
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
        *progress_sum as f64 / calculate_seconds(10, 0, 0, 0) as f64
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

// Helper Functions
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
    let duration = review.duration();
    let current_translations = &review.presentation.translations;
    let current_card = &review.presentation.card;
    let correct = direction == SwipeDirection::Up;
    if duration > 1 {
        let raw_values = get_raw_info_for_cards(current_translations, &progress.scores);
        let new_values = calculate_raw_scores(
            raw_values,
            duration,
            (current_card.base_language.len() + current_card.target_language.len()) as i32,
            correct,
        );
        let updated = update_score(current_translations, new_values, progress);
        let updated_scores = updated.scores.clone();
        progress.set(updated);
        updated_scores
    } else {
        progress.scores.clone()
    }
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
            if let Some(presentation) =
                prepare_next_review(&content, &updated_scores, &review.presentation.translations)
            {
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
) -> Option<ReviewPresentation> {
    let key = find_key_of_lowest_score(&content.cards, scores, current_translations)?;
    let card = &content.cards[&key];
    if let Some(index) = get_random_sentence_index(
        &card.sentence_indexes,
        current_translations.list.len() as i32,
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

fn update_score(
    current_translations: &CurrentTranslations,
    new_values: Vec<(i32, i32)>,
    progress: &SavedProgress,
) -> SavedProgress {
    let mut updated_scores = progress.scores.clone();

    // Subtract score decay amount from all scores
    let subtraction_amount = get_timestamp_s() - progress.last_data_update;
    for (_, score_data) in updated_scores.iter_mut() {
        score_data.score = (score_data.score - subtraction_amount).max(0);
    }

    // Iterate through all cards in current_translations
    for (card, (new_score, new_last_review)) in
        current_translations.list.iter().zip(new_values.iter())
    {
        let key = get_scorecard_name(card); // Get unique key for each card

        updated_scores
            .entry(key)
            .and_modify(|e| {
                e.score = *new_score;
                e.last_review = *new_last_review;
            })
            .or_insert(ScoreData {
                score: *new_score,
                last_review: *new_last_review,
            });
    }

    let last_data_update_value = get_timestamp_s();

    // Store updated scores in LocalStorage
    LocalStorage::set("card_scores", &updated_scores)
        .expect("Failed to store scores in local storage");
    LocalStorage::set("last_data_update", last_data_update_value)
        .expect("Failed to store last data update in local storage");

    SavedProgress {
        scores: updated_scores,
        last_data_update: last_data_update_value,
    }
}

fn calculate_raw_scores(
    raw_values: Vec<(i32, i32)>,
    ms_for_review: i64,
    character_length: i32,
    correct: bool,
) -> Vec<(i32, i32)> {
    let inverted_normalised_scores = get_inverted_normalised_score(raw_values.clone());

    raw_values
        .into_iter()
        .zip(inverted_normalised_scores.into_iter())
        .map(|((score, last_review), inverted_normalised_score)| {
            calculate_raw_score(
                score,
                last_review,
                ms_for_review,
                character_length,
                correct,
                inverted_normalised_score,
                false,
            )
        })
        .collect()
}

fn get_inverted_normalised_score(raw_values: Vec<(i32, i32)>) -> Vec<f32> {
    // Compute total sum of scores
    let total_score: f32 = raw_values.iter().map(|(score, _)| *score as f32).sum();

    // Step 1: Compute normalised_score (ensuring min 0.1)
    let normalised_scores: Vec<f32> = raw_values
        .iter()
        .map(|(score, _)| {
            let norm_score = if total_score > 0.0 {
                *score as f32 / total_score
            } else {
                0.0
            };
            norm_score.max(0.1) // Ensure minimum score of 0.1
        })
        .collect();

    // Step 2: Compute processed_score (1 / normalised_score)
    let processed_scores: Vec<f32> = normalised_scores
        .iter()
        .map(|&norm_score| 1.0 / norm_score)
        .collect();

    // Step 3: Sum all processed_scores
    let total_processed_score: f32 = processed_scores.iter().sum();

    // Step 4: Compute inverted_normalised_score (processed_score / total_processed_score)
    let inverted_normalised_scores: Vec<f32> = processed_scores
        .iter()
        .map(|&processed_score| processed_score / total_processed_score)
        .collect();

    inverted_normalised_scores
}

fn calculate_raw_score(
    score: i32,
    last_review: i32,
    ms_for_review: i64,
    character_length: i32,
    correct: bool,
    inverted_normalised_score: f32,
    mock_last_review: bool,
) -> (i32, i32) {
    let mut new_score = score;
    let mut new_last_review;
    if mock_last_review {
        new_last_review = 1_700_000_000;
    } else {
        new_last_review = get_timestamp_s();
    }

    let seconds_for_review = (ms_for_review as f64 / 1000.0) as f32 * inverted_normalised_score;
    let mut seconds_since_last_review = (new_last_review - last_review) as f32;
    let mut calculated_divisor = (seconds_for_review / 1.5) - (character_length as f32) * 0.02;

    if last_review == 0 {
        new_score = 0;
        seconds_since_last_review = calculate_seconds(10, 0, 0, 0) as f32;
    }

    if calculated_divisor < 0.1 {
        calculated_divisor = 0.1;
    } else if calculated_divisor > 10.0 {
        calculated_divisor = 10.0;
    }

    if correct {
        new_score = new_score + (seconds_since_last_review / calculated_divisor) as i32;
    } else {
        if last_review != 0 {
            new_last_review = (last_review as f64
                + (seconds_since_last_review * inverted_normalised_score) as f64)
                as i32;
        }
    }

    (new_score, new_last_review)
}

fn calculate_final_score(score: i32, last_review: i32) -> i32 {
    let mut new_score = calculate_seconds(10, 0, 0, 0);

    if last_review != 0 {
        new_score = score
            + max(
                last_review + calculate_seconds(10, 0, 2, 0) - get_timestamp_s(),
                0,
            );
    }

    new_score
}

fn get_raw_info_for_cards(
    cards: &CurrentTranslations,
    scores: &HashMap<String, ScoreData>,
) -> Vec<(i32, i32)> {
    cards
        .list
        .iter()
        .map(|card| get_raw_info_for_card(card, scores))
        .collect()
}

fn get_raw_info_for_card(card: &Card, scores: &HashMap<String, ScoreData>) -> (i32, i32) {
    let score_name = get_scorecard_name(card);
    if let Some(score_data) = scores.get(&score_name) {
        (score_data.score, score_data.last_review)
    } else {
        (0, 0)
    }
}

fn get_scorecard_name(card: &Card) -> String {
    scorecard_name(&card.target_language)
}

fn scorecard_name(target_language: &str) -> String {
    target_language.to_string()
}

fn get_score_for_key(key: &str, scores: HashMap<String, ScoreData>) -> i32 {
    if let Some(score_data) = scores.get(key) {
        calculate_final_score(score_data.score, score_data.last_review)
    } else {
        calculate_final_score(0, 0)
    }
}

fn find_key_of_lowest_score(
    cards: &IndexMap<String, CardList>,
    scores: &HashMap<String, ScoreData>,
    current_translations: &CurrentTranslations,
) -> Option<String> {
    let current_keys: Vec<&String> = current_translations
        .list
        .iter()
        .map(|card| &card.target_language)
        .collect();

    cards
        .iter()
        .filter(|(key, _)| !current_keys.contains(key))
        .min_by_key(|(key, _)| get_score_for_key(key, scores.clone()))
        .map(|(key, _)| key.clone())
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

fn calculate_seconds(days: i32, hours: i32, minutes: i32, seconds: i32) -> i32 {
    days * 86400 + hours * 3600 + minutes * 60 + seconds
}

fn calculate_progress_sum(scores: &HashMap<String, ScoreData>) -> i64 {
    scores
        .values()
        .map(|score| i64::from(score.score.min(calculate_seconds(10, 0, 0, 0))))
        .sum()
}

fn get_random_sentence_index(sentence_indexes: &Vec<i32>, max_index: i32) -> Option<i32> {
    if sentence_indexes.is_empty() {
        return None; // Return None if the list is empty
    }

    let effective_max = max_index.max(1);

    let valid_sentence_indexes: Vec<i32> = sentence_indexes
        .iter()
        .take_while(|&&val| val < effective_max)
        .cloned()
        .collect();

    let valid_len = valid_sentence_indexes.len();

    if valid_len == 0 {
        return Some(sentence_indexes[0]);
    }

    let random_index = (Math::random() * valid_len as f64).floor() as usize;
    Some(valid_sentence_indexes[random_index])
}

fn punctuation_list() -> Vec<char> {
    vec![' ', ',', '.', '!', '?']
}

fn find_related_cards(
    updated_scores: &HashMap<String, ScoreData>,
    cards: &IndexMap<String, CardList>,
    words: Vec<String>,
    current_key: &String,
) -> (Vec<Card>, Vec<ProcessingData>) {
    let mut related_cards = Vec::new();

    let mut processed_sentence_data: Vec<ProcessingData> = words
        .iter()
        .map(|word| ProcessingData {
            word: word.clone(),
            found: false,
            translation: "".to_string(),
        })
        .collect();

    for processed_word in processed_sentence_data.iter_mut() {
        let word_key = &processed_word.word;

        if updated_scores.contains_key(&scorecard_name(word_key)) || word_key == current_key {
            // Only add if not already in the related_cards list
            if !related_cards
                .iter()
                .any(|c: &Card| &c.target_language == word_key)
            {
                if let Some(matching_card) = cards.get(word_key) {
                    processed_word.found = true;
                    processed_word.translation = matching_card.base_language.clone();

                    related_cards.push(Card {
                        base_language: matching_card.base_language.clone(),
                        target_language: word_key.clone(),
                    });
                }
            }
        }
    }

    (related_cards, processed_sentence_data)
}

fn split_text(text: &str) -> Vec<String> {
    // Create regex pattern dynamically
    let punctuations: String = punctuation_list()
        .iter()
        .map(|&c| regex::escape(&c.to_string()))
        .collect::<Vec<String>>()
        .join("|");

    let pattern = format!("({})|([^{}]+)", punctuations, punctuations);
    let re = Regex::new(&pattern).unwrap();

    // Find matches and store them in a vector (convert &str to String)
    let parts: Vec<String> = re
        .find_iter(text)
        .map(|m| m.as_str().to_string()) // Convert &str to String
        .collect();

    parts
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
#[path = "tests/learning_tests.rs"]
mod tests;
