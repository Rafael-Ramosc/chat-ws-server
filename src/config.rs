use serde::Deserialize;
use std::fs;

#[derive(Debug, PartialEq, Deserialize)]
pub struct Server {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Config {
    pub server: Server,
}

impl Config {
    pub fn from_file(file_path: &str) -> Result<Config, String> {
        let yaml_str =
            fs::read_to_string(file_path).map_err(|e| format!("Failed to read file: {}", e))?;

        serde_yaml::from_str(&yaml_str).map_err(|e| format!("Failed to parse YAML: {}", e))
    }
}
