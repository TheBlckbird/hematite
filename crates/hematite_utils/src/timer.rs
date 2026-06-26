use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Timer {
    duration: Duration,
    start_time: Instant,
    mode: TimerMode,
    has_finished: bool,
}

impl Timer {
    pub fn new(duration: Duration, mode: TimerMode) -> Self {
        Self {
            duration,
            start_time: Instant::now(),
            mode,
            has_finished: false,
        }
    }

    /// Tick the timer and check whether it has finished
    pub fn tick(&mut self) -> bool {
        let now = Instant::now();
        let time_difference = now - self.start_time;

        if time_difference >= self.duration && !self.has_finished {
            match self.mode {
                TimerMode::Once => self.has_finished = true,
                TimerMode::Repeating => self.start_time = now - (time_difference - self.duration),
            }

            true
        } else {
            false
        }
    }

    /// Reset the timer to restart it
    pub fn reset(&mut self) {
        self.start_time = Instant::now();
        self.has_finished = false;
    }

    /// Whether a timer has finished.
    ///
    /// This will only worw for `TimerMode::Once`
    pub fn has_finished(&self) -> bool {
        self.has_finished
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TimerMode {
    Once,
    Repeating,
}
