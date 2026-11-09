//! Standard Algebraic Notation (SAN) generation and parsing.

use crate::board::Board;
use crate::moves::Move;
use crate::piece::PieceKind;
use crate::square;

/// Renders the move `mv`, played from `board`, in SAN.
pub fn to_san(board: &Board, mv: Move) -> String {
    let piece = board
        .piece_at(mv.from)
        .expect("a move always starts on a piece");

    if piece.kind == PieceKind::King && (square::file(mv.to) - square::file(mv.from)).abs() == 2 {
        let base = if square::file(mv.to) > square::file(mv.from) {
            "O-O"
        } else {
            "O-O-O"
        };
        return format!("{base}{}", check_suffix(board, mv));
    }

    let is_capture = board.piece_at(mv.to).is_some()
        || (piece.kind == PieceKind::Pawn && square::file(mv.from) != square::file(mv.to));

    let mut san = String::new();
    if piece.kind == PieceKind::Pawn {
        if is_capture {
            san.push((b'a' + square::file(mv.from) as u8) as char);
            san.push('x');
        }
        san.push_str(&square::name(mv.to));
        if let Some(kind) = mv.promotion {
            san.push('=');
            san.push(kind.fen_char().to_ascii_uppercase());
        }
    } else {
        san.push(piece.kind.fen_char().to_ascii_uppercase());
        san.push_str(&disambiguate(board, mv, piece.kind));
        if is_capture {
            san.push('x');
        }
        san.push_str(&square::name(mv.to));
    }
    san.push_str(&check_suffix(board, mv));
    san
}

/// Parses a SAN token by comparing it against the SAN of every legal move.
pub fn parse_san(board: &Board, san: &str) -> Option<Move> {
    let target = normalize(san);
    board
        .legal_moves()
        .into_iter()
        .find(|mv| normalize(&to_san(board, *mv)) == target)
}

fn disambiguate(board: &Board, mv: Move, kind: PieceKind) -> String {
    let mut ambiguous = false;
    let mut same_file = false;
    let mut same_rank = false;
    for other in board.legal_moves() {
        if other.from == mv.from || other.to != mv.to {
            continue;
        }
        if let Some(piece) = board.piece_at(other.from) {
            if piece.kind == kind {
                ambiguous = true;
                if square::file(other.from) == square::file(mv.from) {
                    same_file = true;
                }
                if square::rank(other.from) == square::rank(mv.from) {
                    same_rank = true;
                }
            }
        }
    }
    if !ambiguous {
        return String::new();
    }
    let file_char = (b'a' + square::file(mv.from) as u8) as char;
    let rank_char = (b'1' + square::rank(mv.from) as u8) as char;
    if !same_file {
        file_char.to_string()
    } else if !same_rank {
        rank_char.to_string()
    } else {
        format!("{file_char}{rank_char}")
    }
}

fn check_suffix(board: &Board, mv: Move) -> String {
    let mut board = board.clone();
    board.make_move(mv);
    if board.is_in_check(board.side_to_move) {
        if board.legal_moves().is_empty() {
            "#".to_string()
        } else {
            "+".to_string()
        }
    } else {
        String::new()
    }
}

fn normalize(san: &str) -> String {
    let mut s = san.trim().to_string();
    s = s.replace("0-0-0", "O-O-O").replace("0-0", "O-O");
    while matches!(
        s.chars().last(),
        Some('+') | Some('#') | Some('!') | Some('?')
    ) {
        s.pop();
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn san_of(board: &Board, uci: &str) -> String {
        to_san(board, Move::from_uci(uci).unwrap())
    }

    #[test]
    fn renders_common_moves() {
        let board = Board::start();
        assert_eq!(san_of(&board, "e2e4"), "e4");
        assert_eq!(san_of(&board, "g1f3"), "Nf3");
        assert_eq!(san_of(&board, "b1c3"), "Nc3");
    }

    #[test]
    fn renders_castling() {
        let board = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap();
        assert_eq!(san_of(&board, "e1g1"), "O-O");
        assert_eq!(san_of(&board, "e1c1"), "O-O-O");
    }

    #[test]
    fn renders_checkmate_suffix() {
        let board =
            Board::from_fen("rnbqkbnr/pppp1ppp/8/4p3/6P1/5P2/PPPPP2P/RNBQKBNR b KQkq g3 0 2")
                .unwrap();
        assert_eq!(san_of(&board, "d8h4"), "Qh4#");
    }

    #[test]
    fn parses_san_round_trip() {
        let board = Board::start();
        for uci in ["e2e4", "d2d4", "g1f3", "b1c3"] {
            let mv = Move::from_uci(uci).unwrap();
            assert_eq!(parse_san(&board, &to_san(&board, mv)), Some(mv));
        }
    }
}
