use super::*;

fn ready_screen() -> Rc<Screen> {
    let vocabulary = crate::memory_store::content();
    let conversations: ConversationLibrary = serde_json::from_str(
        r#"{"schema_version":2,"source_language":"hr","translation_language":"en","conversations":[],"word_index":{}}"#,
    )
    .unwrap();
    let progress = SavedProgress {
        scores: HashMap::new(),
        last_data_update: 1000,
        conversations: Default::default(),
    };
    let (review, _) = ConversationReview::restore(progress, &conversations, &vocabulary, 1000000);
    Rc::new(Screen::Ready(Ready {
        content: Rc::new(Content {
            vocabulary,
            conversations,
        }),
        review,
        notice: None,
        read_only: false,
        save_error: None,
        revision: 1,
    }))
}

#[test]
fn save_acknowledgements_and_retries_preserve_the_review_snapshot() {
    let screen = ready_screen();
    let Screen::Ready(original) = screen.as_ref() else {
        panic!("ready fixture");
    };
    let progress = original.review.progress.clone();
    let updated = screen.reduce(Action::Saved(1, Err("storage full".into())));
    let Screen::Ready(ready) = updated.as_ref() else {
        panic!("save keeps the screen ready");
    };
    assert_eq!(ready.save_error.as_deref(), Some("storage full"));
    assert!(Rc::ptr_eq(&progress, &ready.review.progress));

    let retried = updated.reduce(Action::RetrySave);
    let Screen::Ready(ready) = retried.as_ref() else {
        panic!("retry keeps the screen ready");
    };
    assert_eq!(ready.revision, 2);
    assert!(Rc::ptr_eq(&progress, &ready.review.progress));

    let saved = retried.reduce(Action::Saved(2, Ok(())));
    let Screen::Ready(ready) = saved.as_ref() else {
        panic!("save keeps the screen ready");
    };
    assert!(ready.save_error.is_none());
    assert!(Rc::ptr_eq(&progress, &ready.review.progress));
}

#[test]
fn stale_save_and_timer_callbacks_leave_the_screen_untouched() {
    let screen = ready_screen();
    let stale_save = screen
        .clone()
        .reduce(Action::Saved(0, Err("old failure".into())));
    assert!(Rc::ptr_eq(&screen, &stale_save));
    let stale_timer = screen.clone().reduce(Action::Timed {
        turn: ("0001".into(), 0),
        action: ChatAction::LearnerAppears,
    });
    assert!(Rc::ptr_eq(&screen, &stale_timer));
}

#[test]
fn score_bar_puts_the_percentage_on_a_visible_fill_element() {
    use yew::virtual_dom::VNode;

    for (score, width, goal) in [
        (0.0, "width: 0.00%", "100"),
        (30.0, "width: 30.00%", "100"),
        (150.0, "width: 50.00%", "300"),
        (1000.0, "width: 25.00%", "4000"),
    ] {
        let VNode::VTag(bar) = score_bar(score) else {
            panic!("score bar must render a DOM element");
        };
        assert_eq!(
            bar.attributes.iter().find(|(name, _)| *name == "role"),
            Some(("role", "progressbar"))
        );
        assert_eq!(
            bar.attributes
                .iter()
                .find(|(name, _)| *name == "aria-valuemax"),
            Some(("aria-valuemax", goal))
        );
        let Some(VNode::VTag(fill)) = bar.children().first() else {
            panic!("score bar must contain a fill element");
        };
        assert_eq!(
            fill.attributes.iter().find(|(name, _)| *name == "class"),
            Some(("class", "score-fill"))
        );
        assert_eq!(
            fill.attributes.iter().find(|(name, _)| *name == "style"),
            Some(("style", width))
        );
    }
}

#[test]
fn learner_borders_follow_saved_ratings_and_leave_ungraded_messages_neutral() {
    for (rating, class) in [
        (None, "learner-message"),
        (Some(Rating::Read), "learner-message"),
        (Some(Rating::Correct), "learner-message rated-correct"),
        (Some(Rating::Incorrect), "learner-message rated-incorrect"),
    ] {
        assert_eq!(learner_message_class(rating).to_string(), class);
    }
}

#[test]
fn conversation_exit_is_only_requested_after_the_last_reply_has_settled() {
    let screen = ready_screen();
    let Screen::Ready(original) = screen.as_ref() else {
        panic!("ready fixture");
    };
    let mut ready = original.clone();
    let vocabulary = crate::memory_store::content();
    let exchange = serde_json::json!({
        "partner": { "hr": "Bok?", "en": "Hello?" },
        "learner": { "hr": "Bok!", "en": "Hello!" },
    });
    let mut conversations: ConversationLibrary = serde_json::from_value(serde_json::json!({
        "schema_version": 2, "source_language": "hr", "translation_language": "en",
        "conversations": [{ "id": "0001", "title": "Hello", "band": 1, "exchanges": vec![exchange; 5] }],
        "word_index": { "bok": [{ "c_id": "0001", "hits": 5 }] },
    }))
    .unwrap();
    conversations.validate_and_rank(&vocabulary).unwrap();
    ready.content = Rc::new(Content {
        vocabulary,
        conversations,
    });
    assert!(!starts_new_conversation(&ready));
    for index in 0..5 {
        for (step, action) in [
            ChatAction::Next,
            ChatAction::PartnerAppears,
            ChatAction::LearnerAppears,
            ChatAction::Flip,
            ChatAction::Rate(Rating::Correct),
        ]
        .into_iter()
        .enumerate()
        {
            assert!(ready.review.act(
                action,
                &ready.content.conversations,
                &ready.content.vocabulary,
                1000000 + index * 10000 + step as i64 * 1000
            ));
        }
        assert!(!starts_new_conversation(&ready));
        ready.review.act(
            ChatAction::Settled,
            &ready.content.conversations,
            &ready.content.vocabulary,
            1005000 + index * 10000,
        );
        assert_eq!(starts_new_conversation(&ready), index == 4);
    }
    // A new run of the same ID still leaves the completed conversation behind.
    ready.review.act(
        ChatAction::Next,
        &ready.content.conversations,
        &ready.content.vocabulary,
        1060000,
    );
    assert_eq!(ready.review.session().unwrap().conversation_id, "0001");
    assert_eq!(ready.review.session().unwrap().exchange_index, 0);
    assert!(!starts_new_conversation(&ready));
}
