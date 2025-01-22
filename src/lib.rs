use yew::prelude::*;
use yew::context::ContextProvider;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::{TouchEvent, window};
use js_sys::{Date, Reflect};
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use serde::{Serialize, Deserialize};
use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use std::collections::HashMap;
use std::cmp::max;

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
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Card {
    front: String,
    back: String,
    sentence_front: String,
    sentence_back: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
struct ScoreData {
    score: i32,
    last_review: i32,
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
    cards: UseStateHandle<Vec<Card>>,
    scores: UseStateHandle<HashMap<String, ScoreData>>,
    last_data_update: UseStateHandle<i32>,
    current_card: UseStateHandle<Card>,
    time_start: UseStateHandle<i64>,
    time_end: UseStateHandle<i64>,
    progress_sum: UseStateHandle<i64>,
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
            <div class={class_name.clone()} style={style.clone()}>
                <h1>{ props.current_card.front.clone() }</h1>
                <p>{ format!("( {} )", props.current_card.sentence_front.clone()) }</p>
            </div>
            <div class={format!("{} back-side", class_name.clone())} style={style.clone()}>
                <h1>{ props.current_card.back.clone() }</h1>
                <p>{ format!("( {} )", props.current_card.sentence_back.clone()) }</p>
            </div>
        </>
    }
}

