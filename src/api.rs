//! A minimal, dependency-free HTTP API exposing the platform state.
//!
//! The [`App`] type holds all mutable state (games, users, tournaments) and turns
//! a parsed [`Request`] into a [`Response`]. It is deliberately framework-free so
//! that the whole crate builds and tests with a plain `cargo test`.

use std::collections::HashMap;

use crate::game::{Game, GameStatus};
use crate::tournament::Arena;
use crate::user::User;

/// A parsed HTTP request.
#[derive(Debug, Clone)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub body: String,
}

impl Request {
    pub fn new(method: &str, path: &str) -> Request {
        Request {
            method: method.to_string(),
            path: path.to_string(),
            body: String::new(),
        }
    }

    pub fn with_body(mut self, body: &str) -> Request {
        self.body = body.to_string();
        self
    }
}

/// A response ready to be written to the wire.
#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub content_type: String,
    pub body: String,
}

impl Response {
    pub fn json(status: u16, body: impl Into<String>) -> Response {
        Response {
            status,
            content_type: "application/json".to_string(),
            body: body.into(),
        }
    }

    pub fn text(status: u16, body: impl Into<String>) -> Response {
        Response {
            status,
            content_type: "text/plain; charset=utf-8".to_string(),
            body: body.into(),
        }
    }

    pub fn not_found() -> Response {
        Response::text(404, "not found")
    }
}

/// The in-memory application state.
#[derive(Debug)]
pub struct App {
    pub games: HashMap<String, Game>,
    pub users: HashMap<String, User>,
    pub tournaments: HashMap<String, Arena>,
    next_id: u64,
}

impl Default for App {
    fn default() -> Self {
        App::new()
    }
}

impl App {
    pub fn new() -> App {
        App {
            games: HashMap::new(),
            users: HashMap::new(),
            tournaments: HashMap::new(),
            next_id: 1,
        }
    }

    fn allocate_id(&mut self) -> String {
        let id = format!("{:08}", self.next_id);
        self.next_id += 1;
        id
    }

    /// Routes a request to the right handler.
    pub fn handle(&mut self, request: &Request) -> Response {
        let segments: Vec<&str> = request
            .path
            .split('?')
            .next()
            .unwrap_or("")
            .trim_matches('/')
            .split('/')
            .filter(|segment| !segment.is_empty())
            .collect();

        match request.method.as_str() {
            "GET" => self.get(&segments),
            "POST" => self.post(&segments, &request.body),
            _ => Response::text(405, "method not allowed"),
        }
    }

