use mio::net::TcpStream;
use mio::Token;
use std::collections::HashMap;
use std::io::Write;

pub fn would_block(err: &std::io::Error) -> bool {
    err.kind() == std::io::ErrorKind::WouldBlock
}

pub fn client_message(received_data: &str) -> String {
    let message_to_client = format!("YOU: {}", received_data);
    message_to_client
}

pub fn chat_message(
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

pub fn log_message(connection: &TcpStream, received_data: &str) -> String {
    let log = format!(
        "LOG :{} SAY: {}",
        connection.peer_addr().unwrap(),
        received_data
    );

    log
}

pub fn accept_connection(address: String, mut connection: &TcpStream) {
    println!("Accepted connection from: {}", address);

    let message = "Connection established!\n";
    connection.write_all(message.as_bytes()).unwrap();
}
