mod config;
mod socket;
mod web_socket;

fn main() {
    let config =
        config::Config::from_file("server_config.yml").expect("Failed to load configuration");
    loop {
        println!("Choose a server to run:");
        println!("1. TCP server (Terminal)");
        println!("2. Web server");

        let mut user_choose = String::new();

        std::io::stdin()
            .read_line(&mut user_choose)
            .expect("Failed to read line");

        let server_choose: i32 = match user_choose.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Please enter a number");
                continue;
            }
        };

        match server_choose {
            1 => {
                println!("TCP Server is running!");
                socket::event_loop(&config).expect("Failed to run event loop");
                break;
            }
            2 => {
                println!("Web server is running!");
                web_socket::event_loop(&config).expect("Failed to run event loop");
                break;
            }
            _ => {
                println!("Invalid choice");
                continue;
            }
        };

        // println!("Closing connection!");
        //break;
    }
}
