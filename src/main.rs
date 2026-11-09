//! The `lichess-server` binary: a small blocking HTTP server.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

use lichess::api::{App, Request};

fn main() {
    let address = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());
    let listener = TcpListener::bind(&address).expect("failed to bind address");
    println!("lichess-rs listening on http://{address}");

    let mut app = App::new();
    for connection in listener.incoming() {
        match connection {
            Ok(stream) => {
                if let Err(error) = handle_connection(stream, &mut app) {
                    eprintln!("connection error: {error}");
                }
            }
            Err(error) => eprintln!("accept error: {error}"),
        }
    }
}

fn handle_connection(mut stream: TcpStream, app: &mut App) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);

    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("GET").to_string();
    let path = parts.next().unwrap_or("/").to_string();

    let mut content_length = 0usize;
    loop {
        let mut header = String::new();
        if reader.read_line(&mut header)? == 0 {
            break;
        }
        let header = header.trim_end();
        if header.is_empty() {
            break;
        }
        if let Some(value) = header
            .strip_prefix("Content-Length:")
            .or_else(|| header.strip_prefix("content-length:"))
        {
            content_length = value.trim().parse().unwrap_or(0);
        }
    }

    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body)?;
    }
    let body = String::from_utf8_lossy(&body).into_owned();

    let response = app.handle(&Request { method, path, body });

    let payload = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        response.status,
        reason(response.status),
        response.content_type,
        response.body.len(),
        response.body
    );
    stream.write_all(payload.as_bytes())?;
    stream.flush()
}

fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        409 => "Conflict",
        _ => "OK",
    }
}
