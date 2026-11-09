//! Move representation.

use crate::piece::PieceKind;
use crate::square;

/// A chess move from one square to another, optionally promoting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Move {
    pub from: u8,
    pub to: u8,
    pub promotion: Option<PieceKind>,
}

impl Move {
    pub const fn new(from: u8, to: u8) -> Self {
        Move {
            from,
            to,
            promotion: None,
        }
    }

    pub const fn promote(from: u8, to: u8, kind: PieceKind) -> Self {
        Move {
            from,
            to,
            promotion: Some(kind),
        }
    }

    /// The move in long algebraic (UCI) notation, e.g. `e2e4` or `e7e8q`.
    pub fn to_uci(self) -> String {
        let mut out = format!("{}{}", square::name(self.from), square::name(self.to));
        if let Some(kind) = self.promotion {
            out.push(kind.fen_char());
        }
        out
    }

    /// Parses a UCI move such as `e7e8q`.
    pub fn from_uci(uci: &str) -> Option<Move> {
        if !uci.is_ascii() || uci.len() < 4 || uci.len() > 5 {
            return None;
        }
        let from = square::parse(&uci[0..2])?;
        let to = square::parse(&uci[2..4])?;
        let promotion = if uci.len() == 5 {
            let (kind, _) = PieceKind::from_fen_char(uci.as_bytes()[4] as char)?;
            Some(kind)
        } else {
            None
        };
        Some(Move {
            from,
            to,
            promotion,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uci_round_trips() {
        for uci in ["e2e4", "g1f3", "a7a8q", "h2h1n"] {
            assert_eq!(Move::from_uci(uci).unwrap().to_uci(), uci);
        }
    }

    #[test]
    fn invalid_uci_is_rejected() {
        assert!(Move::from_uci("e2").is_none());
        assert!(Move::from_uci("z2e4").is_none());
        assert!(Move::from_uci("e2e4x").is_none());
    }
}
