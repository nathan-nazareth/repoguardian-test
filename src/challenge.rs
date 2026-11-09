//! Game challenges between players.

use crate::clock::{Speed, TimeControl};
use crate::piece::Color;

/// The chess variant a game is played in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Variant {
    Standard,
    Chess960,
    KingOfTheHill,
    ThreeCheck,
    Antichess,
    Atomic,
    Horde,
    RacingKings,
    Crazyhouse,
}

impl Variant {
    pub fn name(self) -> &'static str {
        match self {
            Variant::Standard => "standard",
            Variant::Chess960 => "chess960",
            Variant::KingOfTheHill => "kingOfTheHill",
            Variant::ThreeCheck => "threeCheck",
            Variant::Antichess => "antichess",
            Variant::Atomic => "atomic",
            Variant::Horde => "horde",
            Variant::RacingKings => "racingKings",
            Variant::Crazyhouse => "crazyhouse",
        }
    }
}

/// The lifecycle of a challenge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChallengeStatus {
    Created,
    Accepted,
    Declined,
    Canceled,
}

/// A challenge from one player to another (or an open challenge).
#[derive(Debug, Clone)]
pub struct Challenge {
    pub id: String,
    pub challenger: String,
    /// `None` means an open challenge.
    pub opponent: Option<String>,
    pub variant: Variant,
    pub time_control: TimeControl,
    pub rated: bool,
    /// The colour the challenger will play, if fixed.
    pub color: Option<Color>,
    pub status: ChallengeStatus,
}

impl Challenge {
    pub fn new(
        id: impl Into<String>,
        challenger: impl Into<String>,
        opponent: Option<String>,
        time_control: TimeControl,
        rated: bool,
    ) -> Challenge {
        Challenge {
            id: id.into(),
            challenger: challenger.into(),
            opponent,
            variant: Variant::Standard,
            time_control,
            rated,
            color: None,
            status: ChallengeStatus::Created,
        }
    }

    pub fn speed(&self) -> Speed {
        self.time_control.speed()
    }

    pub fn accept(&mut self) -> Result<(), String> {
        if self.status != ChallengeStatus::Created {
            return Err("challenge is no longer pending".to_string());
        }
        self.status = ChallengeStatus::Accepted;
        Ok(())
    }

    pub fn decline(&mut self) -> Result<(), String> {
        if self.status != ChallengeStatus::Created {
            return Err("challenge is no longer pending".to_string());
        }
        self.status = ChallengeStatus::Declined;
        Ok(())
    }

    pub fn cancel(&mut self) -> Result<(), String> {
        if self.status != ChallengeStatus::Created {
            return Err("challenge is no longer pending".to_string());
        }
        self.status = ChallengeStatus::Canceled;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepting_moves_the_state_forward() {
        let mut challenge = Challenge::new(
            "c1",
            "alice",
            Some("bob".to_string()),
            TimeControl::new(300, 0),
            true,
        );
        assert_eq!(challenge.speed(), Speed::Blitz);
        challenge.accept().unwrap();
        assert_eq!(challenge.status, ChallengeStatus::Accepted);
        assert!(challenge.accept().is_err());
    }

    #[test]
    fn open_challenges_have_no_opponent() {
        let challenge = Challenge::new("c2", "alice", None, TimeControl::new(60, 0), false);
        assert!(challenge.opponent.is_none());
        assert_eq!(challenge.speed(), Speed::Bullet);
    }
}
