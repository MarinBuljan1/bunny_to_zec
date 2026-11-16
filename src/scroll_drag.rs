//! Independent horizontal displacement and incremental vertical scrolling.
pub(crate) struct ScrollDrag {
    origin_x: f64,
    x: f64,
    y: f64,
    time: f64,
    velocity: f64,
}

pub(crate) struct ScrollStep {
    pub vertical: f64,
    pub horizontal: f64,
}

impl ScrollDrag {
    pub fn new(x: f64, y: f64, time: f64) -> Self {
        Self {
            origin_x: x,
            x,
            y,
            time,
            velocity: 0.0,
        }
    }

    pub fn move_to(&mut self, x: f64, y: f64, time: f64) -> ScrollStep {
        let vertical = self.y - y;
        if vertical != 0.0 {
            let elapsed = time - self.time;
            if elapsed > 80.0 || self.velocity.signum() != vertical.signum() {
                self.velocity = 0.0;
            }
            self.velocity = self.velocity * 0.5 + vertical / elapsed.clamp(1.0, 50.0) * 0.5;
            self.time = time;
        } else if x != self.x {
            self.velocity = 0.0;
            self.time = time;
        }
        self.x = x;
        self.y = y;
        ScrollStep {
            vertical,
            horizontal: x - self.origin_x,
        }
    }

    pub fn release_velocity(&self, time: f64) -> f64 {
        if time - self.time < 80.0 {
            self.velocity.clamp(-3.0, 3.0)
        } else {
            0.0
        }
    }
}

#[cfg(test)]
#[path = "tests/scroll_drag_tests.rs"]
mod tests;
