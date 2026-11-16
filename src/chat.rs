//! Conversation screen. Browser timers and gestures dispatch pure review actions.
use crate::conversations::{
    typing_delay_ms, AnswerFace, ChatAction, ConversationLibrary, ConversationReview, Rating,
    SessionPhase,
};
use crate::gestures::at_bottom;
use crate::message_bubble::MessageBubble;
use crate::model::{LearningContent, SavedProgress};
use crate::storage;
use gloo_timers::callback::Timeout;
use js_sys::{Date, Math};
use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
use web_sys::HtmlElement;
use yew::prelude::*;

struct Content {
    vocabulary: LearningContent,
    conversations: ConversationLibrary,
}
#[derive(Clone)]
struct Ready {
    content: Rc<Content>,
    review: ConversationReview,
    notice: Option<String>,
    read_only: bool,
    save_error: Option<String>,
    revision: u64,
}
#[derive(Clone, Default)]
enum Screen {
    #[default]
    Loading,
    Failed(String),
    Ready(Ready),
}
enum Action {
    Loaded(Ready),
    Failed(String),
    Review(ChatAction),
    Timed {
        turn: (String, usize),
        action: ChatAction,
    },
    Saved(u64, Result<(), String>),
    RetrySave,
}
impl Reducible for Screen {
    type Action = Action;
    fn reduce(self: Rc<Self>, action: Action) -> Rc<Self> {
        // Reject stale timer/save callbacks before copying any state.
        let action = match action {
            Action::Loaded(ready) => return Rc::new(Self::Ready(ready)),
            Action::Failed(error) => return Rc::new(Self::Failed(error)),
            Action::Timed { turn, action } => {
                let Self::Ready(ready) = self.as_ref() else {
                    return self;
                };
                if ready
                    .review
                    .session()
                    .map(|s| (&s.conversation_id, s.exchange_index))
                    != Some((&turn.0, turn.1))
                {
                    return self;
                }
                Action::Review(action)
            }
            action => action,
        };
        let Self::Ready(current) = self.as_ref() else {
            return self;
        };
        if let Action::Saved(revision, _) = &action {
            if *revision != current.revision {
                return self;
            }
        }
        // Saved progress is shared until an action actually changes persistent state.
        let mut ready = current.clone();
        match action {
            Action::Saved(_, result) => ready.save_error = result.err(),
            Action::RetrySave => ready.revision += 1,
            Action::Review(action) => {
                if !ready.review.act(
                    action,
                    &ready.content.conversations,
                    &ready.content.vocabulary,
                    Date::now() as i64,
                    Math::random(),
                ) {
                    return self;
                }
                if !Rc::ptr_eq(&ready.review.progress, &current.review.progress) {
                    ready.revision += 1;
                }
            }
            _ => return self,
        }
        Rc::new(Self::Ready(ready))
    }
}

#[derive(Properties, PartialEq)]
pub(super) struct ConversationViewProps {
    pub guide_open: bool,
    pub open_settings: Callback<MouseEvent>,
}

