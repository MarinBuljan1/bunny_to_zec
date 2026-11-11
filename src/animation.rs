//! Animation ordering independent of browser timers. Owned timers preserve the
//! existing 400 ms swipe/reveal, 50 ms reposition, and 100 ms timer-start delays.

use crate::SwipeDirection;

const SWIPE_MS: u32 = 400;
const REPOSITION_MS: u32 = 50;
const START_DELAY_MS: u32 = 100;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum AnimationStepKind {
    Reveal,
    Advance(SwipeDirection),
    Reposition,
    StartQuestion,
}

impl AnimationStepKind {
    fn duration_ms(&self) -> u32 {
        match self {
            Self::Reveal | Self::Advance(_) => SWIPE_MS,
            Self::Reposition => REPOSITION_MS,
            Self::StartQuestion => START_DELAY_MS,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AnimationStep {
    pub generation: u64,
    pub kind: AnimationStepKind,
    pub started_at_ms: i64,
}

impl AnimationStep {
    pub fn delay_ms(&self) -> u32 {
        self.kind.duration_ms()
    }
}

#[derive(Clone, Default, PartialEq)]
pub(crate) struct AnimationState {
    face_direction: SwipeDirection,
    step: Option<AnimationStep>,
    generation: u64,
}

impl AnimationState {
    pub fn step(&self) -> Option<AnimationStep> {
        self.step.clone()
    }

    pub fn accepts_swipes(&self) -> bool {
        matches!(
            self.step.as_ref().map(|step| &step.kind),
            None | Some(AnimationStepKind::Reveal)
        )
    }

    pub fn direction(&self) -> SwipeDirection {
        match self.step.as_ref().map(|step| &step.kind) {
            Some(AnimationStepKind::Advance(direction)) => direction.clone(),
            Some(AnimationStepKind::Reposition) => SwipeDirection::Down,
            Some(AnimationStepKind::StartQuestion) => SwipeDirection::None,
            _ => self.face_direction.clone(),
        }
    }

    pub fn fast_move(&self) -> bool {
        matches!(
            self.step.as_ref().map(|step| &step.kind),
            Some(AnimationStepKind::Reposition)
        )
    }

    /// Reject all new swipes while leaving/repositioning/starting a card. Flips
    /// during a pending reveal keep its original timer instead of restarting it.
    pub fn accept_swipe(&mut self, direction: SwipeDirection, at_ms: i64) -> bool {
        if !self.accepts_swipes() {
            return false;
        }
        match direction {
            SwipeDirection::Up | SwipeDirection::Down => {
                self.schedule(AnimationStepKind::Advance(direction), at_ms)
            }
            face => self.face_direction = face,
        }
        true
    }

    pub fn begin_reveal(&mut self, at_ms: i64) {
        if self.step.is_none() {
            self.schedule(AnimationStepKind::Reveal, at_ms);
        }
    }

    /// Return the completed step only for the currently pending timer. Even a
    /// callback already queued when cancellation occurs cannot advance a later card.
    pub fn elapse(&mut self, generation: u64, at_ms: i64) -> Option<AnimationStepKind> {
        let step = self.step.as_ref()?;
        if step.generation != generation {
            return None;
        }
        let completed = step.kind.clone();
        match completed {
            AnimationStepKind::Reveal => self.step = None,
            AnimationStepKind::Advance(_) => self.schedule(AnimationStepKind::Reposition, at_ms),
            AnimationStepKind::Reposition => {
                self.face_direction = SwipeDirection::None;
                self.schedule(AnimationStepKind::StartQuestion, at_ms);
            }
            AnimationStepKind::StartQuestion => self.step = None,
        }
        Some(completed)
    }

    pub fn can_present(&self, generation: u64) -> bool {
        self.step.as_ref().is_some_and(|step| {
            step.generation == generation && step.kind == AnimationStepKind::Reposition
        })
    }

    fn schedule(&mut self, kind: AnimationStepKind, started_at_ms: i64) {
        self.generation = self.generation.wrapping_add(1);
        self.step = Some(AnimationStep {
            generation: self.generation,
            kind,
            started_at_ms,
        });
    }
}

#[cfg(test)]
#[path = "tests/animation_tests.rs"]
mod tests;
