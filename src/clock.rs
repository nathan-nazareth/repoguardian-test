//! Chess clocks and time controls.

use std::time::Duration;

use crate::piece::Color;

/// The Lichess "speed" bucket a time control falls into.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Speed {
    UltraBullet,
    Bullet,
    Blitz,
    Rapid,
    Classical,
    Correspondence,
}

impl Speed {
    pub fn name(self) -> &'static str {
        match self {
            Speed::UltraBullet => "ultraBullet",
            Speed::Bullet => "bullet",
            Speed::Blitz => "blitz",
            Speed::Rapid => "rapid",
            Speed::Classical => "classical",
            Speed::Correspondence => "correspondence",
        }
    }
}

/// A time control: an initial time plus a per-move increment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeControl {
    pub initial: Duration,
    pub increment: Duration,
}

impl TimeControl {
    pub const fn new(initial_secs: u64, increment_secs: u64) -> TimeControl {
        TimeControl {
            initial: Duration::from_secs(initial_secs),
            increment: Duration::from_secs(increment_secs),
        }
    }

    /// Lichess estimates a game at 40 moves when bucketing time controls.
    pub fn estimated_duration(&self) -> Duration {
        self.initial + self.increment * 40
    }

    pub fn speed(&self) -> Speed {
        let initial_secs = self.initial.as_secs();
        let estimated = self.estimated_duration().as_secs() as f64;
        if initial_secs >= 24 * 60 * 60 {
            Speed::Correspondence
        } else if estimated < 30.0 {
            Speed::UltraBullet
        } else if estimated < 180.0 {
            Speed::Bullet
        } else if estimated < 480.0 {
            Speed::Blitz
        } else if estimated < 1500.0 {
            Speed::Rapid
        } else {
            Speed::Classical
        }
    }
}

/// A running clock for both colours.
#[derive(Debug, Clone)]
pub struct Clock {
    pub time_control: TimeControl,
    white: Duration,
    black: Duration,
    running: Option<Color>,
}

impl Clock {
    pub fn new(time_control: TimeControl) -> Clock {
        Clock {
            time_control,
            white: time_control.initial,
            black: time_control.initial,
            running: None,
        }
    }

    pub fn remaining(&self, color: Color) -> Duration {
        match color {
            Color::White => self.white,
            Color::Black => self.black,
        }
    }

    fn set(&mut self, color: Color, value: Duration) {
        match color {
            Color::White => self.white = value,
            Color::Black => self.black = value,
        }
    }

    pub fn turn(&self) -> Option<Color> {
        self.running
    }

    pub fn is_running(&self) -> bool {
        self.running.is_some()
    }

    /// Starts (or resumes) the clock for `color`.
    pub fn start(&mut self, color: Color) {
        self.running = Some(color);
    }

    /// Stops the clock.
    pub fn stop(&mut self) {
        self.running = None;
    }

    /// Consumes `elapsed` from `color`; returns `true` when the flag fell.
    pub fn consume(&mut self, color: Color, elapsed: Duration) -> bool {
        let remaining = self.remaining(color);
        if elapsed >= remaining {
            self.set(color, Duration::ZERO);
            true
        } else {
            self.set(color, remaining - elapsed);
            false
        }
    }

    /// Applies the increment and hands the move to the opponent.
    pub fn finish_move(&mut self, color: Color) {
        let remaining = self.remaining(color) + self.time_control.increment;
        self.set(color, remaining);
        self.running = Some(color.opponent());
    }

    pub fn is_flagged(&self, color: Color) -> bool {
        self.remaining(color).is_zero()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn speed_buckets() {
        assert_eq!(TimeControl::new(20, 0).speed(), Speed::UltraBullet);
        assert_eq!(TimeControl::new(60, 0).speed(), Speed::Bullet);
        assert_eq!(TimeControl::new(180, 0).speed(), Speed::Blitz);
        assert_eq!(TimeControl::new(600, 5).speed(), Speed::Rapid);
        assert_eq!(TimeControl::new(1800, 0).speed(), Speed::Classical);
    }

    #[test]
    fn clock_deducts_and_applies_increment() {
        let mut clock = Clock::new(TimeControl::new(60, 2));
        clock.start(Color::White);
        assert!(!clock.consume(Color::White, Duration::from_secs(5)));
        assert_eq!(clock.remaining(Color::White), Duration::from_secs(55));
        clock.finish_move(Color::White);
        assert_eq!(clock.remaining(Color::White), Duration::from_secs(57));
        assert_eq!(clock.turn(), Some(Color::Black));
    }

    #[test]
    fn clock_flags() {
        let mut clock = Clock::new(TimeControl::new(1, 0));
        assert!(clock.consume(Color::Black, Duration::from_secs(1)));
        assert!(clock.is_flagged(Color::Black));
    }
}