#[function_component(ConversationView)]
pub(super) fn conversation_view(props: &ConversationViewProps) -> Html {
    let screen = use_reducer(Screen::default);
    let reload = use_state(|| 0_u32);
    let timeline = use_node_ref();
    let following = use_mut_ref(|| true);
    let ready = match &*screen {
        Screen::Ready(ready) => Some(ready),
        _ => None,
    };
    let can_pull = !props.guide_open && ready.is_some_and(|r| r.review.can_next());
    {
        let screen = screen.clone();
        use_effect_with_deps(
            move |_| {
                let cancelled = Rc::new(Cell::new(false));
                let task_cancelled = cancelled.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let loaded = async {
                        let content = storage::load_learning_content().await?;
                        let mut conversations = storage::load_conversations().await?;
                        conversations.validate_and_rank(&content.content)?;
                        Ok::<_, String>((content, conversations))
                    }
                    .await;
                    if task_cancelled.get() {
                        return;
                    }
                    match loaded {
                        Err(error) => screen.dispatch(Action::Failed(error)),
                        Ok((loaded, conversations)) => {
                            let Some(loaded) =
                                storage::commit_content_load(loaded, &task_cancelled)
                            else {
                                return;
                            };
                            let (progress, read_only, mut notices) = match storage::load_progress(Date::now() as i64 / 1000) {
                            Ok(loaded) => (loaded.progress, false, loaded.warning.into_iter().collect::<Vec<_>>()),
                            Err(error) => (SavedProgress { scores: HashMap::new(), last_data_update: Date::now() as i64 / 1000, conversations: Default::default() }, true,
                                vec![format!("Your saved progress could not be read: {error} This session will not overwrite it.")]),
                        };
                            notices.extend(loaded.notice);
                            let (review, notice) = ConversationReview::restore(
                                progress,
                                &conversations,
                                &loaded.content,
                                Date::now() as i64,
                            );
                            notices.extend(notice);
                            screen.dispatch(Action::Loaded(Ready {
                                content: Rc::new(Content {
                                    vocabulary: loaded.content,
                                    conversations,
                                }),
                                review,
                                notice: (!notices.is_empty()).then(|| notices.join(" ")),
                                read_only,
                                save_error: None,
                                revision: 0,
                            }));
                        }
                    }
                });
                move || cancelled.set(true)
            },
            *reload,
        );
    }
    {
        let screen = screen.clone();
        let revision = ready.map(|r| r.revision);
        use_effect_with_deps(
            move |_| {
                if let Screen::Ready(ready) = &*screen {
                    if ready.revision > 0 && !ready.read_only {
                        screen.dispatch(Action::Saved(
                            ready.revision,
                            storage::save_progress(&ready.review.progress)
                                .map_err(|e| e.to_string()),
                        ));
                    }
                }
                || ()
            },
            revision,
        );
    }
    let turn = ready
        .and_then(|r| r.review.session())
        .map(|s| (s.conversation_id.clone(), s.exchange_index));
    let typing_phase = ready
        .and_then(|r| r.review.session())
        .map(|s| s.phase.clone())
        .filter(|phase| matches!(phase, SessionPhase::PartnerTyping | SessionPhase::Typing));
    let settling = ready.and_then(|r| r.review.settling);
    {
        let screen = screen.clone();
        let delay = ready
            .and_then(|r| {
                r.review.session().and_then(|s| {
                    r.content
                        .conversations
                        .get(&s.conversation_id)
                        .map(|c| typing_delay_ms(&c.exchanges[s.exchange_index].partner.hr))
                })
            })
            .unwrap_or(1200);
        use_effect_with_deps(
            move |(turn, typing_phase, settling, guide_open)| {
                let timer = if *guide_open {
                    None
                } else {
                    turn.clone().and_then(|turn| {
                        let (delay, action) = match typing_phase {
                            Some(SessionPhase::PartnerTyping) => (1000, ChatAction::PartnerAppears),
                            Some(SessionPhase::Typing) => (delay, ChatAction::LearnerAppears),
                            _ if settling.is_some() => (280, ChatAction::Settled),
                            _ => return None,
                        };
                        Some(Timeout::new(delay, move || {
                            screen.dispatch(Action::Timed { turn, action })
                        }))
                    })
                };
                move || drop(timer)
            },
            (
                turn.clone(),
                typing_phase.clone(),
                settling,
                props.guide_open,
            ),
        );
    }
    {
        let timeline = timeline.clone();
        let following = following.clone();
        // New exchanges always follow the conversation; an arriving reply respects a reader scrolling back.
        let force = typing_phase == Some(SessionPhase::PartnerTyping);
        use_effect_with_deps(
            move |_| {
                if let Some(element) = timeline.cast::<HtmlElement>() {
                    if force || *following.borrow() {
                        element.set_scroll_top(element.scroll_height());
                    }
                }
                || ()
            },
            (
                turn.clone(),
                typing_phase.clone(),
                ready
                    .and_then(|r| r.review.session())
                    .is_some_and(|s| s.phase == SessionPhase::Ready),
                ready.map(|r| r.review.progress.conversations.show_english),
            ),
        );
    }
    let transition =
        crate::conversation_transition::use_conversation_transition(timeline.clone(), turn.clone());
    let leaving_conversation = ready.is_some_and(starts_new_conversation);
    let send = {
        let screen = screen.clone();
        let blocked = props.guide_open;
        let capture = transition.capture.clone();
        Callback::from(move |action| {
            if !blocked {
                if action == ChatAction::Next && leaving_conversation {
                    capture.emit(());
                }
                screen.dispatch(Action::Review(action));
            }
        })
    };
    let pull = crate::gestures::use_conversation_scroll(
        timeline.clone(),
        ready.is_some(),
        can_pull.then(|| send.clone()),
    );
    let onscroll = {
        let following = following.clone();
        let timeline = timeline.clone();
        Callback::from(move |event: Event| {
            if let Some(element) = event.target_dyn_into::<HtmlElement>() {
                if timeline.cast::<HtmlElement>().as_ref() == Some(&element) {
                    *following.borrow_mut() = at_bottom(&element);
                }
            }
        })
    };
    let crate::gestures::ReviewGestures {
        motion,
        onpointerdown,
        onpointermove,
        onpointerup,
        cancel,
        tap: toggle,
        tap_start,
    } = crate::gestures::use_review_gestures(
        if props.guide_open
            || !ready.is_some_and(|r| {
                r.review.can_flip()
                    || r.review
                        .session()
                        .is_some_and(|s| s.phase == SessionPhase::Ready)
            })
        {
            crate::gestures::GestureMode::Disabled
        } else if ready.is_some_and(|r| r.review.can_rate()) {
            crate::gestures::GestureMode::Rate
        } else {
            crate::gestures::GestureMode::Flip
        },
        send.clone(),
        pull.clone(),
    );
    let keyboard = {
        let send = send.clone();
        let can_rate = ready.is_some_and(|r| r.review.can_rate());
        let can_next = ready.is_some_and(|r| r.review.can_next());
        let timeline = timeline.clone();
        Callback::from(move |event: KeyboardEvent| {
            if event.repeat() {
                return;
            }
            if event.target_dyn_into::<HtmlElement>().is_some_and(|e| {
                e.tag_name() == "BUTTON" || e.closest(".message-notes").ok().flatten().is_some()
            }) {
                return;
            }
            let action = match event.key().as_str() {
                "ArrowLeft" | "ArrowRight" => Some(ChatAction::Flip),
                "ArrowUp" if can_rate => Some(ChatAction::Rate(Rating::Correct)),
                "ArrowDown" if can_rate => Some(ChatAction::Rate(Rating::Incorrect)),
                "ArrowUp"
                    if can_next
                        && timeline
                            .cast::<HtmlElement>()
                            .is_some_and(|e| at_bottom(&e)) =>
                {
                    Some(ChatAction::Next)
                }
                _ => None,
            };
            if let Some(action) = action {
                event.prevent_default();
                send.emit(action);
            }
        })
    };
    let bubble_resized = {
        let timeline = timeline.clone();
        let following = following.clone();
        Callback::from(move |_| {
            if *following.borrow() {
                if let Some(element) = timeline.cast::<HtmlElement>() {
                    element.set_scroll_top(element.scroll_height());
                }
            }
        })
    };
    let retry = {
        let reload = reload.clone();
        Callback::from(move |_| reload.set(*reload + 1))
    };
    let Some(ready) = ready else {
        return html! { <main class="chat-app"><h1>{"Bunny to Zec"}</h1>{match &*screen {
            Screen::Failed(error) => html!{<div role="alert"><p>{error}</p><button onclick={retry}>{"Try again"}</button></div>},
            _ => html!{<p role="status">{"Loading conversations…"}</p>},
        }}</main> };
    };
    let session = ready.review.session();
    let conversation = session.and_then(|s| ready.content.conversations.get(&s.conversation_id));
    let english = ready.review.progress.conversations.show_english;
    let learned_words = crate::learning::calculate_progress_sum(&ready.review.progress.scores)
        as f64
        / crate::learning::LEARNED_INTERVAL_SECONDS as f64;
    html! {
        <main class={classes!("chat-app", ((ready.review.can_flip() || session.is_some_and(|s| s.phase == SessionPhase::Ready)) && !props.guide_open).then_some("gesture-active"))}
            onkeydown={keyboard}
            {onpointerdown} {onpointermove} {onpointerup} onpointercancel={cancel.clone()} onlostpointercapture={cancel}>
            <header class="chat-header">
                <button type="button" class={classes!("english-toggle", (!english).then_some("english-off"))}
                    aria-label={if english { "English translations on. Turn off" } else { "English translations off. Turn on" }}
                    aria-pressed={english.to_string()}
                    onclick={send.reform(|_| ChatAction::ToggleEnglish)}>{"EN"}</button>
                {score_bar(learned_words)}
                <button type="button" class="guide-button" onclick={props.open_settings.clone()} aria-haspopup="dialog">{"Guide"}</button>
            </header>
            {if ready.notice.is_some() || ready.save_error.is_some() { html!{
                <aside class="chat-notices" aria-label="App notices">
                    {ready.notice.as_ref().map(|notice| html!{<p class="chat-notice" role="status">{notice}</p>}).unwrap_or_default()}
                    {ready.save_error.as_ref().map(|error| { let screen = screen.clone(); html!{<div class="chat-notice" role="alert"><p>{format!("Progress has not been saved: {error}")}</p><button onclick={Callback::from(move |_| screen.dispatch(Action::RetrySave))}>{"Retry saving"}</button></div>} }).unwrap_or_default()}
                </aside>
            }} else { Html::default() }}
            <div class="chat-history">
            <section ref={timeline} class="chat-timeline" tabindex="0" aria-label="Conversation messages" {onscroll}>
                {if let (Some(session), Some(conversation)) = (session, conversation) {
                    conversation.exchanges.iter().take(session.exchange_index + 1).enumerate().map(|(index, exchange)| {
                        let active = index == session.exchange_index;
                        let partner_typing = active && session.phase == SessionPhase::PartnerTyping;
                        let learner_typing = active && session.phase == SessionPhase::Typing;
                        let masked = active && matches!(session.phase, SessionPhase::Question | SessionPhase::Answer { face: AnswerFace::Masked, .. });
                        let words = if active { session.words.as_slice() } else { session.completed.get(index).map_or(&[] as &[_], |reply| reply.words.as_slice()) };
                        let notes_open = active && ready.review.showing_notes();
                        let rating = session.completed.get(index).map(|reply| reply.rating);
                        let motion_class = classes!("message-motion", "learner-motion");
                        html!{<div class="chat-exchange" key={format!("{}-{index}", conversation.id)}>
                            <div class="message-motion partner-motion"><MessageBubble typing={partner_typing} class={classes!("partner-message")} label="Partner message" layout_key={format!("{partner_typing}-{english}")} start_size={partner_typing.then_some((36.0, 36.0))} on_resized={bubble_resized.clone()}>
                                {if partner_typing { typing_indicator("Partner is typing") } else { html!{
                                    <button class="message-text" type="button" onpointerdown={tap_start.clone()} onclick={toggle.clone()} title="Toggle English for all messages" aria-pressed={english.to_string()}>
                                        <span lang="hr">{&exchange.partner.hr}</span>
                                        {if english { html!{<span class="message-translation" lang="en">{&exchange.partner.en}</span>} } else { Html::default() }}
                                    </button>
                                }}}
                            </MessageBubble></div>
                            {if partner_typing { Html::default() } else { html!{
                                <div ref={if active { motion.clone() } else { NodeRef::default() }} class={motion_class}>
                                <MessageBubble typing={learner_typing} class={learner_message_class(rating)} label={match rating {
                                    Some(Rating::Correct) => "Learner message, marked correct",
                                    Some(Rating::Incorrect) => "Learner message, marked incorrect",
                                    _ => "Learner message",
                                }} layout_key={format!("{learner_typing}-{masked}-{english}-{notes_open}")} on_resized={bubble_resized.clone()}>
                                    {if learner_typing { typing_indicator("Learner is typing") } else { html!{
                                        <button class="message-text" type="button" onpointerdown={tap_start.clone()} onclick={toggle.clone()} title="Toggle English for all messages" aria-pressed={english.to_string()}>
                                            <span lang="hr">{annotated_message(&exchange.learner.hr, words, masked)}</span>
                                            {if english { html!{<span class="message-translation" lang="en">{&exchange.learner.en}</span>} } else { Html::default() }}
                                        </button>
                                    }}}
                                    {if notes_open { html!{
                                        <div class="message-notes" tabindex="0" aria-label="Word explanations">
                                            {for crate::word_notes::word_notes(words, &ready.content.vocabulary, &ready.review.progress.scores).iter().map(|note| html!{
                                                <p class="word-note" key={note.form.clone()}><strong lang="hr">{&note.form}</strong>{" — "}<span class="note-translation">{&note.translation}</span>
                                                    {note.explanation.as_ref().map(|explanation| html!{<>{": "}{explanation}</>}).unwrap_or_default()}
                                                </p>
                                            })}
                                        </div>
                                    }} else { Html::default() }}
                                </MessageBubble></div>
                            }}}
                        </div>}
                    }).collect::<Html>()
                } else { html!{<div class="chat-welcome"><h2>{"Learn through conversation"}</h2><p>{"Read your partner’s message, recall the missing Croatian words, then reveal and rate your reply."}</p><button type="button" onclick={send.reform(|_| ChatAction::Next)}>{"Start conversation"}</button></div>} }}
                <div class="pull-preview" ref={pull.preview.clone()} aria-hidden="true" role="status" aria-label="Pull for next messages">
                    <div class="pull-dot"></div>
                </div>
            </section>
            <div ref={transition.outgoing} class="conversation-exit" aria-hidden="true" inert=""></div>
            </div>
        </main>
    }
}

