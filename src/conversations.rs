//! Conversation selection, masking and session transitions, independent of the browser.

use crate::learning::{
    apply_review, calculate_final_score, ReviewAttempt, ReviewOutcome, LEARNED_INTERVAL_SECONDS,
};
use crate::model::{Card, LearningContent, SavedProgress, ScoreData};
use crate::sentences::split_text;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

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
}

impl ConversationLibrary {
    /// Verify the supplied index against the actual learner text before using weights.
    pub fn validate(&self, vocabulary: &LearningContent) -> Result<(), String> {
        if self.schema_version != 1
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
        for conversation in &self.conversations {
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
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&Conversation> {
        self.conversations.iter().find(|c| c.id == id)
    }

    pub fn choose<'a>(
        &'a self,
        content: &LearningContent,
        progress: &SavedProgress,
        previous: &[Card],
        now_s: i64,
        sample: f64,
    ) -> Option<&'a Conversation> {
        let candidates = || {
            content
                .cards
                .keys()
                .filter(|key| self.word_index.contains_key(*key))
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
        let minimum = hits.iter().map(count).min()?;
        let eligible: Vec<_> = hits.iter().filter(|hit| count(hit) == minimum).collect();
        let total: u64 = eligible.iter().map(|hit| u64::from(hit.hits)).sum();
        let sample = if sample.is_finite() {
            sample.clamp(0.0, 1.0 - f64::EPSILON)
        } else {
            0.0
        };
        let mut choice = (sample * total as f64) as u64;
        for hit in eligible {
            if choice < u64::from(hit.hits) {
                return self.get(&hit.c_id);
            }
            choice -= u64::from(hit.hits);
        }
        None
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
    let words: HashSet<_> = split_text(text)
        .into_iter()
        .map(|word| word.to_lowercase())
        .filter(|word| content.cards.contains_key(word))
        .collect();
    let due: HashSet<_> = words
        .iter()
        .filter(|word| {
            progress
                .scores
                .get(*word)
                .is_some_and(|s| s.score < LEARNED_INTERVAL_SECONDS)
        })
        .collect();
    let unseen: HashSet<_> = words
        .iter()
        .filter(|word| !progress.scores.contains_key(*word))
        .collect();
    let target = content
        .cards
        .keys()
        .filter(|key| {
            if !due.is_empty() {
                due.contains(key)
            } else if !unseen.is_empty() {
                unseen.contains(key)
            } else {
                words.contains(*key)
            }
        })
        .min_by_key(|key| priority(progress.scores.get(*key), now_s));
    let mut included = HashSet::new();
    split_text(text)
        .into_iter()
        .filter_map(|word| {
            let key = word.to_lowercase();
            if (progress.scores.contains_key(&key) || target == Some(&key))
                && included.insert(key.clone())
            {
                content.cards.get(&key).map(|card| Card {
                    target_language: key,
                    base_language: card.base_language.clone(),
                })
            } else {
                None
            }
        })
        .collect()
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub(crate) enum AnswerFace {
    Masked,
    Revealed,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub(crate) enum SessionPhase {
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
    LearnerAppears,
    Flip,
    Rate(Rating),
    Settled,
    ToggleEnglish,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ConversationReview {
    pub progress: SavedProgress,
    pub settling: Option<Rating>,
    started_at_ms: Option<i64>,
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
            progress.conversations.session = None;
            Some("The saved conversation changed. Your word progress and completed conversation counts were kept.".into())
        } else {
            None
        };
        let started_at_ms = progress
            .conversations
            .session
            .as_ref()
            .filter(|s| s.phase == SessionPhase::Question)
            .map(|_| now_ms);
        (
            Self {
                progress,
                settling: None,
                started_at_ms,
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

    pub fn act(
        &mut self,
        action: ChatAction,
        library: &ConversationLibrary,
        content: &LearningContent,
        now_ms: i64,
        sample: f64,
    ) -> bool {
        match action {
            ChatAction::ToggleEnglish => {
                self.progress.conversations.show_english =
                    !self.progress.conversations.show_english;
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
                        let Some(c) = library.choose(
                            content,
                            &self.progress,
                            &previous,
                            now_ms / 1000,
                            sample,
                        ) else {
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
                self.progress.conversations.session = Some(ConversationSession {
                    conversation_id: conversation.id.clone(),
                    exchange_index: index,
                    learner_text: text.clone(),
                    phase: SessionPhase::Typing,
                    words,
                    completed,
                });
                self.started_at_ms = None;
            }
            ChatAction::LearnerAppears => {
                let Some(session) = self
                    .progress
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
            ChatAction::Flip => {
                let Some(session) = self.progress.conversations.session.as_mut() else {
                    return false;
                };
                session.phase = match session.phase {
                    SessionPhase::Question => SessionPhase::Answer {
                        elapsed_ms: now_ms
                            .saturating_sub(self.started_at_ms.take().unwrap_or(now_ms))
                            .max(0),
                        face: AnswerFace::Revealed,
                    },
                    SessionPhase::Answer { elapsed_ms, face } => SessionPhase::Answer {
                        elapsed_ms,
                        face: match face {
                            AnswerFace::Masked => AnswerFace::Revealed,
                            AnswerFace::Revealed => AnswerFace::Masked,
                        },
                    },
                    _ => return false,
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
                self.progress = updated;
            }
        }
        let Some(session) = self.progress.conversations.session.as_mut() else {
            return false;
        };
        session.completed.push(CompletedReply { words, rating });
        session.phase = SessionPhase::Ready;
        if session.exchange_index + 1 == conversation.exchanges.len() {
            let count = self
                .progress
                .conversations
                .read_counts
                .entry(conversation.id.clone())
                .or_default();
            *count = count.saturating_add(1);
        }
        self.settling = (rating != Rating::Read).then_some(rating);
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
