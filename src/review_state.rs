//! Local review and interaction state. Browser clocks and timers stay in the UI.

use std::rc::Rc;

use yew::prelude::*;

use crate::animation::{AnimationState, AnimationStepKind};
use crate::model::Card;
use crate::SwipeDirection;

#[derive(Clone, PartialEq)]
pub(super) enum ReviewKind {
    Welcome,
    Exercise,
}

#[derive(Clone, PartialEq)]
pub(super) struct ReviewPresentation {
    pub card: Card,
    pub review_words: Vec<Card>,
    pub sentence: Html,
    pub explanation: Html,
}

#[derive(Clone, PartialEq)]
enum ExplanationVisibility {
    Hidden,
    Visible,
}

#[derive(Clone, PartialEq)]
enum ReviewPhase {
    Question {
        started_at: i64,
    },
    Answer {
        duration: i64,
        explanation: ExplanationVisibility,
    },
}

/// Presentation is prepared once per card, never during drag updates.
#[derive(Clone, PartialEq)]
pub(super) struct ReviewState {
    pub kind: ReviewKind,
    pub presentation: ReviewPresentation,
    pub animation: AnimationState,
    phase: ReviewPhase,
}

pub(super) enum ReviewAction {
    Swipe {
        direction: SwipeDirection,
        at_ms: i64,
    },
    AnimationElapsed {
        generation: u64,
        at_ms: i64,
    },
    PresentNext {
        generation: u64,
        presentation: ReviewPresentation,
    },
}

impl ReviewState {
    pub fn welcome(now: i64) -> Self {
        Self {
            kind: ReviewKind::Welcome,
            presentation: ReviewPresentation {
                card: Card {
                    base_language: "Swipe to Begin".into(),
                    target_language: "Swipe Up or Down".into(),
                },
                review_words: vec![],
                sentence: html! { "Swipe to Begin" },
                explanation: html! { "Swipe Up or Down" },
            },
            phase: ReviewPhase::Question { started_at: now },
            animation: AnimationState::default(),
        }
    }

    pub fn duration(&self) -> i64 {
        match self.phase {
            ReviewPhase::Question { .. } => 0,
            ReviewPhase::Answer { duration, .. } => duration,
        }
    }

    pub fn is_question(&self) -> bool {
        matches!(self.phase, ReviewPhase::Question { .. })
    }

    pub fn show_explanations(&self) -> bool {
        matches!(
            self.phase,
            ReviewPhase::Answer {
                explanation: ExplanationVisibility::Visible,
                ..
            }
        )
    }
}

impl Reducible for ReviewState {
    type Action = ReviewAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next = (*self).clone();
        match action {
            ReviewAction::PresentNext {
                generation,
                presentation,
            } if self.animation.can_present(generation) => {
                next.kind = ReviewKind::Exercise;
                next.presentation = presentation;
            }
            ReviewAction::Swipe { direction, at_ms } => {
                let reveals_answer =
                    matches!(direction, SwipeDirection::Left | SwipeDirection::Right)
                        && self.is_question();
                if !next.animation.accept_swipe(direction, at_ms) {
                    return self;
                }
                if reveals_answer {
                    let ReviewPhase::Question { started_at } = self.phase else {
                        return self;
                    };
                    next.phase = ReviewPhase::Answer {
                        duration: at_ms - started_at,
                        explanation: ExplanationVisibility::Hidden,
                    };
                    next.animation.begin_reveal(at_ms);
                }
            }
            ReviewAction::AnimationElapsed { generation, at_ms } => {
                match next.animation.elapse(generation, at_ms) {
                    Some(AnimationStepKind::Reveal) => {
                        if let ReviewPhase::Answer { explanation, .. } = &mut next.phase {
                            *explanation = ExplanationVisibility::Visible;
                        }
                    }
                    Some(AnimationStepKind::Reposition) => {
                        if let ReviewPhase::Answer { explanation, .. } = &mut next.phase {
                            *explanation = ExplanationVisibility::Hidden;
                        }
                    }
                    Some(AnimationStepKind::StartQuestion) => {
                        next.phase = ReviewPhase::Question { started_at: at_ms }
                    }
                    Some(AnimationStepKind::Advance(_)) => {}
                    None => return self,
                }
            }
            ReviewAction::PresentNext { .. } => return self,
        }
        next.into()
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
pub(super) struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Default, PartialEq)]
pub(super) enum DragState {
    #[default]
    Idle,
    Active {
        pointer_id: i32,
        origin: Point,
        position: Point,
        viewport_width: f64,
    },
}

