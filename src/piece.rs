//! Chess pieces, colours and their fundamental operations.

use std::fmt;

/// The colour of a piece or a side to move.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    White,
    Black,
}

impl Color {
    /// The opposite colour.
    pub const fn opponent(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    /// Distance (in squares) that pawns of this colour advance.
    pub const fn pawn_dir(self) -> i8 {
        match self {
            Color::White => 8,
            Color::Black => -8,
        }
    }

    /// Rank on which pawns of this colour start.
    pub const fn start_rank(self) -> i8 {
        match self {
            Color::White => 1,
            Color::Black => 6,
        }
    }

    /// Rank on which pawns of this colour promote.
    pub const fn promotion_rank(self) -> i8 {
        match self {
            Color::White => 7,
            Color::Black => 0,
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Color::White => write!(f, "white"),
            Color::Black => write!(f, "black"),
        }
    }
}

/// The kind of a chess piece, independent of colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceKind {
    /// Rough material value in centipawns.
    pub const fn value(self) -> i32 {
        match self {
            PieceKind::Pawn => 100,
            PieceKind::Knight => 320,
            PieceKind::Bishop => 330,
            PieceKind::Rook => 500,
            PieceKind::Queen => 900,
            PieceKind::King => 0,
        }
    }

    /// The lowercase FEN letter for this kind.
    pub const fn fen_char(self) -> char {
        match self {
            PieceKind::Pawn => 'p',
            PieceKind::Knight => 'n',
            PieceKind::Bishop => 'b',
            PieceKind::Rook => 'r',
            PieceKind::Queen => 'q',
            PieceKind::King => 'k',
        }
    }

    /// Parses a FEN letter into a `(kind, colour)` pair.
    pub fn from_fen_char(c: char) -> Option<(PieceKind, Color)> {
        let color = if c.is_ascii_uppercase() {
            Color::White
        } else {
            Color::Black
        };
        let kind = match c.to_ascii_lowercase() {
            'p' => PieceKind::Pawn,
            'n' => PieceKind::Knight,
            'b' => PieceKind::Bishop,
            'r' => PieceKind::Rook,
            'q' => PieceKind::Queen,
            'k' => PieceKind::King,
            _ => return None,
        };
        Some((kind, color))
    }
}

/// A coloured piece.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Piece {
    pub color: Color,
    pub kind: PieceKind,
}

impl Piece {
    pub const fn new(color: Color, kind: PieceKind) -> Self {
        Piece { color, kind }
    }

    /// The FEN letter for this piece (uppercase for white).
    pub fn fen_char(self) -> char {
        let c = self.kind.fen_char();
        if self.color == Color::White {
            c.to_ascii_uppercase()
        } else {
            c
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fen_char_round_trips() {
        for c in ['p', 'n', 'b', 'r', 'q', 'k', 'P', 'N', 'B', 'R', 'Q', 'K'] {
            let (kind, color) = PieceKind::from_fen_char(c).unwrap();
            assert_eq!(Piece::new(color, kind).fen_char(), c);
        }
        assert!(PieceKind::from_fen_char('x').is_none());
    }

    #[test]
    fn colours_are_symmetric() {
        assert_eq!(Color::White.opponent(), Color::Black);
        assert_eq!(Color::Black.opponent(), Color::White);
    }
}
