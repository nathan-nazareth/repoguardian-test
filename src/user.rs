//! User accounts, authentication and rating bookkeeping.

use crate::rating::{GameResult, Rating};

/// A player's optional title.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Title {
    GM,
    IM,
    FM,
    CM,
    NM,
    WGM,
    WIM,
    WFM,
    WCM,
    LM,
    BOT,
}

impl Title {
    pub fn as_str(self) -> &'static str {
        match self {
            Title::GM => "GM",
            Title::IM => "IM",
            Title::FM => "FM",
            Title::CM => "CM",
            Title::NM => "NM",
            Title::WGM => "WGM",
            Title::WIM => "WIM",
            Title::WFM => "WFM",
            Title::WCM => "WCM",
            Title::LM => "LM",
            Title::BOT => "BOT",
        }
    }
}

/// A registered account.
#[derive(Debug, Clone)]
pub struct User {
    pub username: String,
    pub title: Option<Title>,
    pub rating: Rating,
    pub games_played: u64,
    pub online: bool,
    salt: String,
    password_hash: u64,
}

impl User {
    /// Creates a new account, hashing the password with a generated salt.
    pub fn register(username: &str, password: &str) -> User {
        let salt = salt_for(username);
        let password_hash = hash_password(password, &salt);
        User {
            username: username.to_string(),
            title: None,
            rating: Rating::default(),
            games_played: 0,
            online: false,
            salt,
            password_hash,
        }
    }

    /// Checks a cleartext password against the stored hash.
    pub fn authenticate(&self, password: &str) -> bool {
        hash_password(password, &self.salt) == self.password_hash
    }

    /// Folds a game result into this user's rating.
    pub fn record_game(&mut self, opponent: Rating, result: GameResult) {
        self.rating = self.rating.update(&[(opponent, result)]);
        self.games_played += 1;
    }
}

/// A deterministic per-account salt. A real deployment must use a random one.
fn salt_for(username: &str) -> String {
    format!("{:016x}", fnv1a(username.as_bytes(), 0xcbf29ce484222325))
}

/// A deliberately simple iterated FNV-1a hash.
///
/// NOTE: this is *not* a cryptographically secure password hash; it exists so
/// that the demo platform has no external dependencies. Replace it with Argon2
/// or bcrypt before using this outside of a demo.
fn hash_password(password: &str, salt: &str) -> u64 {
    let mut hash = fnv1a(salt.as_bytes(), 0xcbf29ce484222325);
    for _ in 0..1_000 {
        hash = fnv1a(password.as_bytes(), hash);
        hash = fnv1a(salt.as_bytes(), hash);
    }
    hash
}

fn fnv1a(bytes: &[u8], seed: u64) -> u64 {
    let mut hash = seed;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authenticates_the_right_password() {
        let user = User::register("alice", "hunter2");
        assert!(user.authenticate("hunter2"));
        assert!(!user.authenticate("hunter3"));
    }

    #[test]
    fn different_accounts_hash_differently() {
        let a = User::register("alice", "same");
        let b = User::register("bob", "same");
        assert_ne!(a.password_hash, b.password_hash);
    }

    #[test]
    fn recording_a_win_updates_the_rating() {
        let mut user = User::register("carol", "pw");
        let before = user.rating;
        user.record_game(Rating::default(), GameResult::Win);
        assert!(user.rating.rating > before.rating);
        assert_eq!(user.games_played, 1);
    }
}
