//! Tactics puzzles.

use crate::board::Board;
use crate::moves::Move;

/// A tactics puzzle: a position, a solution line and metadata.
#[derive(Debug, Clone)]
pub struct Puzzle {
    pub id: String,
    pub fen: String,
    /// The full solution in UCI. The first move is played by the opponent.
    pub solution: Vec<String>,
    pub rating: u32,
    pub themes: Vec<String>,
}

/// The outcome of a single attempted puzzle move.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Feedback {
    /// Correct; the opponent has replied and it is the solver's turn again.
    Correct,
    /// The puzzle has been solved.
    Solved,
    /// The move does not match the solution.
    Wrong,
}

/// An in-progress puzzle attempt.
#[derive(Debug, Clone)]
pub struct PuzzleSession {
    pub puzzle: Puzzle,
    board: Board,
    index: usize,
    solved: bool,
}

impl PuzzleSession {
    /// Starts a puzzle, immediately applying the opponent's first move.
    pub fn new(puzzle: Puzzle) -> Result<PuzzleSession, String> {
        if puzzle.solution.is_empty() {
            return Err("puzzle has no solution".to_string());
        }
        let mut board = Board::from_fen(&puzzle.fen)?;
        let first = Move::from_uci(&puzzle.solution[0])
            .ok_or_else(|| "malformed solution move".to_string())?;
        if !board.legal_moves().contains(&first) {
            return Err("solution move is illegal in the puzzle position".to_string());
        }
        board.make_move(first);
        Ok(PuzzleSession {
            puzzle,
            board,
            index: 1,
            solved: false,
        })
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn is_solved(&self) -> bool {
        self.solved
    }

    /// Plays the solver's move, applying the opponent's reply on success.
    pub fn play_uci(&mut self, uci: &str) -> Result<Feedback, String> {
        if self.solved {
            return Err("puzzle already solved".to_string());
        }
        let played = Move::from_uci(uci).ok_or_else(|| format!("invalid move '{uci}'"))?;
        if !self.board.legal_moves().contains(&played) {
            return Err(format!("illegal move '{uci}'"));
        }
        let expected = self
            .puzzle
            .solution
            .get(self.index)
            .ok_or_else(|| "puzzle index out of range".to_string())?;
        let expected = Move::from_uci(expected).ok_or_else(|| "malformed solution".to_string())?;
        if played != expected {
            return Ok(Feedback::Wrong);
        }

        self.board.make_move(played);
        self.index += 1;
        if self.index >= self.puzzle.solution.len() {
            self.solved = true;
            return Ok(Feedback::Solved);
        }

        let reply = Move::from_uci(&self.puzzle.solution[self.index])
            .ok_or_else(|| "malformed solution".to_string())?;
        self.board.make_move(reply);
        self.index += 1;
        if self.index >= self.puzzle.solution.len() {
            self.solved = true;
            return Ok(Feedback::Solved);
        }
        Ok(Feedback::Correct)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn back_rank_puzzle() -> Puzzle {
        Puzzle {
            id: "test-1".to_string(),
            fen: "6k1/5ppp/8/8/8/8/5PPP/R5K1 b - - 0 1".to_string(),
            solution: vec!["g8h8".to_string(), "a1a8".to_string()],
            rating: 1200,
            themes: vec!["backRankMate".to_string(), "mateIn1".to_string()],
        }
    }

    #[test]
    fn solves_a_mate_in_one() {
        let mut session = PuzzleSession::new(back_rank_puzzle()).unwrap();
        assert!(!session.is_solved());
        assert_eq!(session.play_uci("a1a8").unwrap(), Feedback::Solved);
        assert!(session.is_solved());
    }

    #[test]
    fn rejects_the_wrong_move() {
        let mut session = PuzzleSession::new(back_rank_puzzle()).unwrap();
        assert_eq!(session.play_uci("a1a2").unwrap(), Feedback::Wrong);
        assert!(!session.is_solved());
    }

    #[test]
    fn rejects_illegal_moves() {
        let mut session = PuzzleSession::new(back_rank_puzzle()).unwrap();
        assert!(session.play_uci("a1a4").is_err());
    }
}
