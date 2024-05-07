use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;
use std::time::Duration;

fn main() -> Result<(), std::io::Error> {
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(1024);

    // let mut clients = HashMap::new();

    let address = "127.0.0.1:8000".parse().unwrap();
    let mut listener = TcpListener::bind(address)?;

    const SERVER: Token = Token(0);
    poll.registry()
        .register(&mut listener, SERVER, Interest::READABLE)
        .unwrap();

    loop {
        poll.poll(&mut events, Some(Duration::from_millis(100)))?;

        for event in events.iter() {
            match event.token() {
                SERVER => loop {
                    match listener.accept() {
                        Ok((connection, address)) => {
                            println!("Accepted connection from: {}", address);
                        }
                        Err(ref err) if would_block(err) => break,
                        Err(err) => return Err(err),
                    }
                },
                _ => unreachable!(),
            }
        }
    }
}

fn would_block(err: &std::io::Error) -> bool {
    err.kind() == std::io::ErrorKind::WouldBlock
}
