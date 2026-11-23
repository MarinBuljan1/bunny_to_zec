//! Conversation selection, masking and session transitions, independent of the browser.

use crate::learning::{
    apply_review, calculate_final_score, calculate_progress_sum, learning_level, ReviewAttempt,
    ReviewOutcome, LEARNED_INTERVAL_SECONDS,
};
use crate::model::{Card, LearningContent, SavedProgress, ScoreData};
use crate::sentences::split_text;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct Message {
    pub hr: String,
    pub en: String,
}
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct Exchange {
    pub partner: Message,
    pub learner: Message,
}
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct Conversation {
    pub id: String,
    pub title: String,
    /// Editorial difficulty, validated as 1 (Starter) through 4 (Complex).
    band: u8,
    pub exchanges: Vec<Exchange>,
}
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct Hit {
    pub c_id: String,
    pub hits: u32,
}
#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct ConversationLibrary {
    schema_version: u32,
    source_language: String,
    translation_language: String,
    pub conversations: Vec<Conversation>,
    word_index: HashMap<String, Vec<Hit>>,
    #[serde(skip)]
    difficulties: HashMap<String, ConversationDifficulty>,
}

#[derive(Debug, PartialEq)]
struct ConversationDifficulty {
    band: u8,
    average_word_rank: f64,
}

impl ConversationLibrary {
    /// Verify bands and the supplied index before using them for selection.
    pub fn validate_and_rank(&mut self, vocabulary: &LearningContent) -> Result<(), String> {
        if self.schema_version != 2
            || self.source_language != "hr"
            || self.translation_language != "en"
        {
            return Err("Unsupported conversation format or languages.".into());
        }
        if self.conversations.is_empty() {
            return Err("The conversation library is empty.".into());
        }
        let mut ids = HashSet::new();
        let mut actual: HashMap<String, Vec<Hit>> = HashMap::new();
        let mut starter_has_vocabulary = false;
        for conversation in &self.conversations {
            if !(1..=4).contains(&conversation.band) {
                return Err(format!(
                    "Conversation {} has an invalid difficulty band.",
                    conversation.id
                ));
            }
            if conversation.id.is_empty()
                || !ids.insert(&conversation.id)
                || conversation.title.trim().is_empty()
                || !(5..=8).contains(&conversation.exchanges.len())
            {
                return Err("A conversation has an invalid ID, title or exchange count.".into());
            }
            let mut counts: HashMap<String, u32> = HashMap::new();
            for exchange in &conversation.exchanges {
                for message in [&exchange.partner, &exchange.learner] {
                    if [&message.hr, &message.en]
                        .iter()
                        .any(|s| s.trim().is_empty() || s.chars().count() > 1000)
                    {
                        return Err(format!(
                            "Conversation {} has an invalid message.",
                            conversation.id
                        ));
                    }
                }
                for word in split_text(&exchange.learner.hr)
                    .into_iter()
                    .map(|word| word.to_lowercase())
                {
                    if vocabulary.cards.contains_key(&word) {
                        *counts.entry(word).or_default() += 1;
                    }
                }
            }
            starter_has_vocabulary |= conversation.band == 1 && !counts.is_empty();
            for (word, hits) in counts {
                actual.entry(word).or_default().push(Hit {
                    c_id: conversation.id.clone(),
                    hits,
                });
            }
        }
        if actual != self.word_index {
            return Err("Conversation selection index does not match the learner messages.".into());
        }
        if !starter_has_vocabulary {
            return Err(
                "The conversation library has no Starter conversation with reviewable vocabulary."
                    .into(),
            );
        }
        self.difficulties = self
            .conversations
            .iter()
            .map(|conversation| {
                (
                    conversation.id.clone(),
                    ConversationDifficulty {
                        band: conversation.band,
                        average_word_rank: conversation_difficulty(conversation, vocabulary),
                    },
                )
            })
            .collect();
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&Conversation> {
        self.conversations.iter().find(|c| c.id == id)
    }

    /// Gate targets before read counts; harder-only words wait for an eligible level.
    /// Equal-read matches use band, average vocabulary rank, then ID, without randomness.
    pub fn choose<'a>(
        &'a self,
        content: &LearningContent,
        progress: &SavedProgress,
        previous: &[Card],
        now_s: i64,
    ) -> Option<&'a Conversation> {
        let maximum_band = maximum_conversation_band(progress);
        let candidates = || {
            content.cards.keys().filter(|key| {
                self.word_index.get(*key).is_some_and(|hits| {
                    hits.iter().any(|hit| {
                        self.difficulties
                            .get(&hit.c_id)
                            .is_some_and(|difficulty| difficulty.band <= maximum_band)
                    })
                })
            })
        };
        let priority = |key: &&String| priority(progress.scores.get(*key), now_s);
        let target = candidates()
            .filter(|key| !previous.iter().any(|card| &card.target_language == *key))
            .min_by_key(priority)
            .or_else(|| candidates().min_by_key(priority))?;
        let hits = self.word_index.get(target)?;
        let count = |hit: &Hit| {
            progress
                .conversations
                .read_counts
                .get(&hit.c_id)
                .copied()
                .unwrap_or(0)
        };
        let (hit, _) = hits
            .iter()
            .filter_map(|hit| {
                self.difficulties
                    .get(&hit.c_id)
                    .map(|difficulty| (hit, difficulty))
            })
            .filter(|(_, difficulty)| difficulty.band <= maximum_band)
            .min_by(|(left, left_difficulty), (right, right_difficulty)| {
                count(left)
                    .cmp(&count(right))
                    .then_with(|| left_difficulty.band.cmp(&right_difficulty.band))
                    .then_with(|| {
                        left_difficulty
                            .average_word_rank
                            .total_cmp(&right_difficulty.average_word_rank)
                    })
                    .then_with(|| left.c_id.cmp(&right.c_id))
            })?;
        self.get(&hit.c_id)
    }
}