#[function_component(FlashcardView)]
fn flashcard_view() -> Html {
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


    {
        let flashcard_context = use_context::<FlashcardContext>().expect("FlashcardContext not found");
        let cards = flashcard_context.cards.clone();
        let scores = flashcard_context.scores.clone();
        let last_data_update = flashcard_context.last_data_update.clone();

        use_effect_with_deps(move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let fetched_cards: Vec<Card> = match Request::get("/assets/croatian.json")
                    .send()
                    .await
                {
                    Ok(response) => {
                        let cards: Vec<Card> = response
                            .json()
                            .await
                            .expect("Failed to parse JSON");
    
                        LocalStorage::set("cards", &cards).expect("Failed to store cards");
                        cards
                    }
                    Err(_) => {
                        LocalStorage::get::<Vec<Card>>("cards").unwrap_or_else(|_| vec![])  // Default to empty vector if no cards found
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
                    start_x.set(touch.client_x() as f64);
                    start_y.set(touch.client_y() as f64);
                    is_moving.set(true);
                }
                "touchmove" => {
                    let touch = get_first_touch(&e, "touches");
                    let current_y = touch.client_y() as f64;
                    let current_x = touch.client_x() as f64;

                    drag_move(&flashcard_context, current_x, current_y);
                }
                "touchend" => {
                    let touch = get_first_touch(&e, "changedTouches");
                    let end_x = touch.client_x() as f64;
                    let end_y = touch.client_y() as f64;

                    calculate_swipe(
                        &flashcard_context,
                        end_x, end_y,
                    );
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
                    start_x.set(e.client_x() as f64);
                    start_y.set(e.client_y() as f64);
                    is_moving.set(true);
                }
                "mousemove" => {
                    if *is_moving {
                        let current_y = e.client_y() as f64;
                        let current_x = e.client_x() as f64;

                        drag_move(&flashcard_context, current_x, current_y);
                    }
                }
                "mouseup" => {
                    let end_x = e.client_x() as f64;
                    let end_y = e.client_y() as f64;

                    calculate_swipe(
                        &flashcard_context,
                        end_x, end_y,
                    );
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

    let progress_bars = (0..2).map(|i| {
        html! {
            <div class={format!("progress-bar bar-{}", i + 1)} style={format!("--progress:{}%", progress_percentage)}>
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
    let cards = use_state(|| vec![]);
    let scores = use_state(|| HashMap::new());
    let last_data_update = use_state(|| get_timestamp_s());
    let current_card = use_state(|| Card {
        front: "Swipe to Begin".to_string(),
        back: "Swipe Up or Down".to_string(),
        sentence_front: "".to_string(),
        sentence_back: "".to_string(),
    });
    let time_start = use_state(|| Date::now() as i64);
    let time_end = use_state(|| Date::now() as i64);
    let progress_sum = use_state(|| 0);

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
                scores: scores.clone(),
                last_data_update: last_data_update.clone(),
                current_card: current_card.clone(),
                time_start: time_start.clone(),
                time_end: time_end.clone(),
                progress_sum: progress_sum.clone(),
            }}>
            <div class="app-container">
                <FlashcardView />
            </div>
        </ContextProvider<FlashcardContext>>
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

    let new_direction = set_direction(flashcard_context, end_x, end_y, set_swipe_direction.clone(), false, false);

    if new_direction == Some(SwipeDirection::Up) || new_direction == Some(SwipeDirection::Down) {
        reset_card(
            flashcard_context,
            new_direction.clone(),
        );
    } else if new_direction == Some(SwipeDirection::Right) || new_direction == Some(SwipeDirection::Left) {
        if *time_start == *time_end {
            time_end.set(get_timestamp_ms());
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
    let scores = flashcard_context.scores.clone();
    let last_data_update = flashcard_context.last_data_update.clone();
    let current_card = flashcard_context.current_card.clone();
    let time_start = flashcard_context.time_start.clone();
    let time_end = flashcard_context.time_end.clone();
    let progress_sum = flashcard_context.progress_sum.clone();

    let duration = *time_end - *time_start;

    let mut updated_scores = (*scores).clone();
    let correct = new_direction == Some(SwipeDirection::Up);
    if duration > 1 {
        let (raw_score, raw_review) = get_raw_info_for_card(&current_card, &scores.clone());
        let (new_score, new_review) = calculate_raw_score(
            raw_score,
            raw_review,
            duration,
            (&current_card.front.len() + &current_card.back.len()) as i32,
            correct,
        );
        if current_card.front != "Swipe to Begin" {
            updated_scores = update_score(&get_scorecard_name(&current_card), new_score, new_review, scores.clone(), last_data_update.clone());
        }
    }
    update_progress(&updated_scores, progress_sum.clone());
    
    

    set_timeout(move || {
        fast_move.set(true);
        set_swipe_direction.set(SwipeDirection::Down);

        let next_index = find_index_of_lowest_score(&cards, &updated_scores, &*current_card);
        if let Some(index) = next_index {
            current_card.set(cards[index].clone());
        } else {
            // Handle the case where no valid index was found (e.g., fallback behavior)
            web_sys::console::log_1(&"No valid card found with the lowest score".into());
        }

        set_timeout(move || {
    
            fast_move.set(false);
            set_swipe_direction.set(SwipeDirection::None);

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
    key: &str,
    new_score: i32,
    new_last_review: i32,
    scores: UseStateHandle<HashMap<String, ScoreData>>,
    last_data_update: UseStateHandle<i32>,
) -> HashMap<String, ScoreData> {
    let mut updated_scores = (*scores).clone();

    let subtraction_amount = get_timestamp_s() - *last_data_update;
    for (_, score_data) in updated_scores.iter_mut() {
        score_data.score = (score_data.score - subtraction_amount).max(0);
    }

    updated_scores.entry(key.to_string())
        .and_modify(|e| {
            e.score = new_score;
            e.last_review = new_last_review;
        })
        .or_insert(ScoreData { score: new_score, last_review: new_last_review });

    scores.set(updated_scores.clone());

    let last_data_update_value = get_timestamp_s();
    last_data_update.set(last_data_update_value);

    LocalStorage::set("card_scores", &updated_scores).expect("Failed to store scores in local storage");
    LocalStorage::set("last_data_update", last_data_update_value).expect("Failed to store last data update in local storage");

    updated_scores
}

fn calculate_raw_score(
    score: i32,
    last_review: i32,
    ms_for_review: i64,
    character_length:i32,
    correct: bool,
) -> (i32, i32) {
    let mut new_score = score;
    let new_last_review = get_timestamp_s();
    let seconds_for_review = (ms_for_review as f64 / 1000.0) as f32;
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

fn get_raw_info_for_card(card: &Card, scores: &UseStateHandle<HashMap<String, ScoreData>>) -> (i32, i32) {
    let score_name = get_scorecard_name(card);
    if let Some(score_data) = scores.get(&score_name) {
        (score_data.score, score_data.last_review)
    } else {
        (0, 0)
    }
}

fn get_scorecard_name(card: &Card) -> String {
    format!("{}{}", card.front, card.back)
}

fn get_score_for_card(card: &Card, scores: HashMap<String, ScoreData>) -> i32 {
    let score_name = get_scorecard_name(card);
    if let Some(score_data) = scores.get(&score_name) {
        calculate_final_score(score_data.score, score_data.last_review)
    } else {
        calculate_final_score(0, 0)
    }
}

fn find_index_of_lowest_score(cards: &[Card], scores: &HashMap<String, ScoreData>, current_card: &Card) -> Option<usize> {
    let mut lowest_index = None;
    let mut lowest_score = None;

    for (index, card) in cards.iter().enumerate() {
        if get_scorecard_name(card) == get_scorecard_name(current_card) {
            continue;
        }

        let score = get_score_for_card(card, scores.clone());

        if lowest_score.is_none() || score < lowest_score.unwrap() {
            lowest_index = Some(index);
            lowest_score = Some(score);
        }
    }

    lowest_index
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