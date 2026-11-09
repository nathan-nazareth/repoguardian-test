//! A very small opening book.

/// `(position key, best move in UCI, opening name)`.
const BOOK: &[(&str, &str, &str)] = &[
    (
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq",
        "e2e4",
        "King's Pawn Opening",
    ),
    (
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq",
        "c7c5",
        "Sicilian Defence",
    ),
    (
        "rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b KQkq",
        "g8f6",
        "Indian Defence",
    ),
    (
        "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq",
        "g1f3",
        "Sicilian Defence",
    ),
    (
        "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq",
        "g1f3",
        "Open Game",
    ),
];

/// The first three FEN fields identify a position for book purposes.
fn position_key(fen: &str) -> String {
    fen.split_whitespace().take(3).collect::<Vec<_>>().join(" ")
}

/// The book move for a position, if we know one.
pub fn book_move(fen: &str) -> Option<&'static str> {
    let key = position_key(fen);
    BOOK.iter()
        .find(|(book_key, _, _)| *book_key == key.as_str())
        .map(|(_, mv, _)| *mv)
}

/// The opening name for a position, if we know one.
pub fn opening_name(fen: &str) -> Option<&'static str> {
    let key = position_key(fen);
    BOOK.iter()
        .find(|(book_key, _, _)| *book_key == key.as_str())
        .map(|(_, _, name)| *name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{Board, START_FEN};
    use crate::moves::Move;

    #[test]
    fn finds_the_first_book_move() {
        assert_eq!(book_move(START_FEN), Some("e2e4"));
        assert_eq!(opening_name(START_FEN), Some("King's Pawn Opening"));
    }

    #[test]
    fn follows_the_sicilian() {
        let mut board = Board::start();
        let mv = Move::from_uci(book_move(&board.to_fen()).unwrap()).unwrap();
        board.make_move(mv);
        assert_eq!(book_move(&board.to_fen()), Some("c7c5"));
    }

    #[test]
    fn unknown_positions_have_no_book_move() {
        assert!(book_move("8/8/8/8/8/8/8/K6k w - - 0 1").is_none());
    }
}
