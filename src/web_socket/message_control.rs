use base64::{engine::general_purpose, Engine};
use sha1::{Digest, Sha1};

pub fn gen_key(key: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(key.as_bytes());
    hasher.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
    let hash = hasher.finalize();
    general_purpose::STANDARD.encode(hash)
}

pub fn would_block(err: &std::io::Error) -> bool {
    err.kind() == std::io::ErrorKind::WouldBlock
}