fn starts_new_conversation(ready: &Ready) -> bool {
    ready.review.can_next()
        && ready.review.session().is_some_and(|session| {
            ready
                .content
                .conversations
                .get(&session.conversation_id)
                .is_some_and(|conversation| {
                    session.exchange_index + 1 == conversation.exchanges.len()
                })
        })
}

fn learner_message_class(rating: Option<Rating>) -> Classes {
    classes!(
        "learner-message",
        match rating {
            Some(Rating::Correct) => Some("rated-correct"),
            Some(Rating::Incorrect) => Some("rated-incorrect"),
            _ => None,
        }
    )
}

fn score_bar(learned_words: f64) -> Html {
    let (next_level, skill_level) = [
        (100.0, "Beginner"),
        (300.0, "Novice"),
        (1000.0, "Conversational"),
        (4000.0, "Intermediate"),
        (10000.0, "Advanced"),
        (100000.0, "Fluent"),
    ]
    .into_iter()
    .find(|(threshold, _)| learned_words < *threshold)
    .unwrap_or((100000.0, "Fluent"));
    let progress_percent = (learned_words / next_level * 100.0).clamp(0.0, 100.0);
    html! {
        <div class="chat-score" role="progressbar" aria-label="Progress towards the next learning level"
            aria-valuemin="0" aria-valuemax={next_level.to_string()}
            aria-valuenow={learned_words.min(next_level).to_string()}
            aria-valuetext={format!("{skill_level}: {learned_words:.1} of {next_level}")}>
            <span class="score-fill" aria-hidden="true" style={format!("width: {progress_percent:.2}%")}></span>
            <span class="score-label">{format!("{skill_level}: {learned_words:.1}/{next_level}")}</span>
        </div>
    }
}

fn annotated_message(text: &str, words: &[crate::model::Card], masked: bool) -> Html {
    crate::sentences::split_text(text).into_iter().map(|token| {
        let key = token.to_lowercase();
        match words.iter().find(|word| word.target_language == key) {
            Some(word) => html!{<span class="review-word"><span class="word-hint">{&word.base_language}</span>
                <span class="word-line" aria-label={if masked { format!("Missing Croatian word: {}", word.base_language) } else { token.clone() }}>
                    <span class={masked.then_some("concealed-word")} aria-hidden={masked.to_string()}>{token}</span>
                </span>
            </span>},
            None => html!{token},
        }
    }).collect()
}

fn typing_indicator(label: &str) -> Html {
    html! {<div class="typing-indicator" role="status" aria-label={label.to_owned()}><i/><i/><i/></div>}
}

#[cfg(test)]
#[path = "tests/chat_tests.rs"]
mod tests;
