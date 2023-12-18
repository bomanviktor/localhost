use crate::server::{handle_client, Server};
use crate::server_config::server_config;
use std::io::ErrorKind;
use std::net::TcpListener;

pub fn servers() -> Vec<Server<'static>> {
    let mut servers = Vec::new();

    for config in server_config() {
        let mut listeners = Vec::new();
        for port in &config.ports {
            // Create a listener for each port
            let address = format!("{}:{}", config.host, port);
            match TcpListener::bind(&address) {
                Ok(listener) => {
                    listener.set_nonblocking(true).unwrap();
                    listeners.push(listener);
                    println!("Server listening on {}", address);
                }
                Err(e) => eprintln!("Error: {}. Unable to listen to: {}", e, address),
            }
        }
        // Make a server and push it to the servers vector
        servers.push(Server::new(listeners, config))
    }
    servers
}

pub fn start(mut servers: Vec<Server>) {
    loop {
        for server in &mut servers {
            for listener in &mut server.listeners {
                match listener.accept() {
                    Ok((mut stream, _addr)) => match handle_client(&mut stream, &server.config) {
                        Ok(_) => continue,
                        Err(e) => eprintln!("Error: {e}"),
                    },
                    Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                        // No incoming connections, continue to the next listener
                        continue;
                    }
                    Err(e) => eprintln!("Error accepting connection: {}", e),
                }
            }
        }
        // Sleep for a short duration to avoid busy waiting
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
