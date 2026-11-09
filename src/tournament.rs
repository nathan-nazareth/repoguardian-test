//! Arena tournaments.

use std::collections::BTreeMap;

use crate::game::Outcome;
use crate::piece::Color;

/// The lifecycle of an arena tournament.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArenaStatus {
    Created,
    Started,
    Finished,
}

/// A Lichess-style arena: players join and score two points per win.
#[derive(Debug, Clone)]
pub struct Arena {
    pub id: String,
    pub name: String,
    pub rounds: u32,
    pub status: ArenaStatus,
    scores: BTreeMap<String, i32>,
}

impl Arena {
    pub fn new(id: impl Into<String>, name: impl Into<String>, rounds: u32) -> Arena {
        Arena {
            id: id.into(),
            name: name.into(),
            rounds,
            status: ArenaStatus::Created,
            scores: BTreeMap::new(),
        }
    }

    pub fn join(&mut self, user: &str) {
        self.scores.entry(user.to_string()).or_insert(0);
    }

    pub fn withdraw(&mut self, user: &str) {
        self.scores.remove(user);
    }

    pub fn is_member(&self, user: &str) -> bool {
        self.scores.contains_key(user)
    }

    pub fn start(&mut self) -> Result<(), String> {
        if self.status != ArenaStatus::Created {
            return Err("tournament already started".to_string());
        }
        self.status = ArenaStatus::Started;
        Ok(())
    }

    pub fn finish(&mut self) {
        self.status = ArenaStatus::Finished;
    }

    /// Records a finished game; a win is worth two points and a draw one.
    pub fn record_result(&mut self, white: &str, black: &str, outcome: Outcome) {
        self.join(white);
        self.join(black);
        match outcome.winner() {
            Some(Color::White) => *self.scores.get_mut(white).unwrap() += 2,
            Some(Color::Black) => *self.scores.get_mut(black).unwrap() += 2,
            None => {
                *self.scores.get_mut(white).unwrap() += 1;
                *self.scores.get_mut(black).unwrap() += 1;
            }
        }
    }

    pub fn score(&self, user: &str) -> i32 {
        self.scores.get(user).copied().unwrap_or(0)
    }

    /// The standings, best first (ties broken alphabetically).
    pub fn standings(&self) -> Vec<(String, i32)> {
        let mut table: Vec<(String, i32)> = self
            .scores
            .iter()
            .map(|(name, score)| (name.clone(), *score))
            .collect();
        table.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        table
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::DrawReason;

    #[test]
    fn arena_scores_matches() {
        let mut arena = Arena::new("t1", "Test Arena", 5);
        arena.start().unwrap();
        arena.record_result("alice", "bob", Outcome::Checkmate(Color::White));
        arena.record_result("alice", "carol", Outcome::Draw(DrawReason::Agreement));

        assert_eq!(arena.score("alice"), 3);
        assert_eq!(arena.score("bob"), 0);
        assert_eq!(arena.score("carol"), 1);
        assert_eq!(arena.standings().first().unwrap().0, "alice");
    }

    #[test]
    fn members_can_withdraw() {
        let mut arena = Arena::new("t2", "Test Arena", 5);
        arena.join("alice");
        assert!(arena.is_member("alice"));
        arena.withdraw("alice");
        assert!(!arena.is_member("alice"));
    }

    #[test]
    fn cannot_start_twice() {
        let mut arena = Arena::new("t3", "Test Arena", 5);
        arena.start().unwrap();
        assert!(arena.start().is_err());
    }
}
