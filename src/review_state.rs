//! Local review and interaction state. Browser clocks and timers stay in the UI.

use std::rc::Rc;

use yew::prelude::*;

use super::{Card, CurrentTranslations, SwipeDirection};

#[derive(Clone, PartialEq)]
pub(super) enum ReviewKind {
    Welcome,
    Exercise,
}

#[derive(Clone, PartialEq)]
pub(super) struct ReviewPresentation {
    pub card: Card,
    pub translations: CurrentTranslations,
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
    phase: ReviewPhase,
}

pub(super) enum ReviewAction {
    Present(ReviewPresentation),
    Reveal(i64),
    ShowExplanations,
    HideExplanations,
    StartTimer(i64),
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
                translations: CurrentTranslations { list: vec![] },
                sentence: html! { "Swipe to Begin" },
                explanation: html! { "Swipe Up or Down" },
            },
            phase: ReviewPhase::Question { started_at: now },
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
            ReviewAction::Present(presentation) => {
                next.kind = ReviewKind::Exercise;
                next.presentation = presentation;
            }
            ReviewAction::Reveal(now) => {
                // Flipping an already revealed answer must not restart its timer.
                if let ReviewPhase::Question { started_at } = self.phase {
                    next.phase = ReviewPhase::Answer {
                        duration: now - started_at,
                        explanation: ExplanationVisibility::Hidden,
                    };
                }
            }
            ReviewAction::ShowExplanations => {
                if let ReviewPhase::Answer { explanation, .. } = &mut next.phase {
                    *explanation = ExplanationVisibility::Visible;
                }
            }
            ReviewAction::HideExplanations => {
                if let ReviewPhase::Answer { explanation, .. } = &mut next.phase {
                    *explanation = ExplanationVisibility::Hidden;
                }
            }
            ReviewAction::StartTimer(started_at) => {
                next.phase = ReviewPhase::Question { started_at };
            }
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
        origin: Point,
        position: Point,
        viewport_width: f64,
    },
}

pub(super) enum DragAction {
    Start(Point),
    Move {
        position: Point,
        viewport_width: f64,
    },
    End,
}

impl DragState {
    pub fn is_moving(&self) -> bool {
        matches!(self, Self::Active { .. })
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
        end: Point,
        current: &SwipeDirection,
    ) -> Option<SwipeDirection> {
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
            DragAction::Start(origin) => Self::Active {
                origin,
                position: origin,
                viewport_width: 1.0,
            }
            .into(),
            DragAction::Move {
                position,
                viewport_width,
            } => match *self {
                Self::Active { origin, .. } => Self::Active {
                    origin,
                    position,
                    viewport_width,
                }
                .into(),
                Self::Idle => self,
            },
            DragAction::End => Self::Idle.into(),
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

#[derive(Clone, Default, PartialEq)]
pub(super) enum AnimationState {
    #[default]
    Settled,
    Swiping(SwipeDirection),
    Repositioning,
}

impl AnimationState {
    pub fn direction(&self) -> SwipeDirection {
        match self {
            Self::Settled => SwipeDirection::None,
            Self::Swiping(direction) => direction.clone(),
            Self::Repositioning => SwipeDirection::Down,
        }
    }
}

#[cfg(test)]
#[path = "tests/review_state_tests.rs"]
mod tests;
