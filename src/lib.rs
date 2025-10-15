use indexmap::IndexMap;
use js_sys::{Date, Math, Reflect};
use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use serde::{Serialize, Deserialize};
use regex::Regex;
use std::cmp::max;
use std::collections::HashMap;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::{Element, EventTarget, HtmlElement, TouchEvent, window};
use yew::context::ContextProvider;
use yew::prelude::*;

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

#[derive(Clone, PartialEq)]
struct FlashcardContext {
    start_x: UseStateHandle<f64>,
    start_y: UseStateHandle<f64>,
    pos_x: UseStateHandle<f64>,
    pos_y: UseStateHandle<f64>,
    is_moving: UseStateHandle<bool>,
    swipe_direction: UseStateHandle<SwipeDirection>,
    drag_direction: UseStateHandle<SwipeDirection>,
    fast_move: UseStateHandle<bool>,
    delta_xy: UseStateHandle<f64>,
    cards: UseStateHandle<IndexMap<String, CardList>>,
    sentences: UseStateHandle<Vec<CardSentences>>,
    scores: UseStateHandle<HashMap<String, ScoreData>>,
    last_data_update: UseStateHandle<i32>,
    current_card: UseStateHandle<Card>,
    current_translations: UseStateHandle<CurrentTranslations>,
    time_start: UseStateHandle<i64>,
    time_end: UseStateHandle<i64>,
    progress_sum: UseStateHandle<i64>,
    processed_sentence: UseStateHandle<Html>,
    explanations: UseStateHandle<HashMap<String, HashMap<String, String>>>,
    show_explanations: UseStateHandle<bool>,
    processed_explanation: UseStateHandle<Html>,
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
    }.to_string();
    
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
    let flashcard_context = use_context::<FlashcardContext>().expect("FlashcardContext not found");
    let pos_x = flashcard_context.pos_x.clone();
    let pos_y = flashcard_context.pos_y.clone();
    let is_moving = flashcard_context.is_moving.clone();
    let swipe_direction = flashcard_context.swipe_direction.clone();
    let drag_direction = flashcard_context.drag_direction.clone();
    let fast_move = flashcard_context.fast_move.clone();
    let delta_xy = flashcard_context.delta_xy.clone();
    let current_card = flashcard_context.current_card.clone();
    let progress_sum = flashcard_context.progress_sum.clone();
    let processed_sentence = flashcard_context.processed_sentence.clone();
    let processed_explanation = flashcard_context.processed_explanation.clone();


    {
        let flashcard_context = use_context::<FlashcardContext>().expect("FlashcardContext not found");
        let cards = flashcard_context.cards.clone();
        let sentences = flashcard_context.sentences.clone();
        let scores = flashcard_context.scores.clone();
        let last_data_update = flashcard_context.last_data_update.clone();
        let explanations = flashcard_context.explanations.clone();

        use_effect_with_deps(move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                // Fetch cards
                let fetched_cards: IndexMap<String, CardList> = match Request::get("assets/croatian.json")
                    .send()
                    .await
                {
                    Ok(response) => {
                        let cards: IndexMap<String, CardList> = response
                            .json()
                            .await
                            .expect("Failed to parse JSON");
    
                        LocalStorage::set("cards", &cards).expect("Failed to store cards");
                        cards
                    }
                    Err(_) => {
                        LocalStorage::get::<IndexMap<String, CardList>>("cards").unwrap_or_else(|_| IndexMap::new())
                    }
                };

                // Fetch sentences
                let card_sentences: Vec<CardSentences> = match Request::get("assets/croatian_sentences.json")
                    .send()
                    .await
                {
                    Ok(response) => {
                        let sentences: Vec<CardSentences> = response
                            .json()
                            .await
                            .expect("Failed to parse JSON");

                        LocalStorage::set("sentences", &sentences).expect("Failed to store sentences");
                        sentences
                    }
                    Err(_) => {
                        LocalStorage::get::<Vec<CardSentences>>("sentences").unwrap_or_else(|_| vec![])
                    }
                };

                // Fetch explanations
                let fetched_explanations: HashMap<String, HashMap<String, String>> =
                    match Request::get("assets/croatian_explanations.json")
                    .send()
                    .await 
                {
                    Ok(response) => {
                        let explanations: HashMap<String, HashMap<String, String>> = response
                            .json()
                            .await
                            .expect("Failed to parse explanations JSON");

                        LocalStorage::set("explanations", &explanations)
                            .expect("Failed to store explanations");
                        explanations
                    }
                    Err(_) => {
                        LocalStorage::get("explanations")
                            .unwrap_or_else(|_| HashMap::new())
                    }
                };
    
                if let Ok(stored_scores) = LocalStorage::get::<HashMap<String, ScoreData>>("card_scores") {
                    scores.set(stored_scores);
                } else {
                    scores.set(HashMap::new());
                }
                if let Ok(stored_last_data_update) = LocalStorage::get::<i32>("last_data_update") {
                    last_data_update.set(stored_last_data_update);
                } else {
                    last_data_update.set(get_timestamp_s());
                }
    
                cards.set(fetched_cards);
                sentences.set(card_sentences);
                explanations.set(fetched_explanations);
            });

            || ()
        }, ());
    }

    let on_touch_event = {
        let flashcard_context = use_context::<FlashcardContext>().expect("FlashcardContext not found");
        let start_x = flashcard_context.start_x.clone();
        let start_y = flashcard_context.start_y.clone();
        let is_moving = flashcard_context.is_moving.clone();

        Callback::from(move |e: TouchEvent| {
            match e.type_().as_str() {
                "touchstart" => {
                    let touch = get_first_touch(&e, "touches");
                    if let Some(target) = e.target() {
                        if should_enable_drag_for_event_target(&target) {
                            start_x.set(touch.client_x() as f64);
                            start_y.set(touch.client_y() as f64);
                            is_moving.set(true);
                        }
                    }
                }
                "touchmove" => {
                    if *is_moving {
                        let touch = get_first_touch(&e, "touches");
                        let current_y = touch.client_y() as f64;
                        let current_x = touch.client_x() as f64;

                        drag_move(&flashcard_context, current_x, current_y);
                    }
                }
                "touchend" => {
                    if *is_moving {
                        let touch = get_first_touch(&e, "changedTouches");
                        let end_x = touch.client_x() as f64;
                        let end_y = touch.client_y() as f64;

                        calculate_swipe(
                            &flashcard_context,
                            end_x, end_y,
                        );
                    }
                }
                _ => {}
            }
        })
    };

    let on_mouse_event = {
        let flashcard_context = use_context::<FlashcardContext>().expect("FlashcardContext not found");
        let start_x = flashcard_context.start_x.clone();
        let start_y = flashcard_context.start_y.clone();
        let is_moving = flashcard_context.is_moving.clone();

        Callback::from(move |e: MouseEvent| {
            e.prevent_default();

            match e.type_().as_str() {
                "mousedown" => {
                    if let Some(target) = e.target() {
                        if should_enable_drag_for_event_target(&target) {
                            start_x.set(e.client_x() as f64);
                            start_y.set(e.client_y() as f64);
                            is_moving.set(true);
                        }
                    }
                }
                "mousemove" => {
                    if *is_moving {
                        let current_y = e.client_y() as f64;
                        let current_x = e.client_x() as f64;

                        drag_move(&flashcard_context, current_x, current_y);
                    }
                }
                "mouseup" => {
                    if *is_moving {
                        let end_x = e.client_x() as f64;
                        let end_y = e.client_y() as f64;

                        calculate_swipe(
                            &flashcard_context,
                            end_x, end_y,
                        );
                    }
                }
                _ => {}
            }
        })
    };

    let bg_color = match *drag_direction {
        SwipeDirection::Up => "0, 255, 0",
        SwipeDirection::Down => "255, 0, 0",
        SwipeDirection::Right => "100, 100, 100",
        SwipeDirection::Left => "100, 100, 100",
        SwipeDirection::None => "0, 0, 0",
    }.to_string();
    let bg_direction = match *drag_direction {
        SwipeDirection::Up => "180deg",
        SwipeDirection::Down => "0deg",
        SwipeDirection::Right => "270deg",
        SwipeDirection::Left => "90deg",
        SwipeDirection::None => "0deg",
    }.to_string();
    let style = format!("--bg-rgba: {}, {}; --bg-direction: {}", bg_color, *delta_xy, bg_direction);

    let progress_card_number = *progress_sum as f64 / calculate_seconds(10, 0, 0, 0) as f64;

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
                 style={format!("--progress:{}%", progress_percentage)}
                 onclick={open_settings_cb.clone()}>
                <p>{format!("{}", skill_level)}</p>
                <p>{format!("{}/{}", rounded_progress_card_number, next_skill_level)}</p>
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
                    swipe_direction={(*swipe_direction).clone()} 
                    is_moving={*is_moving} 
                    pos_x={*pos_x} 
                    pos_y={*pos_y} 
                    fast_move={*fast_move} 
                    current_card={(*current_card).clone()}
                    processed_sentence={(*processed_sentence).to_owned()}
                    show_explanations={*flashcard_context.show_explanations}
                    processed_explanation={(*processed_explanation).to_owned()}
                />
            </div>
        </>
    }
}

