use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::time::Duration;

fn main() -> Result<(), std::io::Error> {
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(1024);
    let mut clients = HashMap::new();

    let address = "127.0.0.1:8000".parse().unwrap();
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
                            println!("Accepted connection from: {}", address);

                            let token = next_token;
                            next_token.0 += 1;

                            poll.registry()
                                .register(&mut connection, token, Interest::READABLE)
                                .unwrap();

                            let message = "Connection established!";
                            connection.write_all(message.as_bytes()).unwrap();

                            clients.insert(token, connection);
                        }
                        Err(ref err) if would_block(err) => break,
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

                                println!(
                                    "{} SAY: {}",
                                    connection.peer_addr().unwrap(),
                                    received_data
                                );

                                let message = "Message received!";
                                connection.write_all(message.as_bytes()).unwrap();
                            }
                            Err(ref err) if would_block(err) => {
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

fn would_block(err: &std::io::Error) -> bool {
    err.kind() == std::io::ErrorKind::WouldBlock
}
