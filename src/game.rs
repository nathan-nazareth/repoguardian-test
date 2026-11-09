//! Game state: move application, termination detection and results.

use crate::board::{Board, START_FEN};
use crate::moves::Move;
use crate::piece::Color;

/// Why a game was drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawReason {
    Stalemate,
    InsufficientMaterial,
    FiftyMoveRule,
    ThreefoldRepetition,
    Agreement,
}

/// How a game finished.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Checkmate(Color),
    Resignation(Color),
    Timeout(Color),
    Draw(DrawReason),
}

impl Outcome {
    /// The winning colour, if any.
    pub fn winner(self) -> Option<Color> {
        match self {
            Outcome::Checkmate(color) | Outcome::Resignation(color) | Outcome::Timeout(color) => {
                Some(color)
            }
            Outcome::Draw(_) => None,
        }
    }

    /// The PGN result string.
    pub fn score(self) -> &'static str {
        match self.winner() {
            Some(Color::White) => "1-0",
            Some(Color::Black) => "0-1",
            None => "1/2-1/2",
        }
    }
}

/// The lifecycle of a game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    Created,
    Started,
    Finished(Outcome),
}

/// A game of chess.
#[derive(Debug, Clone)]
pub struct Game {
    pub board: Board,
    pub moves: Vec<Move>,
    pub status: GameStatus,
    initial_fen: String,
    positions: Vec<String>,
}

impl Default for Game {
    fn default() -> Self {
        Game::new()
    }
}

impl Game {
    /// A new game from the standard starting position.
    pub fn new() -> Game {
        Game::from_fen(START_FEN).expect("the start position is valid")
    }

    /// A new game from an arbitrary position.
    pub fn from_fen(fen: &str) -> Result<Game, String> {
        let board = Board::from_fen(fen)?;
        let mut game = Game {
            board,
            moves: Vec::new(),
            status: GameStatus::Created,
            initial_fen: fen.to_string(),
            positions: Vec::new(),
        };
        game.positions.push(game.position_key());
        Ok(game)
    }

    pub fn initial_fen(&self) -> &str {
        &self.initial_fen
    }

    /// Marks the game as started.
    pub fn start(&mut self) {
        if let GameStatus::Created = self.status {
            self.status = GameStatus::Started;
        }
    }

    pub fn is_finished(&self) -> bool {
        matches!(self.status, GameStatus::Finished(_))
    }

    pub fn outcome(&self) -> Option<Outcome> {
        match self.status {
            GameStatus::Finished(outcome) => Some(outcome),
            _ => None,
        }
    }