#[function_component(App)]
fn app() -> Html {
    let start_x = use_state(|| 0.0);
    let start_y = use_state(|| 0.0);
    let pos_x = use_state(|| 0.0);
    let pos_y = use_state(|| 0.0);
    let is_moving = use_state(|| false);
    let swipe_direction = use_state(|| SwipeDirection::None);
    let drag_direction = use_state(|| SwipeDirection::None);
    let fast_move = use_state(|| false);
    let delta_xy = use_state(|| 0.0);
    let cards = use_state(|| IndexMap::new());
    let sentences = use_state(|| vec![]);
    let scores = use_state(|| HashMap::new());
    let last_data_update = use_state(|| get_timestamp_s());
    let current_card = use_state(|| Card {
        base_language: "Swipe to Begin".to_string(),
        target_language: "Swipe Up or Down".to_string(),
    });
    let current_translations = use_state(|| CurrentTranslations { list: vec![] });
    let time_start = use_state(|| Date::now() as i64);
    let time_end = use_state(|| Date::now() as i64);
    let progress_sum = use_state(|| 0);
    let processed_sentence = use_state(|| html! { "Swipe to Begin" });
    let explanations = use_state(|| HashMap::new());
    let show_explanations = use_state(|| false);
    let processed_explanation = use_state(|| html! { "Swipe Up or Down" });
    let show_settings = use_state(|| false);

    html! {
        <ContextProvider<FlashcardContext> context={
            FlashcardContext { 
                start_x: start_x.clone(), 
                start_y: start_y.clone(), 
                pos_x: pos_x.clone(), 
                pos_y: pos_y.clone(),
                is_moving: is_moving.clone(),
                swipe_direction: swipe_direction.clone(),
                drag_direction: drag_direction.clone(),
                fast_move: fast_move.clone(),
                delta_xy: delta_xy.clone(),
                cards: cards.clone(),
                sentences: sentences.clone(),
                scores: scores.clone(),
                last_data_update: last_data_update.clone(),
                current_card: current_card.clone(),
                current_translations: current_translations.clone(),
                time_start: time_start.clone(),
                time_end: time_end.clone(),
                progress_sum: progress_sum.clone(),
                processed_sentence: processed_sentence.clone(),
                explanations: explanations.clone(),
                show_explanations: show_explanations.clone(),
                processed_explanation: processed_explanation.clone(),
            }}>
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
        </ContextProvider<FlashcardContext>>
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
                                {"When you think of the missing word, swipe left or right to reveal the answer and explanations. The timer stops the moment you reveal."}
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
                                {"Each word’s progress blends three things: how long since you last saw it, how quickly you recalled it, and whether you got it right. Your progress is saved in this browser."}
                            </p>
                        </div>
                        <div class="instruction-row philosophy">
                            <i class="fas fa-bolt instruction-icon" aria-hidden="true"></i>
                            <p class="instruction-text">
                                {"Real conversations are fast. Words become more ‘learnt’ when you can recall them quickly especially after longer breaks."}
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

fn drag_move(
    flashcard_context: &FlashcardContext, 
    current_x: f64, current_y: f64
) {
    let start_x = *flashcard_context.start_x.clone();
    let start_y = *flashcard_context.start_y.clone();
    let pos_x = flashcard_context.pos_x.clone();
    let pos_y = flashcard_context.pos_y.clone();
    let delta_xy = flashcard_context.delta_xy.clone();
    let drag_direction = flashcard_context.drag_direction.clone();

    let window = web_sys::window().unwrap();
    let window_width = window.inner_width().unwrap().as_f64().unwrap();

    pos_y.set(current_y - start_y);
    pos_x.set((current_x - start_x)/window_width * 90.0);
    delta_xy.set(calc_delta_xy(current_x - start_x, current_y - start_y));

    set_direction(flashcard_context, current_x, current_y, drag_direction.clone(), true, true);
}

fn calculate_swipe(
    flashcard_context: &FlashcardContext, 
    end_x: f64, end_y: f64,
) {
    let set_swipe_direction = flashcard_context.swipe_direction.clone();
    let is_moving = flashcard_context.is_moving.clone();
    let pos_x = flashcard_context.pos_x.clone();
    let pos_y = flashcard_context.pos_y.clone();
    let delta_xy = flashcard_context.delta_xy.clone();
    let time_start = flashcard_context.time_start.clone();
    let time_end = flashcard_context.time_end.clone();
    let show_explanations = flashcard_context.show_explanations.clone();

    let new_direction = set_direction(flashcard_context, end_x, end_y, set_swipe_direction.clone(), false, false);

    if new_direction == Some(SwipeDirection::Up) || new_direction == Some(SwipeDirection::Down) {
        reset_card(
            flashcard_context,
            new_direction.clone(),
        );
    } else if new_direction == Some(SwipeDirection::Right) || new_direction == Some(SwipeDirection::Left) {
        if *time_start == *time_end {
            time_end.set(get_timestamp_ms());
            
            set_timeout(move || {
                show_explanations.set(true);
            }, 400);
        }
    }

    is_moving.set(false);
    pos_x.set(0.0);
    pos_y.set(0.0);
    delta_xy.set(0.0);
}

fn set_direction(
    flashcard_context: &FlashcardContext,
    end_x: f64, end_y: f64,
    set_variable: UseStateHandle<SwipeDirection>,
    ignore_none: bool,
    ignore_min_distance: bool,
) -> Option <SwipeDirection> {
    let start_x = *flashcard_context.start_x.clone();
    let start_y = *flashcard_context.start_y.clone();
    let delta_x = end_x - start_x;
    let delta_y = end_y - start_y;
    let delta_xy = calc_delta_xy(delta_x, delta_y);

    let mut new_direction = None;

    (if delta_xy >= 1.0 || ignore_min_distance {
        if delta_x.abs() > delta_y.abs() {
            if *set_variable == SwipeDirection::None || ignore_none {
                if delta_x >= 0.0 {
                    new_direction = Some(SwipeDirection::Right);
                } else {
                    new_direction = Some(SwipeDirection::Left);
                }
            } else {
                new_direction = Some(SwipeDirection::None);
            }
        } else {
            if delta_y >= 0.0 {
                new_direction = Some(SwipeDirection::Down);
            } else {
                new_direction = Some(SwipeDirection::Up);
            }
        }
    });

    if let Some(direction) = new_direction {
        set_variable.set(direction.clone());
        return Some(direction);
    }

    None
}

fn calc_delta_xy(delta_x: f64, delta_y: f64) -> f64 {
    (delta_x.abs() - delta_y.abs()).abs().min(50.0) / 50.0
}

fn reset_card(
    flashcard_context: &FlashcardContext,
    new_direction: Option<SwipeDirection>,
) {
    let set_swipe_direction = flashcard_context.swipe_direction.clone();
    let fast_move = flashcard_context.fast_move.clone();
    let cards = flashcard_context.cards.clone();
    let sentences = flashcard_context.sentences.clone();
    let scores = flashcard_context.scores.clone();
    let last_data_update = flashcard_context.last_data_update.clone();
    let current_card = flashcard_context.current_card.clone();
    let current_translations = flashcard_context.current_translations.clone();
    let time_start = flashcard_context.time_start.clone();
    let time_end = flashcard_context.time_end.clone();
    let progress_sum = flashcard_context.progress_sum.clone();
    let processed_sentence = flashcard_context.processed_sentence.clone();
    let explanations = flashcard_context.explanations.clone();
    let show_explanations = flashcard_context.show_explanations.clone();
    let processed_explanation = flashcard_context.processed_explanation.clone();

    let duration = *time_end - *time_start;

    let mut updated_scores = (*scores).clone();
    let correct = new_direction == Some(SwipeDirection::Up);
    if duration > 1 { // Avoids dividing by zero
        let raw_values = get_raw_info_for_cards(&current_translations, &scores.clone());
        let new_values = calculate_raw_scores(
            raw_values,
            duration,
            (&current_card.base_language.len() + &current_card.target_language.len()) as i32,
            correct,
        );
        // if current_card.base_language != "Swipe to Begin" {
        updated_scores = update_score(&current_translations, new_values, scores.clone(), last_data_update.clone());
        // }
    }
    update_progress(&updated_scores, progress_sum.clone());
    
    

    set_timeout(move || {
        fast_move.set(true);
        set_swipe_direction.set(SwipeDirection::Down);

        let next_key = find_key_of_lowest_score(&cards, &updated_scores, &current_translations);
        if let Some(key) = next_key {
            // current_card.set(cards[index].clone());
            if let Some(sentence_index) = get_random_sentence_index(&cards[&key].sentence_indexes, current_translations.list.len() as i32) {
                let new_sentences = sentences[sentence_index as usize].clone();
                let split_text = split_text(&new_sentences.target_language);
                let (related_cards, processed_sentence_values) = find_related_cards(&updated_scores, &cards, split_text, &key);
                current_translations.set(CurrentTranslations { list: related_cards.clone() });
                
                let processed_sentence_final = construct_processed_sentence(processed_sentence_values);

                let processed_explanation_final = construct_processed_explanation(&related_cards, &*explanations, &updated_scores);

                current_card.set(Card {
                    base_language: new_sentences.base_language.clone(),
                    target_language: new_sentences.target_language.clone()
                });

                processed_sentence.set(processed_sentence_final);

                processed_explanation.set(processed_explanation_final);

            } else {
                // Handle the case where no sentence index is available
                current_card.set(Card {
                    base_language: cards[&key].base_language.clone(),
                    target_language: key.clone(),
                });

                processed_sentence.set(html! { <span>{ cards[&key].base_language.clone() }</span> });

                processed_explanation.set(html! { <span>{ " " }</span> });
            }
            
        } else {
            // Handle the case where no valid index was found (e.g., fallback behavior)
            web_sys::console::log_1(&"No valid card found with the lowest score".into());
        }

        set_timeout(move || {
    
            fast_move.set(false);
            set_swipe_direction.set(SwipeDirection::None);
            show_explanations.set(false);

            set_timeout(move || {
                
                let current_time = get_timestamp_ms();
                time_start.set(current_time);
                time_end.set(current_time);
        
            }, 100);
    
        }, 50);

    }, 400);
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
    current_translations: &UseStateHandle<CurrentTranslations>,
    new_values: Vec<(i32, i32)>,
    scores: UseStateHandle<HashMap<String, ScoreData>>,
    last_data_update: UseStateHandle<i32>,
) -> HashMap<String, ScoreData> {
    let mut updated_scores = (*scores).clone();

    // Subtract score decay amount from all scores
    let subtraction_amount = get_timestamp_s() - *last_data_update;
    for (_, score_data) in updated_scores.iter_mut() {
        score_data.score = (score_data.score - subtraction_amount).max(0);
    }

    // Iterate through all cards in current_translations
    for (card, (new_score, new_last_review)) in current_translations.list.iter().zip(new_values.iter()) {
        let key = get_scorecard_name(card); // Get unique key for each card

        updated_scores.entry(key)
            .and_modify(|e| {
                e.score = *new_score;
                e.last_review = *new_last_review;
            })
            .or_insert(ScoreData { score: *new_score, last_review: *new_last_review });
    }

    // Update state with new scores
    scores.set(updated_scores.clone());

    // Update last data update timestamp
    let last_data_update_value = get_timestamp_s();
    last_data_update.set(last_data_update_value);

    // Store updated scores in LocalStorage
    LocalStorage::set("card_scores", &updated_scores).expect("Failed to store scores in local storage");
    LocalStorage::set("last_data_update", last_data_update_value).expect("Failed to store last data update in local storage");

    updated_scores
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
                false
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
    character_length:i32,
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
        new_score = new_score + (
            seconds_since_last_review 
            / calculated_divisor
        ) as i32;
    } else {
        if last_review != 0 {
            new_last_review = (last_review as f64 + (seconds_since_last_review * inverted_normalised_score) as f64) as i32;
        }
    }

    (new_score, new_last_review)
}

fn calculate_final_score(
    score: i32,
    last_review: i32,
) -> i32 {
    let mut new_score = calculate_seconds(10, 0, 0, 0);

    if last_review != 0 {
        new_score = score + max(last_review + calculate_seconds(10, 0, 2, 0) - get_timestamp_s(), 0);
    }

    new_score
}

fn get_raw_info_for_cards(cards: &UseStateHandle<CurrentTranslations>, scores: &UseStateHandle<HashMap<String, ScoreData>>) -> Vec<(i32, i32)> {
    cards.list.iter()
        .map(|card| get_raw_info_for_card(card, scores))
        .collect()
}

fn get_raw_info_for_card(card: &Card, scores: &UseStateHandle<HashMap<String, ScoreData>>) -> (i32, i32) {
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
    current_translations: &UseStateHandle<CurrentTranslations>,
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
    let duration = now / 1000.0;  // Convert to seconds
    duration as i32
}

fn get_timestamp_ms() -> i64 {
    let now = Date::now();
    now as i64
}

fn update_progress(updated_scores: &HashMap<String, ScoreData>, progress_sum: UseStateHandle<i64>) {
    let mut score_sum = 0;
    for (_, score_data) in updated_scores.iter() {
        score_sum += score_data.score.min(calculate_seconds(10, 0, 0, 0));
    }
    progress_sum.set(score_sum.into());
}

fn calculate_seconds(days: i32, hours: i32, minutes: i32, seconds: i32) -> i32 {
    days * 86400 + hours * 3600 + minutes * 60 + seconds
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
            if !related_cards.iter().any(|c: &Card| &c.target_language == word_key) {
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
    let parts: Vec<String> = re.find_iter(text)
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
                &&
                !find_ancestor_with_class(Some(elem.clone()), "explanation-div")
                    .map(|card_elem| card_elem.scroll_height() > card_elem.client_height())
                    .unwrap_or(false);
        }
    }
    false
}




