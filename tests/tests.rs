// use yew_project::{Card, CardList, find_related_cards, replace_with_blanks, ScoreData};
// use std::collections::HashMap;

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_replace_with_blanks() {
//         let examples = vec![
//             (
//                 "Hello, world! How are you?",
//                 "world",
//                 "Hello, _! How are you?",
//             ),
//             (
//                 "Rust is great. I love Rust!",
//                 "Rust",
//                 "_ is great. I love _!",
//             ),
//             (
//                 "This is a simple test.",
//                 "simple",
//                 "This is a _ test.",
//             ),
//             (
//                 "No matching word here.",
//                 "notfound",
//                 "No matching word here.",
//             ),
//             (
//                 "The quick brown fox jumps over the lazy dog.",
//                 "fox",
//                 "The quick brown _ jumps over the lazy dog.",
//             ),
//             (
//                 "i think this is a good litmus test.",
//                 "i",
//                 "_ think this is a good litmus test.",
//             ),
//         ];

//         for (input, back_word, expected) in examples {
//             let result = replace_with_blanks(input, back_word);
//             assert_eq!(result, expected, "Failed on input: '{}', back_word: '{}'", input, back_word);
//         }
//     }
// }
