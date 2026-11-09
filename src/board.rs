//! The chessboard: piece placement, FEN handling, move generation and
//! make/unmake, plus a small perft driver used by the tests.

use crate::moves::Move;
use crate::piece::{Color, Piece, PieceKind};
use crate::square;

/// The standard starting position in FEN.
pub const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

const KNIGHT_DELTAS: [(i8, i8); 8] = [
    (1, 2),
    (2, 1),
    (2, -1),
    (1, -2),
    (-1, -2),
    (-2, -1),
    (-2, 1),
    (-1, 2),
];
const KING_DELTAS: [(i8, i8); 8] = [
    (1, 0),
    (-1, 0),
    (0, 1),
    (0, -1),
    (1, 1),
    (1, -1),
    (-1, 1),
    (-1, -1),
];
const BISHOP_DIRS: [(i8, i8); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
const ROOK_DIRS: [(i8, i8); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

/// Castling availability for both sides.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CastlingRights {
    pub white_kingside: bool,
    pub white_queenside: bool,
    pub black_kingside: bool,
    pub black_queenside: bool,
}

impl CastlingRights {
    pub const NONE: CastlingRights = CastlingRights {
        white_kingside: false,
        white_queenside: false,
        black_kingside: false,
        black_queenside: false,
    };

    pub const ALL: CastlingRights = CastlingRights {
        white_kingside: true,
        white_queenside: true,
        black_kingside: true,
        black_queenside: true,
    };

    /// Parses the castling field of a FEN string.
    pub fn from_fen(field: &str) -> CastlingRights {
        let mut rights = CastlingRights::NONE;
        for ch in field.chars() {
            match ch {
                'K' => rights.white_kingside = true,
                'Q' => rights.white_queenside = true,
                'k' => rights.black_kingside = true,
                'q' => rights.black_queenside = true,
                _ => {}
            }
        }
        rights
    }

    /// Serialises the rights back into a FEN field.
    pub fn to_fen(self) -> String {
        let mut out = String::new();
        if self.white_kingside {
            out.push('K');
        }
        if self.white_queenside {
            out.push('Q');
        }
        if self.black_kingside {
            out.push('k');
        }
        if self.black_queenside {
            out.push('q');
        }
        if out.is_empty() {
            out.push('-');
        }
        out
    }

    /// Removes any right invalidated by a piece moving from or to `sq`.
    fn remove_square(&mut self, sq: u8) {
        match sq {
            4 => {
                self.white_kingside = false;
                self.white_queenside = false;
            }
            7 => self.white_kingside = false,
            0 => self.white_queenside = false,
            60 => {
                self.black_kingside = false;
                self.black_queenside = false;
            }
            63 => self.black_kingside = false,
            56 => self.black_queenside = false,
            _ => {}
        }
    }
}

/// Everything required to undo a move made on a [`Board`].
#[derive(Debug, Clone, Copy)]
pub struct Undo {
    captured: Option<Piece>,
    captured_square: Option<u8>,
    castling: CastlingRights,
    ep_square: Option<u8>,
    halfmove_clock: u32,
    fullmove_number: u32,
}

/// A complete chess position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    pub squares: [Option<Piece>; 64],
    pub side_to_move: Color,
    pub castling: CastlingRights,
    pub ep_square: Option<u8>,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
}

impl Default for Board {
    fn default() -> Self {
        Board::start()
    }
}

impl Board {
    /// The initial position.
    pub fn start() -> Board {
        Board::from_fen(START_FEN).expect("the starting position is always valid")
    }

