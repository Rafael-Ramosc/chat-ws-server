pub mod log;
pub mod message_control;

use crate::config::Config;
use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::time::Duration;

pub fn event_loop(config: &Config) -> Result<(), std::io::Error> {
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(1024);
    let mut clients = HashMap::new();

    let server_ip_port = format!("{}:{}", config.server.host, config.server.port);
    let address = server_ip_port.parse().unwrap();
    let mut listener = TcpListener::bind(address)?;

    const SERVER: Token = Token(0);
    poll.registry()
        .register(&mut listener, SERVER, Interest::READABLE)
        .unwrap();

    let mut next_token = Token(1);

    loop {
        poll.poll(&mut events, Some(Duration::from_millis(100)))?;

        for event in events.iter() {
            match event.token() {
                SERVER => loop {
                    match listener.accept() {
                        Ok((mut connection, address)) => {
                            let address_str = address.to_string();

                            message_control::accept_connection(address_str.clone(), &connection);

                            let token = next_token;
                            next_token.0 += 1;

                            poll.registry()
                                .register(&mut connection, token, Interest::READABLE)
                                .unwrap();

                            clients.insert(token, connection);

                            message_control::accept_connection_all(
                                &format!("{} connected\n", address_str),
                                &clients,
                            );
                        }
                        Err(ref err) if message_control::would_block(err) => break,
                        Err(err) => return Err(err),
                    }
                },
                token => {
                    let mut connection = clients.remove(&token).unwrap();

                    loop {
                        let mut buffer = [0; 1024];
                        match connection.read(&mut buffer) {
                            Ok(0) => {
                                println!("Client disconnected");
                                break;
                            }
                            Ok(n) => {
                                let received_data = String::from_utf8_lossy(&buffer[..n]);
                                let message_to_sender: String;

                                let log = message_control::log_message(
                                    &connection,
                                    &received_data.as_ref(),
                                );
                                print!("{}", log);

                                match log::log_create(&log) {
                                    Ok(()) => {}
                                    Err(e) => {
                                        eprintln!("Erro with log: {}", e);
                                    }
                                }

                                message_to_sender =
                                    message_control::client_message(received_data.as_ref());

                                connection.write_all(message_to_sender.as_bytes()).unwrap();

                                message_control::chat_message(
                                    &mut clients,
                                    token,
                                    &connection,
                                    &received_data.as_ref(),
                                );
                            }
                            Err(ref err) if message_control::would_block(err) => {
                                clients.insert(token, connection);
                                break;
                            }
                            Err(err) => {
                                println!("Error: {}", err);
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}