    pub fn result(&self) -> &'static str {
        match self.outcome() {
            Some(outcome) => outcome.score(),
            None => "*",
        }
    }

    pub fn turn(&self) -> Color {
        self.board.side_to_move
    }

    pub fn legal_moves(&self) -> Vec<Move> {
        self.board.legal_moves()
    }

    pub fn current_fen(&self) -> String {
        self.board.to_fen()
    }

    /// Plays a legal move.
    pub fn play(&mut self, mv: Move) -> Result<(), String> {
        if self.is_finished() {
            return Err("the game is already over".to_string());
        }
        if !self.board.legal_moves().contains(&mv) {
            return Err(format!("illegal move {}", mv.to_uci()));
        }
        self.board.make_move(mv);
        self.moves.push(mv);
        if let GameStatus::Created = self.status {
            self.status = GameStatus::Started;
        }
        self.positions.push(self.position_key());
        self.refresh();
        Ok(())
    }

    /// Plays a move given in UCI notation.
    pub fn play_uci(&mut self, uci: &str) -> Result<Move, String> {
        let mv = Move::from_uci(uci).ok_or_else(|| format!("invalid move '{uci}'"))?;
        self.play(mv)?;
        Ok(mv)
    }

    /// A player resigns.
    pub fn resign(&mut self, color: Color) -> Result<(), String> {
        if self.is_finished() {
            return Err("the game is already over".to_string());
        }
        self.status = GameStatus::Finished(Outcome::Resignation(color));
        Ok(())
    }

    /// The game is declared drawn by agreement.
    pub fn declare_draw(&mut self) -> Result<(), String> {
        if self.is_finished() {
            return Err("the game is already over".to_string());
        }
        self.status = GameStatus::Finished(Outcome::Draw(DrawReason::Agreement));
        Ok(())
    }

    /// The SAN of every move played so far.
    pub fn san_history(&self) -> Vec<String> {
        let mut board = Board::from_fen(&self.initial_fen).expect("valid initial position");
        let mut history = Vec::new();
        for mv in &self.moves {
            history.push(crate::san::to_san(&board, *mv));
            board.make_move(*mv);
        }
        history
    }

    /// Re-evaluates the termination conditions; useful after loading a position.
    pub fn check_termination(&mut self) {
        self.refresh();
    }

    fn position_key(&self) -> String {
        self.board
            .to_fen()
            .split_whitespace()
            .take(4)
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn refresh(&mut self) {
        if self.is_finished() {
            return;
        }
        if self.board.legal_moves().is_empty() {
            if self.board.is_in_check(self.board.side_to_move) {
                let winner = self.board.side_to_move.opponent();
                self.status = GameStatus::Finished(Outcome::Checkmate(winner));
            } else {
                self.status = GameStatus::Finished(Outcome::Draw(DrawReason::Stalemate));
            }
            return;
        }
        if self.board.halfmove_clock >= 100 {
            self.status = GameStatus::Finished(Outcome::Draw(DrawReason::FiftyMoveRule));
            return;
        }
        if self.board.is_insufficient_material() {
            self.status = GameStatus::Finished(Outcome::Draw(DrawReason::InsufficientMaterial));
            return;
        }
        let key = self.position_key();
        if self
            .positions
            .iter()
            .filter(|entry| entry.as_str() == key.as_str())
            .count()
            >= 3
        {
            self.status = GameStatus::Finished(Outcome::Draw(DrawReason::ThreefoldRepetition));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fools_mate_ends_the_game() {
        let mut game = Game::new();
        for uci in ["f2f3", "e7e5", "g2g4"] {
            game.play_uci(uci).unwrap();
        }
        game.play_uci("d8h4").unwrap();
        assert_eq!(
            game.status,
            GameStatus::Finished(Outcome::Checkmate(Color::Black))
        );
        assert_eq!(game.result(), "0-1");
        assert_eq!(game.san_history().last().unwrap(), "Qh4#");
    }

    #[test]
    fn illegal_moves_are_rejected() {
        let mut game = Game::new();
        assert!(game.play_uci("e2e5").is_err());
        game.play_uci("e2e4").unwrap();
        assert!(game.play_uci("e2e4").is_err());
    }

    #[test]
    fn stalemate_is_a_draw() {
        let mut game = Game::from_fen("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1").unwrap();
        game.start();
        game.check_termination();
        assert_eq!(
            game.status,
            GameStatus::Finished(Outcome::Draw(DrawReason::Stalemate))
        );
    }

    #[test]
    fn bare_kings_are_drawn() {
        let mut game = Game::from_fen("8/8/8/8/8/8/8/K6k w - - 0 1").unwrap();
        game.start();
        game.check_termination();
        assert_eq!(
            game.status,
            GameStatus::Finished(Outcome::Draw(DrawReason::InsufficientMaterial))
        );
    }

    #[test]
    fn threefold_repetition_is_a_draw() {
        let mut game = Game::new();
        game.start();
        for uci in [
            "g1f3", "g8f6", "f3g1", "f6g8", "g1f3", "g8f6", "f3g1", "f6g8",
        ] {
            game.play_uci(uci).unwrap();
        }
        assert_eq!(
            game.status,
            GameStatus::Finished(Outcome::Draw(DrawReason::ThreefoldRepetition))
        );
    }
}
