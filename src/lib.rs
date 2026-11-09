//! # lichess-rs
//!
//! A modular, idiomatic Rust re-implementation of the core of
//! [lichess.org](https://lichess.org): the chess rules engine, games, clocks,
//! ratings, users, puzzles, tournaments and a small HTTP API.
//!
//! The crate is intentionally dependency-free so that it builds and tests with a
//! plain `cargo test`. Every subsystem lives in its own module and the pure game
//! logic (board, move generation, rules, SAN, PGN) is covered by unit tests.

pub mod api;
pub mod board;
pub mod challenge;
pub mod clock;
pub mod game;
pub mod moves;
pub mod opening;
pub mod pgn;
pub mod piece;
pub mod puzzle;
pub mod rating;
pub mod san;
pub mod square;
pub mod tournament;
pub mod user;

pub use board::{Board, START_FEN};
pub use clock::{Clock, Speed, TimeControl};
pub use game::{DrawReason, Game, GameStatus, Outcome};
pub use moves::Move;
pub use piece::{Color, Piece, PieceKind};
pub use rating::Rating;
pub use user::User;
