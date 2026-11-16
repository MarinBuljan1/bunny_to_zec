use gloo_events::EventListener;
use gloo_timers::callback::Timeout;
use js_sys::{Date, Math};
use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsCast;
use web_sys::{window, Element, EventTarget, PointerEvent};
use yew::prelude::*;

mod animation;
mod chat;
mod conversations;
mod drag_motion;
mod gesture_hint;
mod gestures;
mod guide;
mod learning;
mod message_bubble;
mod model;
mod persistence;
mod review_state;
mod scroll_drag;
mod scroll_motion;
mod sentences;
mod storage;
mod word_notes;
use animation::AnimationStepKind;
use guide::Guide;
use learning::{
    apply_review, calculate_progress_sum, find_key_of_lowest_score, ReviewAttempt, ReviewOutcome,
    LEARNED_INTERVAL_SECONDS,
};
use model::{Card, LearningContent, SavedProgress, ScoreData};
use review_state::{
    DragAction, DragState, Point, ReviewAction, ReviewKind, ReviewPresentation, ReviewState,
};
use sentences::{choose_sentence_index, prepare_sentence, split_text, ProcessedToken};
use storage::ContentLoadState;

#[derive(PartialEq)]
enum SaveStatus {
    Saved,
    Warning(String),
    Unsaved(String),
    ReadOnly(String),
}

#[derive(Clone, Default, PartialEq, Debug)]
enum SwipeDirection {
    #[default]
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
    let front = use_node_ref();
    let explanation_scrolls = use_state_eq(|| false);
    {
        let front = front.clone();
        let explanation_scrolls = explanation_scrolls.clone();
        use_effect_with_deps(
            move |_| {
                let measure = move || {
                    let scrolls = front
                        .cast::<Element>()
                        .and_then(|card| card.query_selector(".explanation-div").ok().flatten())
                        .is_some_and(|explanation| {
                            explanation.scroll_height() > explanation.client_height()
                        });
                    explanation_scrolls.set(scrolls);
                };
                measure();
                let resize = window()
                    .map(|window| EventListener::new(&window, "resize", move |_| measure()));
                move || drop(resize)
            },
            (props.show_explanations, props.processed_explanation.clone()),
        );
    }
    let front_class = if props.show_explanations && *explanation_scrolls {
        "front-side explanation-scrollable"
    } else {
        "front-side"
    };
    let back_visible = matches!(
        props.swipe_direction,
        SwipeDirection::Left | SwipeDirection::Right
    );
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
            <div ref={front} inert={back_visible.then_some("")} aria-hidden={back_visible.to_string()} class={format!("{class_name} {front_class}")} style={style.clone()}>
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
            <div inert={(!back_visible).then_some("")} aria-hidden={(!back_visible).to_string()} class={format!("{} back-side", class_name.clone())} style={style.clone()}>
                <p class="top">
                    <span aria-hidden="true">{ props.processed_sentence.clone() }</span>
                    <span class="sr-only">{ &props.current_card.target_language }</span>
                </p>
                <p class="bottom">{ format!("( {} )", props.current_card.base_language.clone()) }</p>
            </div>
        </>
    }
}

#[derive(Properties, PartialEq, Clone)]
struct FlashcardViewProps {
    open_settings: Callback<()>,
    guide_open: bool,
}