fn maximum_conversation_band(progress: &SavedProgress) -> u8 {
    let learned_words =
        calculate_progress_sum(&progress.scores) as f64 / LEARNED_INTERVAL_SECONDS as f64;
    learning_level(learned_words).maximum_conversation_band
}

/// Every Croatian word occurrence on either side contributes; punctuation and English do not.
fn conversation_difficulty(conversation: &Conversation, content: &LearningContent) -> f64 {
    let hardest = content.cards.len().max(1) as f64;
    let mut total = 0.0;
    let mut count = 0;
    for exchange in &conversation.exchanges {
        for message in [&exchange.partner, &exchange.learner] {
            for token in split_text(&message.hr) {
                if !token.chars().any(char::is_alphanumeric) {
                    continue;
                }
                let rank = content
                    .cards
                    .get_index_of(&token.to_lowercase())
                    .map(|index| (index + 1) as f64)
                    .unwrap_or(hardest);
                total += rank;
                count += 1;
            }
        }
    }
    if count == 0 {
        hardest
    } else {
        total / f64::from(count)
    }
}

fn priority(score: Option<&ScoreData>, now_s: i64) -> i128 {
    score
        .map(|score| calculate_final_score(score, now_s))
        .unwrap_or(i128::from(LEARNED_INTERVAL_SECONDS))
}