#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::indexmap;

    #[test]
    fn test_find_related_cards() {
        let updated_scores: HashMap<String, ScoreData> = vec![
            ("hola".to_string(), ScoreData { score: 5, last_review: 0 }),
            ("mundo".to_string(), ScoreData { score: 3, last_review: 0 }),
        ]
        .into_iter()
        .collect();

        let words = vec!["hola".to_string(), "óxido".to_string(), "mundo".to_string()];

        let cards = indexmap! {
            "hola".to_string() => CardList { base_language: "hello".to_string(), sentence_indexes: vec![0], },
            "mundo".to_string() => CardList { base_language: "world".to_string(), sentence_indexes: vec![0], },
            "óxido".to_string() => CardList { base_language: "rust".to_string(), sentence_indexes: vec![0], },
        };

        let current_card = "mundo".to_string();

        let (related_cards, processed_sentence_data) =
            find_related_cards(&updated_scores, &cards, words, &current_card);

        // Expected related cards (must match words in `cards`)
        let expected_related_cards = vec![
            Card { base_language: "hello".to_string(), target_language: "hola".to_string() },
            Card { base_language: "world".to_string(), target_language: "mundo".to_string() }
        ];

        // Expected processed sentence data
        let expected_processed_sentence_data = vec![
            ProcessingData { word: "hola".to_string(), found: true, translation: "hello".to_string() },
            ProcessingData { word: "óxido".to_string(), found: false, translation: "".to_string() },
            ProcessingData { word: "mundo".to_string(), found: true, translation: "world".to_string() }
        ];

        // Check related cards match expected results
        assert_eq!(related_cards, expected_related_cards, "Related cards do not match expected values.");

        // Check processed sentence data
        assert_eq!(
            processed_sentence_data,
            expected_processed_sentence_data,
            "Processed sentence data is incorrect."
        );
    }

    #[test]
    fn test_split_text() {
        let examples_split_text = vec![
            ("Hello, world! Rust is great.",
             vec!["Hello", ",", " ", "world", "!", " ", "Rust", " ", "is", " ", "great", "."]),
            ("Hey! How's it going?",
             vec!["Hey", "!", " ", "How's", " ", "it", " ", "going", "?"]),
            ("No punctuation",
             vec!["No", " ", "punctuation"]),
            ("A, B, C!",
             vec!["A", ",", " ", "B", ",", " ", "C", "!"]),
            ("  Spacing  Test  ",
             vec![" ", " ", "Spacing", " ", " ", "Test", " ", " "])
        ];

        for (input, expected) in examples_split_text {
            let result = split_text(input);
            assert_eq!(result, expected, "Failed on input: '{}'", input);
        }
    }

    #[test]
    fn test_get_inverted_normalised_score() {
        let test_cases = vec![
            // (Input Values, Expected Output)
            (
                vec![(1, 0), (2, 0), (3, 0), (4, 0)], 
                vec![0.48, 0.24, 0.16, 0.12] // Expected normalized values
            ),
            (
                vec![(10, 0), (20, 0), (30, 0), (40, 0)], 
                vec![0.48, 0.24, 0.16, 0.12] // Expected normalized values
            ),
            (
                vec![(2, 0), (5, 0), (10, 0), (20, 0)], 
                vec![0.43573, 0.32244, 0.16122, 0.08061] // Expected normalized values
            )
        ];

        for (input_values, expected_output) in test_cases {
            let result = get_inverted_normalised_score(input_values.clone());
            
            // Ensure result length matches expected length
            assert_eq!(result.len(), expected_output.len(), "Result length mismatch for input: {:?}", input_values);

            // Ensure each calculated value is within a small margin of error (floating point comparison)
            for (res, exp) in result.iter().zip(expected_output.iter()) {
                assert!(
                    (res - exp).abs() < 1e-6, 
                    "Expected {}, got {} for input {:?}", exp, res, input_values
                );
            }
        }
    }

    #[test]
    fn test_calculate_raw_score() {
        let test_cases = vec![
            // (score, last_review, ms_for_review, character_length, correct, inverted_score, expected_score, expected_last_review)
            (100, 1_700_000_000 - 1000, 5000, 20, true, 0.48, 933, 1_700_000_000),
            (200, 1_700_000_000 - 1000, 5000, 20, true, 0.24, 2700, 1_700_000_000),
            (300, 1_700_000_000 - 1000, 5000, 20, true, 0.16, 7800, 1_700_000_000),
            (400, 1_700_000_000 - 1000, 5000, 20, true, 0.12, 10400, 1_700_000_000),
            (100, 1_700_000_000 - 1000, 5000, 20, false, 0.48, 100, 1_699_999_480),
            (200, 1_700_000_000 - 1000, 5000, 20, false, 0.24, 200, 1_699_999_240),
            (300, 1_700_000_000 - 1000, 5000, 20, false, 0.16, 300, 1_699_999_160),
            (400, 1_700_000_000 - 1000, 5000, 20, false, 0.12, 400, 1_699_999_120),
        ];

        for (score, last_review, ms_for_review, character_length, correct, inverted_score, expected_score, expected_last_review) in test_cases {
            let (new_score, new_last_review) = calculate_raw_score(
                score,
                last_review,
                ms_for_review,
                character_length,
                correct,
                inverted_score,
                true,  // Enable mock_last_review for consistent testing
            );

            assert_eq!(new_score, expected_score, "Incorrect score for input: {:?}", (score, last_review, ms_for_review, character_length, correct, inverted_score));
            assert_eq!(new_last_review, expected_last_review, "Incorrect last_review for input: {:?}", (score, last_review, ms_for_review, character_length, correct, inverted_score));
        }
    }
}
