//! Conversation screen. Browser timers and gestures dispatch pure review actions.
use crate::conversations::{
    typing_delay_ms, AnswerFace, ChatAction, ConversationLibrary, ConversationReview, Rating,
    SessionPhase,
};
use crate::gestures::at_bottom;
use crate::message_bubble::MessageBubble;
use crate::model::{LearningContent, SavedProgress};
use crate::storage::{self, ContentLoadState};
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
    Ready(Box<Ready>),
}
enum Action {
    Loaded(Box<Ready>),
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
        match action {
            Action::Loaded(ready) => Rc::new(Self::Ready(ready)),
            Action::Failed(error) => Rc::new(Self::Failed(error)),
            action => {
                let Self::Ready(current) = self.as_ref() else {
                    return self;
                };
                let mut ready = current.clone();
                match action {
                    Action::Saved(revision, result) if revision == ready.revision => {
                        ready.save_error = result.err()
                    }
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
                        if ready.review.progress != current.review.progress {
                            ready.revision += 1;
                        }
                    }
                    Action::Timed { turn, action } => {
                        if ready
                            .review
                            .session()
                            .map(|s| (&s.conversation_id, s.exchange_index))
                            != Some((&turn.0, turn.1))
                        {
                            return self;
                        }
                        if !ready.review.act(
                            action,
                            &ready.content.conversations,
                            &ready.content.vocabulary,
                            Date::now() as i64,
                            Math::random(),
                        ) {
                            return self;
                        }
                        if ready.review.progress != current.review.progress {
                            ready.revision += 1;
                        }
                    }
                    _ => return self,
                }
                Rc::new(Self::Ready(ready))
            }
        }
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
    let can_pull = use_mut_ref(|| false);
    let following = use_mut_ref(|| true);
    let ready = match &*screen {
        Screen::Ready(ready) => Some(ready),
        _ => None,
    };
    *can_pull.borrow_mut() = !props.guide_open && ready.is_some_and(|r| r.review.can_next());
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
                            let Some(ContentLoadState::Ready(loaded)) =
                                storage::commit_content_load(Ok(loaded), &task_cancelled)
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
                            screen.dispatch(Action::Loaded(Box::new(Ready {
                                content: Rc::new(Content {
                                    vocabulary: loaded.content,
                                    conversations,
                                }),
                                review,
                                notice: (!notices.is_empty()).then(|| notices.join(" ")),
                                read_only,
                                save_error: None,
                                revision: 0,
                            })));
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
    let pull = crate::gestures::use_conversation_scroll(
        timeline.clone(),
        ready.is_some(),
        if *can_pull.borrow() {
            let screen = screen.clone();
            Some(Callback::from(move |action| {
                screen.dispatch(Action::Review(action))
            }))
        } else {
            None
        },
    );
    let send = {
        let screen = screen.clone();
        let blocked = props.guide_open;
        Callback::from(move |action| {
            if !blocked {
                screen.dispatch(Action::Review(action));
            }
        })
    };
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
        hint,
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
    let button = |label: &str, action| {
        let send = send.clone();
        html! {<button type="button" disabled={props.guide_open || settling.is_some()} onclick={Callback::from(move |_| send.emit(action))}>{label}</button>}
    };
    html! {
        <main class={classes!("chat-app", ((ready.review.can_flip() || session.is_some_and(|s| s.phase == SessionPhase::Ready)) && !props.guide_open).then_some("gesture-active"))}
            onkeydown={keyboard}
            {onpointerdown} {onpointermove} {onpointerup} onpointercancel={cancel.clone()} onlostpointercapture={cancel}>
            <header class="chat-header"><div><h1>{"Bunny to Zec"}</h1><p>{conversation.map(|c| c.title.as_str()).unwrap_or("Croatian conversations")}</p></div>
                <button type="button" onclick={props.open_settings.clone()} aria-label="Learning progress and guide">{format!("{skill_level}: {learned_words:.1}/{next_level} · Guide")}</button>
            </header>
            <div class="chat-toolbar"><button type="button" aria-pressed={english.to_string()} onclick={{ let send = send.clone(); Callback::from(move |_| send.emit(ChatAction::ToggleEnglish)) }}>{if english { "English on" } else { "English off" }}</button>
                {session.map(|s| html!{<span>{format!("Reply {} of {}", s.exchange_index + 1, conversation.map_or(0, |c| c.exchanges.len()))}</span>}).unwrap_or_default()}
            </div>
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
                                <MessageBubble typing={learner_typing} class={classes!("learner-message")} label="Learner message" layout_key={format!("{learner_typing}-{masked}-{english}-{notes_open}")} on_resized={bubble_resized.clone()}>
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
                } else { html!{<div class="chat-welcome"><h2>{"Learn through conversation"}</h2><p>{"Read your partner’s message, recall the missing Croatian words, then reveal and rate your reply."}</p></div>} }}
            {if session.is_some_and(|s| s.phase == SessionPhase::Ready) { html!{
                <div class="pull-preview" ref={pull.preview.clone()} aria-hidden="true" role="status" aria-label="Pull for next messages">
                    <div class="pull-dot"></div>
                </div>
            }} else { Html::default() }}
            </section>
            </div>
            <footer class="chat-footer">
                {ready.notice.as_ref().map(|notice| html!{<p class="chat-notice" role="status">{notice}</p>}).unwrap_or_default()}
                {ready.save_error.as_ref().map(|error| { let screen = screen.clone(); html!{<div class="chat-notice" role="alert"><p>{format!("Progress has not been saved: {error}")}</p><button onclick={Callback::from(move |_| screen.dispatch(Action::RetrySave))}>{"Retry saving"}</button></div>} }).unwrap_or_default()}
                <div class="chat-controls">{match session.map(|s| &s.phase) {
                    None => button("Start conversation", ChatAction::Next),
                    Some(SessionPhase::PartnerTyping) => html!{<p role="status">{"A message is being typed…"}</p>},
                    Some(SessionPhase::Typing) => html!{<p role="status">{"Your reply is being typed…"}</p>},
                    Some(SessionPhase::Question) => button("Reveal / flip", ChatAction::Flip),
                    Some(SessionPhase::Answer {..}) => html!{<>{button("Incorrect ↓", ChatAction::Rate(Rating::Incorrect))}{button("Flip", ChatAction::Flip)}{button("Correct ↑", ChatAction::Rate(Rating::Correct))}</>},
                    Some(SessionPhase::Ready) => html!{<>{button("Flip", ChatAction::Flip)}{button(if conversation.is_some_and(|c| session.is_some_and(|s| s.exchange_index + 1 == c.exchanges.len())) { "Next conversation" } else { "Next messages" }, ChatAction::Next)}</>},
                }}</div>
                <p ref={hint} class="chat-hint" role="status">
                    <span class="normal-gesture-hint">
                        {if ready.review.can_rate() && ready.review.showing_notes() { "Swipe sideways on the message or notes to flip. Long notes scroll; other up/down swipes rate." } else { match session.map(|s| &s.phase) {
                            Some(SessionPhase::Ready) => "Swipe sideways for word explanations. Pull up at the bottom for the next messages.",
                            Some(SessionPhase::Answer {..}) => "Swipe up for correct, down to practice again. Tap any message to toggle English.",
                            Some(SessionPhase::Question) => "Swipe left or right to reveal. Tap any message to toggle English.",
                            _ => "Tap any message to turn English on or off for every message.",
                        }}}
                    </span>
                    <span class="gesture-release-hint" aria-hidden="true"></span>
                </p>
            </footer>
        </main>
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