/// All occurrences are masked; review/scoring includes each distinct form once.
pub(crate) fn review_words(
    text: &str,
    content: &LearningContent,
    progress: &SavedProgress,
    now_s: i64,
) -> Vec<Card> {
    let mut included = HashSet::new();
    let words: Vec<_> = split_text(text)
        .into_iter()
        .map(|word| word.to_lowercase())
        .filter(|word| included.insert(word.clone()))
        .filter_map(|word| {
            let (rank, _, card) = content.cards.get_full(&word)?;
            Some((word, rank, card))
        })
        .collect();
    let target = words
        .iter()
        .min_by_key(|(word, rank, _)| {
            let score = progress.scores.get(word);
            let group = match score {
                Some(score) if score.score < LEARNED_INTERVAL_SECONDS => 0,
                None => 1,
                Some(_) => 2,
            };
            // Preserve due-before-new priority and vocabulary-order ties while
            // inspecting only forms in this reply, not the whole vocabulary.
            (group, priority(score, now_s), *rank)
        })
        .map(|(_, rank, _)| *rank);
    words
        .into_iter()
        .filter(|(word, rank, _)| progress.scores.contains_key(word) || target == Some(*rank))
        .map(|(word, _, card)| Card {
            target_language: word,
            base_language: card.base_language.clone(),
        })
        .collect()
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub(crate) enum AnswerFace {
    Masked,
    Revealed,
    Notes,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub(crate) enum SessionPhase {
    PartnerTyping,
    Typing,
    Question,
    Answer { elapsed_ms: i64, face: AnswerFace },
    Ready,
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub(crate) enum Rating {
    Correct,
    Incorrect,
    Read,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub(crate) struct CompletedReply {
    pub words: Vec<Card>,
    pub rating: Rating,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub(crate) struct ConversationSession {
    pub conversation_id: String,
    pub exchange_index: usize,
    pub learner_text: String,
    pub phase: SessionPhase,
    pub words: Vec<Card>,
    pub completed: Vec<CompletedReply>,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub(crate) struct ConversationProgress {
    pub read_counts: HashMap<String, u64>,
    pub session: Option<ConversationSession>,
    pub show_english: bool,
}

impl ConversationProgress {
    pub fn structurally_valid(&self) -> bool {
        self.session.as_ref().is_none_or(|session| {
            !session.conversation_id.is_empty() && session.exchange_index < 8
                && session.completed.len() == session.exchange_index + usize::from(session.phase == SessionPhase::Ready)
                && !matches!(session.phase, SessionPhase::Answer { elapsed_ms, .. } if elapsed_ms < 0)
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ChatAction {
    Next,
    PartnerAppears,
    LearnerAppears,
    Flip,
    Rate(Rating),
    Settled,
    ToggleEnglish,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ConversationReview {
    /// Shared across view-only actions; persistent updates detach the snapshot.
    pub progress: Rc<SavedProgress>,
    pub settling: Option<Rating>,
    started_at_ms: Option<i64>,
    notes_visible: bool,
}

impl ConversationReview {
    /// Content edits may invalidate a cursor, but never discard vocabulary scores/read counts.
    pub fn restore(
        mut progress: SavedProgress,
        library: &ConversationLibrary,
        content: &LearningContent,
        now_ms: i64,
    ) -> (Self, Option<String>) {
        let valid = progress.conversations.structurally_valid()
            && progress
                .conversations
                .session
                .as_ref()
                .is_none_or(|session| {
                    library.get(&session.conversation_id).is_some_and(|c| {
                        c.exchanges
                            .get(session.exchange_index)
                            .is_some_and(|exchange| exchange.learner.hr == session.learner_text)
                            && valid_saved_words(&session.words, &session.learner_text, content)
                            && session.completed.iter().zip(&c.exchanges).all(
                                |(reply, exchange)| {
                                    valid_saved_words(&reply.words, &exchange.learner.hr, content)
                                },
                            )
                            && session.completed.len() <= c.exchanges.len()
                    })
                });
        let notice = if !valid {
            Some("The saved conversation changed. Your word progress and completed conversation counts were kept.".into())
        } else if progress
            .conversations
            .session
            .as_ref()
            .is_some_and(|session| {
                library
                    .get(&session.conversation_id)
                    .is_some_and(|conversation| {
                        conversation.band > maximum_conversation_band(&progress)
                    })
            })
        {
            Some("The saved conversation is above your current level. Your word progress and completed conversation counts were kept; the next conversation will match your level.".into())
        } else {
            None
        };
        if notice.is_some() {
            progress.conversations.session = None;
        }
        let started_at_ms = progress
            .conversations
            .session
            .as_ref()
            .filter(|s| s.phase == SessionPhase::Question)
            .map(|_| now_ms);
        (
            Self {
                progress: Rc::new(progress),
                settling: None,
                started_at_ms,
                notes_visible: false,
            },
            notice,
        )
    }

    pub fn session(&self) -> Option<&ConversationSession> {
        self.progress.conversations.session.as_ref()
    }
    pub fn can_next(&self) -> bool {
        self.settling.is_none()
            && self
                .session()
                .is_none_or(|s| s.phase == SessionPhase::Ready)
    }
    pub fn can_rate(&self) -> bool {
        self.settling.is_none()
            && self
                .session()
                .is_some_and(|s| matches!(s.phase, SessionPhase::Answer { .. }))
    }

    pub fn can_flip(&self) -> bool {
        self.settling.is_none()
            && self.session().is_some_and(|s| {
                matches!(
                    s.phase,
                    SessionPhase::Question | SessionPhase::Answer { .. } | SessionPhase::Ready
                )
            })
    }

    pub fn showing_notes(&self) -> bool {
        self.session().is_some_and(|s| {
            matches!(
                s.phase,
                SessionPhase::Answer {
                    face: AnswerFace::Notes,
                    ..
                }
            ) || (matches!(s.phase, SessionPhase::Answer { .. } | SessionPhase::Ready)
                && self.notes_visible)
        })
    }

    pub fn act(
        &mut self,
        action: ChatAction,
        library: &ConversationLibrary,
        content: &LearningContent,
        now_ms: i64,
    ) -> bool {
        match action {
            ChatAction::ToggleEnglish => {
                let progress = Rc::make_mut(&mut self.progress);
                progress.conversations.show_english = !progress.conversations.show_english;
            }
            ChatAction::Settled => {
                if self.settling.take().is_none() {
                    return false;
                }
            }
            ChatAction::Next if self.can_next() => {
                let previous = self.session().map(|s| s.words.clone()).unwrap_or_default();
                let current = self.session().and_then(|s| {
                    library
                        .get(&s.conversation_id)
                        .map(|c| (c, s.exchange_index + 1))
                });
                let (conversation, index) = match current {
                    Some((c, index)) if index < c.exchanges.len() => (c, index),
                    _ => {
                        let Some(c) =
                            library.choose(content, &self.progress, &previous, now_ms / 1000)
                        else {
                            return false;
                        };
                        (c, 0)
                    }
                };
                let completed = if index == 0 {
                    Vec::new()
                } else {
                    self.session()
                        .map(|s| s.completed.clone())
                        .unwrap_or_default()
                };
                let text = &conversation.exchanges[index].learner.hr;
                let words = review_words(text, content, &self.progress, now_ms / 1000);
                Rc::make_mut(&mut self.progress).conversations.session =
                    Some(ConversationSession {
                        conversation_id: conversation.id.clone(),
                        exchange_index: index,
                        learner_text: text.clone(),
                        phase: SessionPhase::PartnerTyping,
                        words,
                        completed,
                    });
                self.started_at_ms = None;
                self.notes_visible = false;
            }
            ChatAction::PartnerAppears => {
                let Some(session) = Rc::make_mut(&mut self.progress)
                    .conversations
                    .session
                    .as_mut()
                    .filter(|s| s.phase == SessionPhase::PartnerTyping)
                else {
                    return false;
                };
                session.phase = SessionPhase::Typing;
            }
            ChatAction::LearnerAppears => {
                let Some(session) = Rc::make_mut(&mut self.progress)
                    .conversations
                    .session
                    .as_mut()
                    .filter(|s| s.phase == SessionPhase::Typing)
                else {
                    return false;
                };
                session.phase = SessionPhase::Question;
                self.started_at_ms = Some(now_ms);
                if session.words.is_empty() {
                    return self.finish_reply(Rating::Read, library, now_ms);
                }
            }
            ChatAction::Flip if self.can_flip() => {
                let elapsed_ms = match self.session().map(|session| session.phase.clone()) {
                    Some(SessionPhase::Question) => now_ms
                        .saturating_sub(self.started_at_ms.take().unwrap_or(now_ms))
                        .max(0),
                    Some(
                        SessionPhase::Answer {
                            face: AnswerFace::Revealed,
                            ..
                        }
                        | SessionPhase::Ready,
                    ) => {
                        // Details are transient: keep sharing the saved snapshot.
                        self.notes_visible = !self.notes_visible;
                        return true;
                    }
                    // Keep the frozen recall time when normalizing an older Notes/Masked face.
                    Some(SessionPhase::Answer { elapsed_ms, .. }) => elapsed_ms,
                    _ => return false,
                };
                self.notes_visible = false;
                let Some(session) = Rc::make_mut(&mut self.progress)
                    .conversations
                    .session
                    .as_mut()
                else {
                    return false;
                };
                session.phase = SessionPhase::Answer {
                    elapsed_ms,
                    face: AnswerFace::Revealed,
                };
            }
            ChatAction::Rate(rating) if self.can_rate() && rating != Rating::Read => {
                return self.finish_reply(rating, library, now_ms)
            }
            _ => return false,
        }
        true
    }

    fn finish_reply(&mut self, rating: Rating, library: &ConversationLibrary, now_ms: i64) -> bool {
        let Some(session) = self.session() else {
            return false;
        };
        let Some(conversation) = library.get(&session.conversation_id) else {
            return false;
        };
        let Some(exchange) = conversation.exchanges.get(session.exchange_index) else {
            return false;
        };
        let elapsed = match session.phase {
            SessionPhase::Answer { elapsed_ms, .. } => elapsed_ms,
            _ if rating == Rating::Read => 0,
            _ => return false,
        };
        let words = session.words.clone();
        if !words.is_empty() {
            let attempt = ReviewAttempt {
                elapsed_ms: elapsed,
                character_length: (exchange.learner.hr.len() + exchange.learner.en.len())
                    .min(i32::MAX as usize) as i32,
                outcome: if rating == Rating::Correct {
                    ReviewOutcome::Correct
                } else {
                    ReviewOutcome::Incorrect
                },
            };
            if let Some(updated) = apply_review(&self.progress, &words, &attempt, now_ms / 1000) {
                self.progress = Rc::new(updated);
            }
        }
        let progress = Rc::make_mut(&mut self.progress);
        let Some(session) = progress.conversations.session.as_mut() else {
            return false;
        };
        session.completed.push(CompletedReply { words, rating });
        session.phase = SessionPhase::Ready;
        if session.exchange_index + 1 == conversation.exchanges.len() {
            let count = progress
                .conversations
                .read_counts
                .entry(conversation.id.clone())
                .or_default();
            *count = count.saturating_add(1);
        }
        self.settling = (rating != Rating::Read).then_some(rating);
        self.notes_visible = false;
        true
    }
}

/// Stored targets must still be distinct, canonical vocabulary forms in this message.
fn valid_saved_words(words: &[Card], text: &str, content: &LearningContent) -> bool {
    let present: HashSet<_> = split_text(text)
        .into_iter()
        .map(|token| token.to_lowercase())
        .collect();
    let mut seen = HashSet::new();
    words.iter().all(|word| {
        seen.insert(&word.target_language)
            && present.contains(&word.target_language)
            && content
                .cards
                .get(&word.target_language)
                .is_some_and(|card| card.base_language == word.base_language)
    })
}

/// Give the partner a brief reading interval, bounded for very short/long messages.
pub(crate) fn typing_delay_ms(partner: &str) -> u32 {
    (900 + partner.split_whitespace().count() as u32 * 160).clamp(1200, 4000)
}

#[cfg(test)]
#[path = "tests/conversation_tests.rs"]
mod tests;
