use localhost::client::handle_client;
use localhost::server::Server;
use localhost::server_config::config::servers;
use std::io::ErrorKind;

fn main() {
    start(servers());
}

// Refactor this to its own module.
fn start(mut servers: Vec<Server>) {
    loop {
        for server in &mut servers {
            for listener in &mut server.listeners {
                match listener.accept() {
                    Ok((stream, _addr)) => handle_client(stream, &server.config),
                    Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                        // No incoming connections, continue to the next listener
                        continue;
                    }
                    Err(ref e) if e.kind() == ErrorKind::AlreadyExists => {
                        continue;
                    }
                    Err(e) => eprintln!("Error accepting connection: {}", e),
                }
            }
        }
        // Sleep for a short duration to avoid busy waiting
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
