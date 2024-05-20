mod config;
mod socket;

fn main() {
    let config =
        config::Config::from_file("server_config.yml").expect("Failed to load configuration");

    println!("Server is running!");

    socket::event_loop(&config).expect("Failed to run event loop");
}
