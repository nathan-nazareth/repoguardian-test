//! PGN import and export.

use crate::board::{Board, START_FEN};
use crate::game::Game;
use crate::moves::Move;
use crate::san;

/// Renders a game as a PGN document.
pub fn to_pgn(game: &Game, white: &str, black: &str) -> String {
    let mut out = String::new();
    out.push_str("[Event \"lichess-rs\"]\n");
    out.push_str("[Site \"lichess.org\"]\n");
    out.push_str("[Date \"????.??.??\"]\n");
    out.push_str(&format!("[White \"{white}\"]\n"));
    out.push_str(&format!("[Black \"{black}\"]\n"));
    out.push_str(&format!("[Result \"{}\"]\n", game.result()));
    if game.initial_fen() != START_FEN {
        out.push_str(&format!("[FEN \"{}\"]\n", game.initial_fen()));
        out.push_str("[SetUp \"1\"]\n");
    }
    out.push('\n');

    let history = game.san_history();
    let mut line = String::new();
    for (index, san) in history.iter().enumerate() {
        if index % 2 == 0 {
            line.push_str(&format!("{}. ", index / 2 + 1));
        }
        line.push_str(san);
        line.push(' ');
        if line.len() >= 78 {
            out.push_str(line.trim_end());
            out.push('\n');
            line.clear();
        }
    }
    line.push_str(game.result());
    out.push_str(line.trim_end());
    out.push('\n');
    out
}

/// Parses the main line of a PGN document starting from `start_fen`.
pub fn parse_moves(pgn: &str, start_fen: &str) -> Result<Vec<Move>, String> {
    let mut board = Board::from_fen(start_fen)?;
    let mut moves = Vec::new();
    for token in tokenize(pgn) {
        let mv = san::parse_san(&board, &token)
            .ok_or_else(|| format!("could not parse move '{token}'"))?;
        board.make_move(mv);
        moves.push(mv);
    }
    Ok(moves)
}

fn tokenize(pgn: &str) -> Vec<String> {
    let mut text = String::new();
    for line in pgn.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('[') {
            continue;
        }
        text.push_str(trimmed);
        text.push(' ');
    }

    let mut cleaned = String::new();
    let mut brace_depth = 0u32;
    let mut paren_depth = 0u32;
    for ch in text.chars() {
        match ch {
            '{' => brace_depth += 1,
            '}' => brace_depth = brace_depth.saturating_sub(1),
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            _ if brace_depth == 0 && paren_depth == 0 => cleaned.push(ch),
            _ => {}
        }
    }

    cleaned
        .split_whitespace()
        .filter(|token| !is_move_number(token) && !is_result(token) && !token.starts_with('$'))
        .map(|token| token.to_string())
        .collect()
}

fn is_move_number(token: &str) -> bool {
    let digits: String = token.chars().take_while(|c| c.is_ascii_digit()).collect();
    !digits.is_empty() && token[digits.len()..].chars().all(|c| c == '.')
}

fn is_result(token: &str) -> bool {
    matches!(token, "1-0" | "0-1" | "1/2-1/2" | "*")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Game;

    #[test]
    fn exports_a_pgn_with_the_result() {
        let mut game = Game::new();
        for uci in ["f2f3", "e7e5", "g2g4", "d8h4"] {
            game.play_uci(uci).unwrap();
        }
        let pgn = to_pgn(&game, "Alice", "Bob");
        assert!(pgn.contains("[White \"Alice\"]"));
        assert!(pgn.contains("Qh4#"));
        assert!(pgn.contains("0-1"));
    }

    #[test]
    fn parses_moves_from_a_pgn() {
        let moves = parse_moves("1. f3 e5 2. g4 Qh4# 0-1", START_FEN).unwrap();
        assert_eq!(moves.len(), 4);
        assert_eq!(moves[3].to_uci(), "d8h4");
    }
}
