//! # Event Loop
//!
//! This module contains the main event loop for the server application. It sets up a TCP listener,
//! accepts incoming connections, and handles client communication using the Mio library for
//! asynchronous I/O.

pub mod log;
pub mod message_control;

use crate::config::Config;
use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::time::Duration;

/// The main event loop function that drives the server application.
///
/// # Parameters
///
/// - `config: &Config` - A reference to the server configuration.
///
/// # Returns
///
/// - `Result<(), std::io::Error>` - Returns `Ok(())` if the event loop runs successfully, or an
///   `Err` containing an `std::io::Error` if an error occurs.

pub fn event_loop(config: &Config) -> Result<(), std::io::Error> {
    //Create a new Poll instance and an Events buffer with a capacity of 1024.
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(1024);

    //Initialize an empty HashMap called clients to store client connections.
    let mut clients = HashMap::new();

    //Bind a TcpListener to the server's IP address and port specified in the configuration.
    let server_ip_port = format!("{}:{}", config.server.host, config.server.port);
    let address = server_ip_port.parse().unwrap();
    let mut listener = TcpListener::bind(address)?; //TODO: verificar se ja tiver ip e porta em uso

    //Register the TcpListener with the Poll instance using the SERVER token.
    const SERVER: Token = Token(0);
    poll.registry()
        .register(&mut listener, SERVER, Interest::READABLE)
        .unwrap();

    let mut next_token = Token(1);

    loop {
        //Poll for events with a timeout of 100 milliseconds.
        poll.poll(&mut events, Some(Duration::from_millis(100)))?;

        for event in events.iter() {
            match event.token() {
                //If the event token is SERVER
                //Accept new client connections in a loop until a WouldBlock error occurs.
                SERVER => loop {
                    match listener.accept() {
                        Ok((mut connection, address)) => {
                            let address_str = address.to_string();

                            message_control::accept_connection(address_str.clone(), &connection);

                            //Assign a unique token to the connection
                            let token = next_token;
                            next_token.0 += 1;

                            //Register the connection with the Poll instance.
                            poll.registry()
                                .register(&mut connection, token, Interest::READABLE)
                                .unwrap();

                            //Insert the connection into the clients map.
                            clients.insert(token, connection);

                            //Broadcast a connection message to all clients
                            message_control::accept_connection_all(
                                &format!("{} connected\n", address_str),
                                &clients,
                            );
                        }
                        Err(ref err) if message_control::would_block(err) => break,
                        Err(err) => return Err(err),
                    }
                },
                //If the event token is a client token.
                token => {
                    //Remove the corresponding client connection from the clients map.
                    let mut connection = clients.remove(&token).unwrap();

                    //Read data from the connection in a loop until a WouldBlock error occurs or the connection is closed.
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

                                //Create a log entry for the message.
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
                                //If the connection is still open, insert it back into the clients map.
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
