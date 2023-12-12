use localhost::client::handle_client;
use localhost::server_config::config::listeners;
use std::net::TcpListener;

fn main() {
    start_server(listeners());
}

// Refactor this to its own module.
fn start_server(listeners: Vec<TcpListener>) {
    for listener in listeners.iter().cycle() {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => handle_client(stream), // Put connection in a vector.
                Err(e) => eprintln!("Error accepting connection: {}", e),
            }
        }

        // Loop through this vector to look for requests. Put them in a queue.

        // Handle requests, starting from the first one, and send appropriate response.
    }
}
