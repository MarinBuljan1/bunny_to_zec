use super::*;

#[test]
fn test_split_text() {
    let examples_split_text = vec![
        (
            "Hello, world! Rust is great.",
            vec![
                "Hello", ",", " ", "world", "!", " ", "Rust", " ", "is", " ", "great", ".",
            ],
        ),
        (
            "Hey! How's it going?",
            vec!["Hey", "!", " ", "How's", " ", "it", " ", "going", "?"],
        ),
        ("No punctuation", vec!["No", " ", "punctuation"]),
        ("A, B, C!", vec!["A", ",", " ", "B", ",", " ", "C", "!"]),
        (
            "  Spacing  Test  ",
            vec![" ", " ", "Spacing", " ", " ", "Test", " ", " "],
        ),
    ];

    for (input, expected) in examples_split_text {
        let result = split_text(input);
        assert_eq!(result, expected, "Failed on input: '{input}'");
    }
}

#[test]
fn scanner_matches_the_original_regex_on_the_sentence_corpus_and_boundary_inputs() {
    // Keep an independent reference for the previous behavior, including its pipe quirk.
    let punctuation = [' ', ',', '.', '!', '?']
        .iter()
        .map(|character| regex::escape(&character.to_string()))
        .collect::<Vec<_>>()
        .join("|");
    let reference = regex::Regex::new(&format!("({punctuation})|([^{punctuation}]+)")).unwrap();
    let check = |text: &str| {
        let expected: Vec<String> = reference
            .find_iter(text)
            .map(|matched| matched.as_str().to_owned())
            .collect();
        assert_eq!(split_text(text), expected, "Tokenizer changed for {text:?}");
    };
    let library: crate::conversations::ConversationLibrary =
        serde_json::from_str(include_str!("../../assets/conversations.json")).unwrap();
    for conversation in library.conversations {
        for exchange in conversation.exchanges {
            for message in [exchange.partner, exchange.learner] {
                check(&message.hr);
                check(&message.en);
            }
        }
    }
    let fragments = [
        "", "bok", "?????", "don't", " ", "  ", ",", ".", "!", "?", "|", "||", "\t", "\n", "?",
        "??", "[]", "/", ";",
    ];
    for first in fragments {
        for second in fragments {
            for third in fragments {
                check(&format!("{first}{second}{third}"));
            }
        }
    }
}