    /// A board with no pieces at all; mostly useful for tests.
    pub fn empty() -> Board {
        Board {
            squares: [None; 64],
            side_to_move: Color::White,
            castling: CastlingRights::NONE,
            ep_square: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    /// Parses a FEN string.
    pub fn from_fen(fen: &str) -> Result<Board, String> {
        let fields: Vec<&str> = fen.split_whitespace().collect();
        if fields.len() < 4 {
            return Err(format!("invalid FEN: {fen}"));
        }

        let mut squares = [None; 64];
        let ranks: Vec<&str> = fields[0].split('/').collect();
        if ranks.len() != 8 {
            return Err(format!("invalid FEN rank count: {fen}"));
        }
        for (index, rank_field) in ranks.iter().enumerate() {
            let rank = 7 - index as i8;
            let mut file = 0i8;
            for ch in rank_field.chars() {
                if let Some(digit) = ch.to_digit(10) {
                    file += digit as i8;
                } else if let Some((kind, color)) = PieceKind::from_fen_char(ch) {
                    match square::make(file, rank) {
                        Some(sq) => squares[sq as usize] = Some(Piece::new(color, kind)),
                        None => return Err(format!("invalid FEN: {fen}")),
                    }
                    file += 1;
                } else {
                    return Err(format!("invalid FEN piece '{ch}' in {fen}"));
                }
            }
            if file != 8 {
                return Err(format!("invalid FEN rank width in {fen}"));
            }
        }

        let side_to_move = match fields[1] {
            "w" => Color::White,
            "b" => Color::Black,
            other => return Err(format!("invalid side to move '{other}'")),
        };

        let castling = CastlingRights::from_fen(fields[2]);

        let ep_square = if fields[3] == "-" {
            None
        } else {
            square::parse(fields[3])
                .ok_or_else(|| format!("invalid en passant square '{}'", fields[3]))
                .map(Some)?
        };

        let halfmove_clock = fields
            .get(4)
            .and_then(|field| field.parse().ok())
            .unwrap_or(0);
        let fullmove_number = fields
            .get(5)
            .and_then(|field| field.parse().ok())
            .unwrap_or(1);

        Ok(Board {
            squares,
            side_to_move,
            castling,
            ep_square,
            halfmove_clock,
            fullmove_number,
        })
    }

    /// Serialises the position to FEN.
    pub fn to_fen(&self) -> String {
        let mut out = String::new();
        for rank in (0..8i8).rev() {
            let mut empty = 0;
            for file in 0..8i8 {
                let sq = (rank * 8 + file) as usize;
                match self.squares[sq] {
                    Some(piece) => {
                        if empty > 0 {
                            out.push_str(&empty.to_string());
                            empty = 0;
                        }
                        out.push(piece.fen_char());
                    }
                    None => empty += 1,
                }
            }
            if empty > 0 {
                out.push_str(&empty.to_string());
            }
            if rank > 0 {
                out.push('/');
            }
        }
        out.push(' ');
        out.push(match self.side_to_move {
            Color::White => 'w',
            Color::Black => 'b',
        });
        out.push(' ');
        out.push_str(&self.castling.to_fen());
        out.push(' ');
        match self.ep_square {
            Some(sq) => out.push_str(&square::name(sq)),
            None => out.push('-'),
        }
        out.push(' ');
        out.push_str(&self.halfmove_clock.to_string());
        out.push(' ');
        out.push_str(&self.fullmove_number.to_string());
        out
    }

    /// The piece on `sq`, if any.
    pub fn piece_at(&self, sq: u8) -> Option<Piece> {
        self.squares.get(sq as usize).copied().flatten()
    }

    /// The square of `color`'s king, if it is on the board.
    pub fn king_square(&self, color: Color) -> Option<u8> {
        self.squares
            .iter()
            .position(|slot| {
                matches!(slot, Some(piece) if piece.kind == PieceKind::King && piece.color == color)
            })
            .map(|index| index as u8)
    }

    /// Whether `color`'s king is currently attacked.
    pub fn is_in_check(&self, color: Color) -> bool {
        match self.king_square(color) {
            Some(king) => self.is_square_attacked(king, color.opponent()),
            None => false,
        }
    }

    /// Whether `target` is attacked by any piece of colour `by`.
    pub fn is_square_attacked(&self, target: u8, by: Color) -> bool {
        let target_file = square::file(target);
        let target_rank = square::rank(target);

        // Pawns.
        let pawn_rank = target_rank - (if by == Color::White { 1 } else { -1 });
        for file_delta in [-1i8, 1] {
            if let Some(sq) = square::make(target_file + file_delta, pawn_rank) {
                if let Some(piece) = self.piece_at(sq) {
                    if piece.color == by && piece.kind == PieceKind::Pawn {
                        return true;
                    }
                }
            }
        }

        // Knights.
        for &(df, dr) in KNIGHT_DELTAS.iter() {
            if let Some(sq) = square::make(target_file + df, target_rank + dr) {
                if let Some(piece) = self.piece_at(sq) {
                    if piece.color == by && piece.kind == PieceKind::Knight {
                        return true;
                    }
                }
            }
        }

        // King.
        for &(df, dr) in KING_DELTAS.iter() {
            if let Some(sq) = square::make(target_file + df, target_rank + dr) {
                if let Some(piece) = self.piece_at(sq) {
                    if piece.color == by && piece.kind == PieceKind::King {
                        return true;
                    }
                }
            }
        }

        // Bishops and queens.
        for &(df, dr) in BISHOP_DIRS.iter() {
            let mut file = target_file + df;
            let mut rank = target_rank + dr;
            while let Some(sq) = square::make(file, rank) {
                if let Some(piece) = self.piece_at(sq) {
                    if piece.color == by
                        && matches!(piece.kind, PieceKind::Bishop | PieceKind::Queen)
                    {
                        return true;
                    }
                    break;
                }
                file += df;
                rank += dr;
            }
        }

        // Rooks and queens.
        for &(df, dr) in ROOK_DIRS.iter() {
            let mut file = target_file + df;
            let mut rank = target_rank + dr;
            while let Some(sq) = square::make(file, rank) {
                if let Some(piece) = self.piece_at(sq) {
                    if piece.color == by && matches!(piece.kind, PieceKind::Rook | PieceKind::Queen)
                    {
                        return true;
                    }
                    break;
                }
                file += df;
                rank += dr;
            }
        }

        false
    }

    /// All moves that ignore king safety.
    pub fn pseudo_legal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        for sq in 0..64u8 {
            let piece = match self.piece_at(sq) {
                Some(piece) => piece,
                None => continue,
            };
            if piece.color != self.side_to_move {
                continue;
            }
            match piece.kind {
                PieceKind::Pawn => self.pawn_moves(sq, &mut moves),
                PieceKind::Knight => self.step_moves(sq, &KNIGHT_DELTAS, &mut moves),
                PieceKind::Bishop => self.slide_moves(sq, &BISHOP_DIRS, &mut moves),
                PieceKind::Rook => self.slide_moves(sq, &ROOK_DIRS, &mut moves),
                PieceKind::Queen => {
                    self.slide_moves(sq, &BISHOP_DIRS, &mut moves);
                    self.slide_moves(sq, &ROOK_DIRS, &mut moves);
                }
                PieceKind::King => {
                    self.step_moves(sq, &KING_DELTAS, &mut moves);
                    self.castling_moves(sq, &mut moves);
                }
            }
        }
        moves
    }

    fn step_moves(&self, from: u8, deltas: &[(i8, i8)], moves: &mut Vec<Move>) {
        let (file, rank) = (square::file(from), square::rank(from));
        for &(df, dr) in deltas {
            if let Some(to) = square::make(file + df, rank + dr) {
                match self.squares[to as usize] {
                    Some(piece) if piece.color == self.side_to_move => {}
                    _ => moves.push(Move::new(from, to)),
                }
            }
        }
    }

    fn slide_moves(&self, from: u8, dirs: &[(i8, i8)], moves: &mut Vec<Move>) {
        let (file, rank) = (square::file(from), square::rank(from));
        for &(df, dr) in dirs {
            let mut file = file + df;
            let mut rank = rank + dr;
            while let Some(to) = square::make(file, rank) {
                match self.squares[to as usize] {
                    Some(piece) => {
                        if piece.color != self.side_to_move {
                            moves.push(Move::new(from, to));
                        }
                        break;
                    }
                    None => moves.push(Move::new(from, to)),
                }
                file += df;
                rank += dr;
            }
        }
    }

    fn pawn_moves(&self, from: u8, moves: &mut Vec<Move>) {
        let side = self.side_to_move;
        let (file, rank) = (square::file(from), square::rank(from));
        let direction = if side == Color::White { 1 } else { -1 };
        let start_rank = side.start_rank();
        let promotion_rank = side.promotion_rank();

        if let Some(one) = square::make(file, rank + direction) {
            if self.squares[one as usize].is_none() {
                self.push_pawn_move(from, one, promotion_rank, moves);
                if rank == start_rank {
                    if let Some(two) = square::make(file, rank + 2 * direction) {
                        if self.squares[two as usize].is_none() {
                            moves.push(Move::new(from, two));
                        }
                    }
                }
            }
        }

        for file_delta in [-1i8, 1] {
            if let Some(to) = square::make(file + file_delta, rank + direction) {
                match self.squares[to as usize] {
                    Some(piece) if piece.color != side => {
                        self.push_pawn_move(from, to, promotion_rank, moves);
                    }
                    Some(_) => {}
                    None if self.ep_square == Some(to) => moves.push(Move::new(from, to)),
                    None => {}
                }
            }
        }
    }

    fn push_pawn_move(&self, from: u8, to: u8, promotion_rank: i8, moves: &mut Vec<Move>) {
        if square::rank(to) == promotion_rank {
            for kind in [
                PieceKind::Queen,
                PieceKind::Rook,
                PieceKind::Bishop,
                PieceKind::Knight,
            ] {
                moves.push(Move::promote(from, to, kind));
            }
        } else {
            moves.push(Move::new(from, to));
        }
    }

    fn castling_moves(&self, from: u8, moves: &mut Vec<Move>) {
        let side = self.side_to_move;
        let home = if side == Color::White { 4 } else { 60 };
        if from != home {
            return;
        }
        let opponent = side.opponent();
        let (kingside, queenside) = if side == Color::White {
            (self.castling.white_kingside, self.castling.white_queenside)
        } else {
            (self.castling.black_kingside, self.castling.black_queenside)
        };
        let rank = square::rank(home);

        if kingside {
            let f = square::make(5, rank).unwrap();
            let g = square::make(6, rank).unwrap();
            let rook_home = square::make(7, rank).unwrap();
            let rook_is_home = matches!(
                self.piece_at(rook_home),
                Some(piece) if piece.kind == PieceKind::Rook && piece.color == side
            );
            if rook_is_home
                && self.squares[f as usize].is_none()
                && self.squares[g as usize].is_none()
                && !self.is_square_attacked(home, opponent)
                && !self.is_square_attacked(f, opponent)
                && !self.is_square_attacked(g, opponent)
            {
                moves.push(Move::new(home, g));
            }
        }

        if queenside {
            let d = square::make(3, rank).unwrap();
            let c = square::make(2, rank).unwrap();
            let b = square::make(1, rank).unwrap();
            let rook_home = square::make(0, rank).unwrap();
            let rook_is_home = matches!(
                self.piece_at(rook_home),
                Some(piece) if piece.kind == PieceKind::Rook && piece.color == side
            );
            if rook_is_home
                && self.squares[d as usize].is_none()
                && self.squares[c as usize].is_none()
                && self.squares[b as usize].is_none()
                && !self.is_square_attacked(home, opponent)
                && !self.is_square_attacked(d, opponent)
                && !self.is_square_attacked(c, opponent)
            {
                moves.push(Move::new(home, c));
            }
        }
    }

    /// All fully legal moves for the side to move.
    pub fn legal_moves(&self) -> Vec<Move> {
        let side = self.side_to_move;
        let mut board = self.clone();
        let mut legal = Vec::new();
        for mv in self.pseudo_legal_moves() {
            let undo = board.make_move(mv);
            if !board.is_in_check(side) {
                legal.push(mv);
            }
            board.undo_move(mv, undo);
        }
        legal
    }

    /// Legal moves that start on `from`.
    pub fn legal_moves_from(&self, from: u8) -> Vec<Move> {
        self.legal_moves()
            .into_iter()
            .filter(|mv| mv.from == from)
            .collect()
    }

    /// Plays a move, returning the information required to undo it.
    pub fn make_move(&mut self, mv: Move) -> Undo {
        let moving = self.squares[mv.from as usize].expect("no piece on departure square");
        let mut undo = Undo {
            captured: None,
            captured_square: None,
            castling: self.castling,
            ep_square: self.ep_square,
            halfmove_clock: self.halfmove_clock,
            fullmove_number: self.fullmove_number,
        };

        let is_en_passant = moving.kind == PieceKind::Pawn
            && self.ep_square == Some(mv.to)
            && square::file(mv.from) != square::file(mv.to);
        if is_en_passant {
            let captured_square = (mv.to as i8 - moving.color.pawn_dir()) as u8;
            undo.captured = self.squares[captured_square as usize];
            undo.captured_square = Some(captured_square);
            self.squares[captured_square as usize] = None;
        } else if let Some(captured) = self.squares[mv.to as usize] {
            undo.captured = Some(captured);
            undo.captured_square = Some(mv.to);
        }

        self.squares[mv.from as usize] = None;
        let mut placed = moving;
        if let Some(kind) = mv.promotion {
            placed.kind = kind;
        }
        self.squares[mv.to as usize] = Some(placed);

        if moving.kind == PieceKind::King
            && (square::file(mv.to) - square::file(mv.from)).abs() == 2
        {
            let rank = square::rank(mv.from);
            let (rook_from_file, rook_to_file) = if square::file(mv.to) > square::file(mv.from) {
                (7, 5)
            } else {
                (0, 3)
            };
            let rook_from = square::make(rook_from_file, rank).unwrap();
            let rook_to = square::make(rook_to_file, rank).unwrap();
            self.squares[rook_to as usize] = self.squares[rook_from as usize];
            self.squares[rook_from as usize] = None;
        }

        self.castling.remove_square(mv.from);
        self.castling.remove_square(mv.to);

        self.ep_square =
            if moving.kind == PieceKind::Pawn && (mv.from as i8 - mv.to as i8).abs() == 16 {
                Some(((mv.from as i8 + mv.to as i8) / 2) as u8)
            } else {
                None
            };

        if moving.kind == PieceKind::Pawn || undo.captured.is_some() {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }

        if self.side_to_move == Color::Black {
            self.fullmove_number += 1;
        }
        self.side_to_move = self.side_to_move.opponent();

        undo
    }

    /// Reverts a move previously played with [`Board::make_move`].
    pub fn undo_move(&mut self, mv: Move, undo: Undo) {
        self.side_to_move = self.side_to_move.opponent();
        self.castling = undo.castling;
        self.ep_square = undo.ep_square;
        self.halfmove_clock = undo.halfmove_clock;
        self.fullmove_number = undo.fullmove_number;

        let moved = self.squares[mv.to as usize].expect("no piece on arrival square");

        if moved.kind == PieceKind::King && (square::file(mv.to) - square::file(mv.from)).abs() == 2
        {
            let rank = square::rank(mv.from);
            let (rook_from_file, rook_to_file) = if square::file(mv.to) > square::file(mv.from) {
                (7, 5)
            } else {
                (0, 3)
            };
            let rook_from = square::make(rook_from_file, rank).unwrap();
            let rook_to = square::make(rook_to_file, rank).unwrap();
            self.squares[rook_from as usize] = self.squares[rook_to as usize];
            self.squares[rook_to as usize] = None;
        }

        let original_kind = if mv.promotion.is_some() {
            PieceKind::Pawn
        } else {
            moved.kind
        };
        self.squares[mv.from as usize] = Some(Piece::new(moved.color, original_kind));
        self.squares[mv.to as usize] = None;
        if let (Some(captured), Some(captured_square)) = (undo.captured, undo.captured_square) {
            self.squares[captured_square as usize] = Some(captured);
        }
    }

    /// True when checkmate is impossible for either side with the material left.
    pub fn is_insufficient_material(&self) -> bool {
        let mut knights = 0u32;
        let mut light_bishops = 0u32;
        let mut dark_bishops = 0u32;
        for (index, slot) in self.squares.iter().enumerate() {
            let piece = match slot {
                Some(piece) => piece,
                None => continue,
            };
            match piece.kind {
                PieceKind::King => {}
                PieceKind::Pawn | PieceKind::Rook | PieceKind::Queen => return false,
                PieceKind::Knight => knights += 1,
                PieceKind::Bishop => {
                    let square = index as u8;
                    if (square::file(square) + square::rank(square)) % 2 == 0 {
                        light_bishops += 1;
                    } else {
                        dark_bishops += 1;
                    }
                }
            }
        }
        let minors = knights + light_bishops + dark_bishops;
        if minors <= 1 {
            return true;
        }
        knights == 0 && (light_bishops == 0 || dark_bishops == 0)
    }

    /// Counts leaf nodes of the move tree up to `depth` plies.
    pub fn perft(&self, depth: u32) -> u64 {
        let mut board = self.clone();
        board.perft_inner(depth)
    }

    fn perft_inner(&mut self, depth: u32) -> u64 {
        if depth == 0 {
            return 1;
        }
        let mut nodes = 0;
        for mv in self.legal_moves() {
            let undo = self.make_move(mv);
            nodes += self.perft_inner(depth - 1);
            self.undo_move(mv, undo);
        }
        nodes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_position_round_trips_through_fen() {
        let board = Board::start();
        assert_eq!(board.to_fen(), START_FEN);
        assert_eq!(Board::from_fen(START_FEN).unwrap(), board);
    }

    #[test]
    fn initial_position_has_twenty_moves() {
        assert_eq!(Board::start().legal_moves().len(), 20);
    }

    #[test]
    fn perft_start_position() {
        let board = Board::start();
        assert_eq!(board.perft(1), 20);
        assert_eq!(board.perft(2), 400);
        assert_eq!(board.perft(3), 8_902);
        assert_eq!(board.perft(4), 197_281);
    }

    #[test]
    fn perft_kiwipete() {
        let board =
            Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
                .unwrap();
        assert_eq!(board.perft(1), 48);
        assert_eq!(board.perft(2), 2_039);
        assert_eq!(board.perft(3), 97_862);
    }

    #[test]
    fn en_passant_is_generated_and_captures() {
        let fen = "rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3";
        let mut board = Board::from_fen(fen).unwrap();
        let mv = Move::from_uci("e5f6").unwrap();
        assert!(board.legal_moves().contains(&mv));
        let undo = board.make_move(mv);
        assert!(board.piece_at(square::parse("f6").unwrap()).is_some());
        assert!(board.piece_at(square::parse("f5").unwrap()).is_none());
        board.undo_move(mv, undo);
        assert_eq!(board.to_fen(), fen);
    }

    #[test]
    fn castling_moves_the_rook() {
        let mut board = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap();
        let mv = Move::from_uci("e1g1").unwrap();
        assert!(board.legal_moves().contains(&mv));
        board.make_move(mv);
        assert!(board.piece_at(square::parse("g1").unwrap()).is_some());
        assert!(board.piece_at(square::parse("f1").unwrap()).is_some());
        assert!(board.piece_at(square::parse("h1").unwrap()).is_none());
    }

    #[test]
    fn promotion_generates_four_choices() {
        let board = Board::from_fen("8/P7/8/8/8/8/8/4K2k w - - 0 1").unwrap();
        let promotions: Vec<Move> = board
            .legal_moves()
            .into_iter()
            .filter(|mv| mv.promotion.is_some())
            .collect();
        assert_eq!(promotions.len(), 4);
    }

    #[test]
    fn checkmate_has_no_legal_moves() {
        let board =
            Board::from_fen("rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3")
                .unwrap();
        assert!(board.is_in_check(board.side_to_move));
        assert!(board.legal_moves().is_empty());
    }

    #[test]
    fn bare_kings_are_insufficient_material() {
        let board = Board::from_fen("8/8/8/8/8/8/8/K6k w - - 0 1").unwrap();
        assert!(board.is_insufficient_material());
    }

    #[test]
    fn make_and_undo_restore_the_position() {
        let mut board = Board::start();
        for uci in ["e2e4", "e7e5", "g1f3", "b8c6"] {
            let before = board.to_fen();
            let mv = Move::from_uci(uci).unwrap();
            let undo = board.make_move(mv);
            board.undo_move(mv, undo);
            assert_eq!(board.to_fen(), before);
        }
    }
}
