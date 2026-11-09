# lichess-rs

A modular, dependency-free Rust re-implementation of the core of
[lichess.org](https://lichess.org).

The previous demo files (`auth.py`, `utils.py`, `package.json`) were removed and
replaced by this Rust codebase.

## Layout

| Module | Responsibility |
| ------ | -------------- |
| `piece`, `square`, `moves` | Fundamental chess types |
| `board` | Position, FEN, move generation, make/unmake, perft |
| `san`, `pgn` | Algebraic notation and PGN import/export |
| `game` | Game state machine, outcomes, draw rules |
| `clock` | Time controls and the Lichess speed buckets |
| `rating` | Glicko-2 rating updates |
| `user` | Accounts and authentication |
| `challenge` | Challenges between players |
| `tournament` | Arena tournaments and standings |
| `puzzle` | Tactics puzzle sessions |
| `opening` | A tiny opening book |
| `api` | In-memory application state and a small HTTP router |

## Running the tests

```sh
cargo test
```

The core game logic is verified with unit tests, including perft counts on the
initial position and on the *Kiwipete* position, plus checkmate, stalemate,
promotion, en passant and castling tests.

## Running the server

```sh
cargo run --bin lichess-server -- 127.0.0.1:8080
```

Then, for example:

```sh
curl -X POST localhost:8080/api/game                      # create a game -> {"id":"00000001"}
curl -X POST localhost:8080/api/game/00000001/move -d e2e4
curl localhost:8080/api/game/00000001
curl -X POST localhost:8080/api/user -d alice:s3cret
curl localhost:8080/api/user/alice
```