    fn get(&mut self, segments: &[&str]) -> Response {
        match segments {
            [] => Response::text(200, "lichess-rs — a Rust clone of lichess.org"),
            ["api"] => Response::json(200, r#"{"name":"lichess-rs","version":"0.1.0"}"#),
            ["api", "game", id] => match self.games.get(*id) {
                Some(game) => Response::json(200, game_json(*id, game)),
                None => Response::not_found(),
            },
            ["api", "user", name] => match self.users.get(*name) {
                Some(user) => Response::json(200, user_json(user)),
                None => Response::not_found(),
            },
            ["api", "tournament"] => {
                let items: Vec<String> = self
                    .tournaments
                    .iter()
                    .map(|(id, arena)| {
                        format!(
                            "{{\"id\":\"{id}\",\"name\":\"{}\",\"status\":\"{:?}\"}}",
                            arena.name, arena.status
                        )
                    })
                    .collect();
                Response::json(200, format!("[{}]", items.join(",")))
            }
            ["api", "tournament", id] => match self.tournaments.get(*id) {
                Some(arena) => Response::json(200, arena_json(*id, arena)),
                None => Response::not_found(),
            },
            _ => Response::not_found(),
        }
    }

    fn post(&mut self, segments: &[&str], body: &str) -> Response {
        match segments {
            ["api", "game"] => {
                let id = self.allocate_id();
                self.games.insert(id.clone(), Game::new());
                Response::json(200, format!("{{\"id\":\"{id}\"}}"))
            }
            ["api", "game", id, "move"] => {
                let uci = body.trim();
                match self.games.get_mut(*id) {
                    Some(game) => match game.play_uci(uci) {
                        Ok(mv) => Response::json(
                            200,
                            format!(
                                "{{\"ok\":true,\"move\":\"{}\",\"fen\":\"{}\"}}",
                                mv.to_uci(),
                                game.current_fen()
                            ),
                        ),
                        Err(error) => {
                            Response::json(400, format!("{{\"ok\":false,\"error\":\"{error}\"}}"))
                        }
                    },
                    None => Response::not_found(),
                }
            }
            ["api", "game", id, "resign"] => match self.games.get_mut(*id) {
                Some(game) => {
                    if game.is_finished() {
                        return Response::json(400, "{\"ok\":false,\"error\":\"game is over\"}");
                    }
                    let turn = game.turn();
                    let _ = game.resign(turn);
                    Response::json(200, "{\"ok\":true}")
                }
                None => Response::not_found(),
            },
            ["api", "user"] => {
                let mut parts = body.trim().splitn(2, ':');
                let username = parts.next().unwrap_or("").trim();
                let password = parts.next().unwrap_or("");
                if username.is_empty() || password.is_empty() {
                    return Response::json(400, "{\"error\":\"expected 'username:password'\"}");
                }
                if self.users.contains_key(username) {
                    return Response::json(409, "{\"error\":\"username already taken\"}");
                }
                self.users
                    .insert(username.to_string(), User::register(username, password));
                Response::json(200, format!("{{\"username\":\"{username}\"}}"))
            }
            ["api", "tournament"] => {
                let id = self.allocate_id();
                self.tournaments.insert(
                    id.clone(),
                    Arena::new(id.clone(), format!("Arena {id}"), 10),
                );
                Response::json(200, format!("{{\"id\":\"{id}\"}}"))
            }
            _ => Response::not_found(),
        }
    }
}

fn game_json(id: &str, game: &Game) -> String {
    format!(
        "{{\"id\":\"{id}\",\"fen\":\"{}\",\"status\":\"{}\",\"result\":\"{}\"}}",
        game.current_fen(),
        status_name(game),
        game.result()
    )
}

fn status_name(game: &Game) -> &'static str {
    match game.status {
        GameStatus::Created => "created",
        GameStatus::Started => "started",
        GameStatus::Finished(_) => "finished",
    }
}

fn user_json(user: &User) -> String {
    format!(
        "{{\"username\":\"{}\",\"rating\":{:.0},\"games\":{}}}",
        user.username, user.rating.rating, user.games_played
    )
}

fn arena_json(id: &str, arena: &Arena) -> String {
    let standings: Vec<String> = arena
        .standings()
        .iter()
        .map(|(name, score)| format!("{{\"name\":\"{name}\",\"score\":{score}}}"))
        .collect();
    format!(
        "{{\"id\":\"{id}\",\"name\":\"{}\",\"standings\":[{}]}}",
        arena.name,
        standings.join(",")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id_from(body: &str) -> String {
        body.split("\"id\":\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap()
            .to_string()
    }

    #[test]
    fn index_responds() {
        let mut app = App::new();
        assert_eq!(app.handle(&Request::new("GET", "/")).status, 200);
    }

    #[test]
    fn create_and_play_a_game() {
        let mut app = App::new();
        let created = app.handle(&Request::new("POST", "/api/game"));
        assert_eq!(created.status, 200);
        let id = id_from(&created.body);

        let played =
            app.handle(&Request::new("POST", &format!("/api/game/{id}/move")).with_body("e2e4"));
        assert_eq!(played.status, 200);
        assert!(played.body.contains("\"ok\":true"));

        let fetched = app.handle(&Request::new("GET", &format!("/api/game/{id}")));
        assert!(fetched.body.contains("fen"));
    }

    #[test]
    fn register_and_fetch_user() {
        let mut app = App::new();
        let registered = app.handle(&Request::new("POST", "/api/user").with_body("alice:s3cret"));
        assert_eq!(registered.status, 200);

        let fetched = app.handle(&Request::new("GET", "/api/user/alice"));
        assert_eq!(fetched.status, 200);
        assert!(fetched.body.contains("alice"));
    }

    #[test]
    fn unknown_routes_are_404() {
        let mut app = App::new();
        assert_eq!(app.handle(&Request::new("GET", "/nope")).status, 404);
    }
}
