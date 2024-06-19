//! # Event Loop
//!
//! This module contains the main event loop for the WebSocket server application. It sets up a TCP listener,
//! accepts incoming connections, and handles client communication using the Mio library for
//! asynchronous I/O and the http_muncher library for WebSocket handling.

pub mod log;
pub mod message_control;

use crate::config::Config;
use http_muncher::{Parser, ParserHandler};
use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::time::Duration;

struct HttpParser {
    current_key: Option<String>,
    headers: HashMap<String, String>,
}

impl ParserHandler for HttpParser {
    fn on_header_field(&mut self, _parser: &mut Parser, s: &[u8]) -> bool {
        self.current_key = Some(String::from_utf8_lossy(s).to_string());
        true
    }

    fn on_header_value(&mut self, _parser: &mut Parser, s: &[u8]) -> bool {
        self.headers.insert(
            self.current_key.clone().unwrap(),
            String::from_utf8_lossy(s).to_string(),
        );
        true
    }

    fn on_headers_complete(&mut self, _parser: &mut Parser) -> bool {
        false
    }
}

impl HttpParser {
    fn new() -> Self {
        HttpParser {
            current_key: None,
            headers: HashMap::new(),
        }
    }
}

struct WebSocketClient {
    socket: mio::net::TcpStream,
    http_parser: Parser,
    interest: Interest,
    headers: HashMap<String, String>,
}

impl WebSocketClient {
    fn new(socket: mio::net::TcpStream) -> Self {
        WebSocketClient {
            socket,
            http_parser: Parser::request(),
            interest: Interest::READABLE,
            headers: HashMap::new(),
        }
    }

    fn read(&mut self, poll: &mut Poll, token: Token) -> Result<(), std::io::Error> {
        loop {
            let mut buffer = [0; 1024];
            match self.socket.read(&mut buffer) {
                Ok(0) => {
                    println!("Client disconnected");
                    break;
                }
                Ok(n) => {
                    self.http_parser.parse(&mut HttpParser::new(), &buffer[..n]);
                    if self.http_parser.is_upgrade() {
                        poll.registry()
                            .reregister(&mut self.socket, token, Interest::READABLE)?;
                        poll.registry()
                            .reregister(&mut self.socket, token, Interest::WRITABLE)?;
                        poll.registry()
                            .reregister(&mut self.socket, token, self.interest)?;
                        break;
                    }
                }
                Err(ref err) if message_control::would_block(err) => break,
                Err(err) => {
                    println!("Error: {}", err);
                    break;
                }
            }
        }
        Ok(())
    }

    fn write(&mut self, poll: &mut Poll, token: Token) {
        let response_key =
            message_control::gen_key(&self.headers.get("Sec-WebSocket-Key").unwrap());
        let response = format!(
            "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {}\r\n\r\n",
            response_key
        );
        self.socket.write_all(response.as_bytes()).unwrap();
        self.interest |= Interest::READABLE;
        poll.registry()
            .reregister(&mut self.socket, token, self.interest)
            .unwrap();
    }
}

pub fn event_loop(config: &Config) -> Result<(), std::io::Error> {
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(1024);
    let mut clients = HashMap::new();

    let server_ip_port = format!("{}:{}", config.server.host, config.server.port);
    let address = server_ip_port.parse().unwrap();
    println!("Server running on: {}", address);
    let mut listener = TcpListener::bind(address)?;

    const SERVER: Token = Token(0);
    poll.registry()
        .register(&mut listener, SERVER, Interest::READABLE)?;

    let mut next_token = Token(1);

    loop {
        poll.poll(&mut events, Some(Duration::from_millis(100)))?;

        for event in events.iter() {
            match event.token() {
                SERVER => loop {
                    match listener.accept() {
                        Ok((socket, address)) => {
                            let address_str = address.to_string();
                            println!("New client connected: {}", address_str);

                            let token = next_token;
                            next_token.0 += 1;

                            let mut client = WebSocketClient::new(socket);
                            poll.registry()
                                .register(&mut client.socket, token, client.interest)?;

                            clients.insert(token, client);
                        }
                        Err(ref err) if message_control::would_block(err) => break,
                        Err(err) => return Err(err),
                    }
                },
                token => {
                    let mut client = clients.remove(&token).unwrap();

                    if event.is_readable() {
                        client.read(&mut poll, token)?;
                    }

                    if event.is_writable() {
                        client.write(&mut poll, token);
                    }

                    if client.interest.is_readable() || client.interest.is_writable() {
                        poll.registry()
                            .reregister(&mut client.socket, token, client.interest)?;
                        clients.insert(token, client);
                    }
                }
            }
        }
    }
}