pub(super) enum DragAction {
    Start {
        pointer_id: i32,
        origin: Point,
    },
    Move {
        pointer_id: i32,
        position: Point,
        viewport_width: f64,
    },
    End(i32),
}

impl DragState {
    pub fn is_moving(&self) -> bool {
        matches!(self, Self::Active { .. })
    }

    pub fn is_active_pointer(&self, pointer_id: i32) -> bool {
        matches!(self, Self::Active { pointer_id: active, .. } if *active == pointer_id)
    }

    pub fn offset(&self) -> Point {
        match self {
            Self::Idle => Point::default(),
            Self::Active {
                origin, position, ..
            } => Point {
                x: position.x - origin.x,
                y: position.y - origin.y,
            },
        }
    }

    pub fn rotation(&self) -> f64 {
        match self {
            Self::Idle => 0.0,
            Self::Active { viewport_width, .. } => self.offset().x / viewport_width * 90.0,
        }
    }

    pub fn intensity(&self) -> f64 {
        swipe_intensity(self.offset())
    }

    pub fn direction(&self) -> SwipeDirection {
        if self.is_moving() {
            direction(self.offset())
        } else {
            SwipeDirection::None
        }
    }

    pub fn completed_direction(
        &self,
        pointer_id: i32,
        end: Point,
        current: &SwipeDirection,
    ) -> Option<SwipeDirection> {
        if !self.is_active_pointer(pointer_id) {
            return None;
        }
        let Self::Active { origin, .. } = self else {
            return None;
        };
        let offset = Point {
            x: end.x - origin.x,
            y: end.y - origin.y,
        };
        if swipe_intensity(offset) < 1.0 {
            return None;
        }
        let next = direction(offset);
        if matches!(next, SwipeDirection::Left | SwipeDirection::Right)
            && *current != SwipeDirection::None
        {
            Some(SwipeDirection::None)
        } else {
            Some(next)
        }
    }
}

impl Reducible for DragState {
    type Action = DragAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            DragAction::Start { pointer_id, origin } if !self.is_moving() => Self::Active {
                pointer_id,
                origin,
                position: origin,
                viewport_width: 1.0,
            }
            .into(),
            DragAction::Move {
                pointer_id,
                position,
                viewport_width,
            } if self.is_active_pointer(pointer_id) => match *self {
                Self::Active { origin, .. } => Self::Active {
                    pointer_id,
                    origin,
                    position,
                    viewport_width,
                }
                .into(),
                Self::Idle => self,
            },
            DragAction::End(pointer_id) if self.is_active_pointer(pointer_id) => Self::Idle.into(),
            _ => self,
        }
    }
}

fn direction(offset: Point) -> SwipeDirection {
    if offset.x.abs() > offset.y.abs() {
        if offset.x >= 0.0 {
            SwipeDirection::Right
        } else {
            SwipeDirection::Left
        }
    } else if offset.y >= 0.0 {
        SwipeDirection::Down
    } else {
        SwipeDirection::Up
    }
}

fn swipe_intensity(offset: Point) -> f64 {
    // Preserve the existing axis-difference threshold, including diagonal drags.
    (offset.x.abs() - offset.y.abs()).abs().min(50.0) / 50.0
}

#[cfg(test)]
#[path = "tests/review_state_tests.rs"]
mod tests;
