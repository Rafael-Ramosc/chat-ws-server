mod event_loop;

fn main() {
    event_loop::event_loop().expect("Failed to run event loop");
}