#[function_component(FlashcardView)]
fn flashcard_view(props: &FlashcardViewProps) -> Html {
    let content = use_state(|| ContentLoadState::Loading);
    let load_attempt = use_state(|| 0_u32);
    let save_status = use_state(|| SaveStatus::Saved);
    let progress = use_state(|| SavedProgress {
        scores: HashMap::new(),
        last_data_update: get_timestamp_s(),
        conversations: Default::default(),
    });
    let review = use_reducer(|| ReviewState::welcome(get_timestamp_ms()));
    let drag = use_reducer(DragState::default);
    let drag_surface = use_node_ref();

    // Progress is read independently of content/network success, once per mount.
    {
        let progress = progress.clone();
        let save_status = save_status.clone();
        use_effect_with_deps(
            move |_| {
                match storage::load_progress(get_timestamp_s()) {
                    Ok(loaded) => {
                        let status = loaded
                            .warning
                            .map(SaveStatus::Warning)
                            .unwrap_or(SaveStatus::Saved);
                        progress.set(loaded.progress);
                        save_status.set(status);
                    }
                    Err(error) => {
                        let message = format!("{error} Saved progress is unchanged. This session won't be saved. Reload after fixing browser storage.");
                        save_status.set(SaveStatus::ReadOnly(message));
                    }
                }
                || ()
            },
            (),
        );
    }
    {
        let content = content.clone();
        use_effect_with_deps(
            move |_| {
                // The request may finish after teardown, but a superseded load must
                // never write a cache or update UI state. Cleanup owns cancellation.
                let cancelled = Rc::new(Cell::new(false));
                let request_cancelled = cancelled.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let result = storage::load_learning_content().await;
                    if let Some(state) = storage::commit_content_load(result, &request_cancelled) {
                        content.set(state);
                    }
                });
                move || cancelled.set(true)
            },
            *load_attempt,
        );
    }

    let retry_loading = {
        let content = content.clone();
        let load_attempt = load_attempt.clone();
        Callback::from(move |_| {
            content.set(ContentLoadState::Loading);
            load_attempt.set(load_attempt.wrapping_add(1));
        })
    };
    let retry_saving = {
        let progress = progress.clone();
        let save_status = save_status.clone();
        Callback::from(move |_| save_progress_and_update_status(&progress, &save_status))
    };

    {
        let animation_step = review.animation.step();
        let review = review.clone();
        let content = content.clone();
        let progress = progress.clone();
        let save_status = save_status.clone();
        use_effect_with_deps(
            move |step| {
                let timer = step.as_ref().map(|step: &animation::AnimationStep| {
                    match &step.kind {
                        AnimationStepKind::Advance(_) => {
                            save_review_result(&progress, &review, step, &save_status);
                        }
                        AnimationStepKind::Reposition => {
                            if let ContentLoadState::Ready(loaded) = &*content {
                                if let Some(presentation) = prepare_next_review(
                                    &loaded.content,
                                    &progress.scores,
                                    &review.presentation.review_words,
                                    get_timestamp_s(),
                                    Math::random(),
                                ) {
                                    review.dispatch(ReviewAction::PresentNext {
                                        generation: step.generation,
                                        presentation,
                                    });
                                } else {
                                    web_sys::console::log_1(
                                        &"No valid card found with the lowest score".into(),
                                    );
                                }
                            }
                        }
                        AnimationStepKind::Reveal | AnimationStepKind::StartQuestion => {}
                    }
                    let generation = step.generation;
                    // Start the full interval after preparation so the offscreen card
                    // gets its original 50 ms before returning to the front.
                    let delay = step.delay_ms();
                    let review = review.clone();
                    Timeout::new(delay, move || {
                        review.dispatch(ReviewAction::AnimationElapsed {
                            generation,
                            at_ms: get_timestamp_ms(),
                        })
                    })
                });
                // Changing phase or unmounting drops the handle and cancels its timer.
                move || drop(timer)
            },
            animation_step,
        );
    }

    let on_swipe_completed = {
        let drag = drag.clone();
        let review = review.clone();
        Callback::from(move |(pointer_id, end): (i32, Point)| {
            if let Some(direction) =
                drag.completed_direction(pointer_id, end, &review.animation.direction())
            {
                review.dispatch(ReviewAction::Swipe {
                    direction,
                    at_ms: get_timestamp_ms(),
                });
            }
            drag.dispatch(DragAction::End(pointer_id));
        })
    };

    let on_pointer_event = {
        let drag = drag.clone();
        let drag_surface = drag_surface.clone();
        let content = content.clone();
        let review = review.clone();
        Callback::from(move |event: PointerEvent| {
            let pointer_id = event.pointer_id();
            let position = Point {
                x: event.client_x() as f64,
                y: event.client_y() as f64,
            };
            match event.type_().as_str() {
                "pointerdown" => {
                    if !review.animation.accepts_swipes()
                        || !matches!(&*content, ContentLoadState::Ready(_))
                        || !event.is_primary()
                        || event.button() != 0
                        || drag.is_moving()
                        || !event
                            .target()
                            .as_ref()
                            .is_some_and(should_enable_drag_for_event_target)
                    {
                        return;
                    }
                    // Yew delegates listeners to the app root, so current_target()
                    // is not this view. Capture on the node that owns our callbacks.
                    let Some(surface) = drag_surface.cast::<Element>() else {
                        return;
                    };
                    if let Err(error) = surface.set_pointer_capture(pointer_id) {
                        web_sys::console::error_2(
                            &"Could not capture the pointer; drag was not started".into(),
                            &error,
                        );
                        return;
                    }
                    event.prevent_default();
                    drag.dispatch(DragAction::Start {
                        pointer_id,
                        origin: position,
                    });
                }
                "pointermove" if drag.is_active_pointer(pointer_id) => {
                    if event.buttons() & 1 == 0 {
                        drag.dispatch(DragAction::End(pointer_id));
                        return;
                    }
                    event.prevent_default();
                    move_drag(&drag, pointer_id, position);
                }
                "pointerup" if drag.is_active_pointer(pointer_id) => {
                    event.prevent_default();
                    on_swipe_completed.emit((pointer_id, position));
                    // The browser releases capture after pointerup; lost capture only cancels.
                }
                "pointercancel" | "lostpointercapture" => {
                    drag.dispatch(DragAction::End(pointer_id))
                }
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

    let actions_enabled = matches!(&*content, ContentLoadState::Ready(_))
        && review.animation.accepts_swipes()
        && !drag.is_moving()
        && !props.guide_open;
    let act = {
        let review = review.clone();
        Callback::from(move |direction| {
            if actions_enabled {
                let direction = if matches!(direction, SwipeDirection::Left | SwipeDirection::Right)
                    && review.animation.direction() != SwipeDirection::None
                {
                    SwipeDirection::None
                } else {
                    direction
                };
                review.dispatch(ReviewAction::Swipe {
                    direction,
                    at_ms: get_timestamp_ms(),
                });
            }
        })
    };
    let on_key = {
        let act = act.clone();
        let drag_surface = drag_surface.clone();
        Callback::from(move |event: KeyboardEvent| {
            if event
                .target()
                .and_then(|target| target.dyn_into::<Element>().ok())
                != drag_surface.cast::<Element>()
                || event.repeat()
                || event.alt_key()
                || event.ctrl_key()
                || event.meta_key()
            {
                return;
            }
            let direction = match event.key().as_str() {
                "ArrowLeft" => SwipeDirection::Left,
                "ArrowRight" => SwipeDirection::Right,
                "ArrowUp" => SwipeDirection::Up,
                "ArrowDown" => SwipeDirection::Down,
                _ => return,
            };
            event.prevent_default();
            act.emit(direction);
        })
    };
    let progress_label = if is_initial_state {
        "Guide".to_owned()
    } else {
        format!("{skill_level}: {rounded_progress_card_number}/{next_skill_level} | Guide")
    };

    html! {
        <>
            <button type="button" class="progress-bar" onclick={open_settings_cb}
                aria-haspopup="dialog" aria-label={format!("Open guide. {progress_label}")}
                style={format!("--progress:{progress_percentage}%")}>
                <span>{progress_label}</span>
            </button>
            <div ref={drag_surface} class="flashcard-view" tabindex="0" role="region" aria-label="Flashcard. Use arrow keys to review." onkeydown={on_key}
                onpointerdown={on_pointer_event.clone()}
                onpointermove={on_pointer_event.clone()}
                onpointerup={on_pointer_event.clone()}
                onpointercancel={on_pointer_event.clone()}
                onlostpointercapture={on_pointer_event}
                style={style.clone()}>
                <div class="drag-surface" aria-hidden="true"></div>
                { match &*content {
                    ContentLoadState::Loading => html! { <section class="load-panel" role="status"><h2>{"Loading flashcards..."}</h2></section> },
                    ContentLoadState::Failed(message) => html! {
                        <section class="load-panel" role="alert">
                            <h2>{"Flashcards could not be loaded"}</h2><p>{message}</p>
                            <button onclick={retry_loading}>{"Try again"}</button>
                        </section>
                    },
                    ContentLoadState::Ready(_) => html! {
                        <Flashcard
                            swipe_direction={review.animation.direction()}
                            is_moving={drag.is_moving()}
                            pos_x={drag.rotation()}
                            pos_y={drag.offset().y}
                            fast_move={review.animation.fast_move()}
                            current_card={review.presentation.card.clone()}
                            processed_sentence={review.presentation.sentence.clone()}
                            show_explanations={review.show_explanations()}
                            processed_explanation={review.presentation.explanation.clone()}
                        />
                    },
                } }

            </div>
            <nav class="review-controls" aria-label="Review actions">
                <button type="button" class="down" aria-disabled={(!actions_enabled).to_string()} onclick={act.reform(|_| SwipeDirection::Down)}>{if review.is_question() { "Skip / next" } else { "Incorrect / next" }}</button>
                <button type="button" class="side" aria-disabled={(!actions_enabled).to_string()} onclick={act.reform(|_| SwipeDirection::Left)}>{"Reveal / flip"}</button>
                <button type="button" class="up" aria-disabled={(!actions_enabled).to_string()} onclick={act.reform(|_| SwipeDirection::Up)}>{if review.is_question() { "Start / next" } else { "Correct / next" }}</button>
            </nav>
            <div class="app-notices">
                { if let ContentLoadState::Ready(loaded) = &*content {
                    loaded.notice.as_ref().map(|notice| html! { <aside class="app-notice" role="status">{notice}</aside> }).unwrap_or_default()
                } else { Html::default() } }
                { match &*save_status {
                    SaveStatus::Saved => Html::default(),
                    SaveStatus::Warning(message) => html! { <aside class="app-notice" role="status">{message}</aside> },
                    SaveStatus::ReadOnly(message) => html! { <aside class="app-notice" role="alert">{message}</aside> },
                    SaveStatus::Unsaved(message) => html! { <aside class="app-notice" role="alert"><p>{message}</p><button onclick={retry_saving}>{"Retry saving"}</button></aside> },
                } }
            </div>
        </>
    }
}

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

fn move_drag(drag: &UseReducerHandle<DragState>, pointer_id: i32, position: Point) {
    let Some(viewport_width) = window()
        .and_then(|window| window.inner_width().ok())
        .and_then(|width| width.as_f64())
        .filter(|width| *width > 0.0)
    else {
        drag.dispatch(DragAction::End(pointer_id));
        web_sys::console::error_1(&"Could not read viewport width; drag was cancelled".into());
        return;
    };
    drag.dispatch(DragAction::Move {
        pointer_id,
        position,
        viewport_width,
    });
}

fn save_review_result(
    progress: &UseStateHandle<SavedProgress>,
    review: &ReviewState,
    step: &animation::AnimationStep,
    save_status: &UseStateHandle<SaveStatus>,
) {
    if let Some(updated) = calculate_review_result(progress, review, step) {
        save_progress_and_update_status(&updated, save_status);
        progress.set(updated);
    }
}

/// Translate an accepted review into learning inputs without browser effects.
fn calculate_review_result(
    progress: &SavedProgress,
    review: &ReviewState,
    step: &animation::AnimationStep,
) -> Option<SavedProgress> {
    let AnimationStepKind::Advance(direction) = &step.kind else {
        return None;
    };
    let attempt = ReviewAttempt {
        elapsed_ms: review.duration(),
        // Preserve the original byte-length measurement, including accented text.
        character_length: (review.presentation.card.base_language.len()
            + review.presentation.card.target_language.len()) as i32,
        outcome: if *direction == SwipeDirection::Up {
            ReviewOutcome::Correct
        } else {
            ReviewOutcome::Incorrect
        },
    };
    apply_review(
        progress,
        &review.presentation.review_words,
        &attempt,
        step.started_at_ms / 1000,
    )
}

fn save_progress_and_update_status(progress: &SavedProgress, status: &UseStateHandle<SaveStatus>) {
    if matches!(&**status, SaveStatus::ReadOnly(_)) {
        return;
    }
    status.set(match storage::save_progress(progress) {
        Ok(()) => SaveStatus::Saved,
        Err(error) => SaveStatus::Unsaved(format!("{error} Your progress is kept in this session, but may be lost if you reload or close it.")),
    });
}

/// Prepare presentation once when advancing, preserving selection and sentence rules.
fn prepare_next_review(
    content: &LearningContent,
    scores: &HashMap<String, ScoreData>,
    previous_review_words: &[Card],
    now_s: i64,
    sentence_sample: f64,
) -> Option<ReviewPresentation> {
    let key = find_key_of_lowest_score(&content.cards, scores, previous_review_words, now_s)?;
    let card = &content.cards[&key];
    if let Some(index) = choose_sentence_index(
        &card.sentence_indexes,
        previous_review_words.len() as i32,
        sentence_sample,
    ) {
        let sentence = &content.sentences[index as usize];
        let (review_words, tokens) = prepare_sentence(
            scores,
            &content.cards,
            split_text(&sentence.target_language),
            &key,
        );
        let explanation =
            construct_processed_explanation(&review_words, &content.explanations, scores);
        Some(ReviewPresentation {
            card: Card {
                base_language: sentence.base_language.clone(),
                target_language: sentence.target_language.clone(),
            },
            review_words,
            sentence: construct_processed_sentence(&tokens),
            explanation,
        })
    } else {
        Some(ReviewPresentation {
            card: Card {
                base_language: card.base_language.clone(),
                target_language: key,
            },
            // Preserve the existing no-sentence fallback; changing its review words is a separate decision.
            review_words: previous_review_words.to_vec(),
            sentence: html! { <span>{ card.base_language.clone() }</span> },
            explanation: html! { <span>{ " " }</span> },
        })
    }
}

fn get_timestamp_s() -> i64 {
    let now = Date::now();
    let duration = now / 1000.0; // Convert to seconds
    duration as i64
}

fn get_timestamp_ms() -> i64 {
    let now = Date::now();
    now as i64
}

fn construct_processed_sentence(tokens: &[ProcessedToken]) -> Html {
    html! {
        { for tokens.iter().map(|token| {
            if let Some(translation) = &token.translation {
                // Retain the original byte-based padding, including non-ASCII words.
                let padding = (translation.len() + 2).saturating_sub(token.text.len()) / 2;
                let spaces = "\u{00A0}".repeat(padding);
                html! {
                    <span class="hidden-word" role="img" aria-label={format!("Missing word: {translation}")} data-translation={translation.clone()}>
                        <span aria-hidden="true">{spaces.clone()}{&token.text}{spaces}</span>
                    </span>
                }
            } else {
                html! { <span class="visible-word">{&token.text}</span> }
            }
        })}
    }
}

fn construct_processed_explanation(
    review_words: &[Card],
    explanations: &HashMap<String, HashMap<String, String>>,
    scores: &HashMap<String, ScoreData>,
) -> Html {
    let mut nodes = Vec::new();
    for word in review_words {
        let Some(entries) = explanations.get(&word.base_language) else {
            continue;
        };
        // The existing UI shows a header for every nonempty group, even when
        // none of its targets pass the learned/current-word filter.
        if entries.is_empty() {
            continue;
        }
        nodes.push(html! {
            <p class="explanation base-word"><strong>{&word.base_language}</strong></p>
        });
        for (target, text) in entries {
            if target == &word.target_language || scores.contains_key(target) {
                nodes.push(html! {
                    <p class="explanation target-word">
                        <strong>{format!("{target}: ")}</strong>{text}
                    </p>
                });
            }
        }
    }
    html! { <div class="explanation-div" tabindex="0" role="region" aria-label="Word explanations">{for nodes}</div> }
}

fn should_enable_drag_for_event_target(target: &EventTarget) -> bool {
    let Some(element) = target.dyn_ref::<Element>() else {
        return false;
    };
    let in_view = element.closest(".flashcard-view").ok().flatten().is_some();
    let scrollable_explanation = element
        .closest(".explanation-div")
        .ok()
        .flatten()
        .is_some_and(|explanation| explanation.scroll_height() > explanation.client_height());
    in_view && !scrollable_explanation
}

#[cfg(test)]
#[path = "tests/presentation_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/memory_store.rs"]
mod memory_store;
