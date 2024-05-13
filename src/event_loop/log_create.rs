use std::fs;
use std::fs::File;
use std::io::prelude::*;

//to-do: verificar ciclo de vida!

pub fn log_create(log: &str) -> std::io::Result<String> {
    let dir_path = "logs";
    let file_path = "logs/log.txt";

    fs::create_dir_all(dir_path)?;

    let mut file = match File::open(file_path) {
        Ok(mut file) => {
            file.write_all(log.as_bytes())?;
            file
        }
        Err(_) => match File::create(file_path) {
            Ok(file) => {
                println!("File log.txt created!");
                file
            }
            Err(error) => {
                println!("Error when trying to create file! {}", error);
                return Err(error);
            }
        },
    };

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    Ok(contents)
}
