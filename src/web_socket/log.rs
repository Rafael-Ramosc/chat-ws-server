use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

#[allow(dead_code)]
pub fn log_create(log: &str) -> std::io::Result<()> {
    let dir_path = Path::new("logs");
    let file_path = dir_path.join("log.txt");

    fs::create_dir_all(dir_path)?;

    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(&file_path)?;

    write!(file, "{}", log)?;

    Ok(())
}
