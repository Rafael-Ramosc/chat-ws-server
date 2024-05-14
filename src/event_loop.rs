mod log_create;

use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::time::Duration;

pub fn event_loop() -> Result<(), std::io::Error> {
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(1024);
    let mut clients = HashMap::new();

    let address = "0.0.0.0:8000".parse().unwrap();
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

                            let message = "Connection established!\n";
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
                                let message_to_sender: String;

                                let log = log_message(&connection, &received_data.as_ref());
                                print!("{}", log);

                                match log_create::log_create(&log) {
                                    Ok(()) => {}
                                    Err(e) => {
                                        eprintln!("Erro with log: {}", e);
                                    }
                                }

                                message_to_sender = client_message(received_data.as_ref());

                                connection.write_all(message_to_sender.as_bytes()).unwrap();

                                chat_message(
                                    &mut clients,
                                    token,
                                    &connection,
                                    &received_data.as_ref(),
                                );
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

fn client_message(received_data: &str) -> String {
    let message_to_client = format!("YOU: {}", received_data);
    message_to_client
}

fn chat_message(
    clients: &mut HashMap<Token, TcpStream>,
    token: Token,
    connection: &TcpStream,
    received_data: &str,
) {
    for (other_token, other_connection) in clients.iter_mut() {
        if other_token != &token {
            let message = format!("{} SAY: {}", connection.peer_addr().unwrap(), received_data);

            other_connection
                .write_all(message.as_bytes())
                .expect("Failed tp write to client");
        }
    }
}

fn log_message(connection: &TcpStream, received_data: &str) -> String {
    let log = format!(
        "LOG :{} SAY: {}",
        connection.peer_addr().unwrap(),
        received_data
    );

    log
}
