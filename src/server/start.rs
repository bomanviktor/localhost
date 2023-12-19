use crate::server::{handle_client, ListenerConfig, Server};
use crate::server_config::server_config;
use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::sync::Arc;

pub fn servers() -> Vec<Server<'static>> {
    let mut servers = Vec::new();

    for config in server_config() {
        let mut listeners = Vec::new();
        for port in &config.ports {
            // Create a listener for each port
            let address = format!("{}:{}", config.host, port);
            match TcpListener::bind(address.parse::<SocketAddr>().unwrap()) {
                Ok(listener) => {
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

pub fn start(servers: Vec<Server<'static>>) {
    let mut poll = Poll::new().expect("Failed to create Poll instance");
    let mut events = Events::with_capacity(128);
    let mut connections = HashMap::new();
    let mut token_id = 2; // Start token counting from 2
    let mut all_listeners = Vec::new(); // Store all listeners
    let mut listener_configs = Vec::new(); // Store ListenerConfig instances

    for server in servers {
        let config = Arc::new(server.config);

        for listener in server.listeners {
            let listener_index = all_listeners.len();
            all_listeners.push(listener);
            let token = Token(token_id);
            token_id += 1;

            poll.registry()
                .register(
                    &mut all_listeners[listener_index],
                    token,
                    Interest::READABLE,
                )
                .expect("Failed to register listener");

            listener_configs.push(ListenerConfig {
                listener_index,
                config: Arc::clone(&config),
            });
        }
    }

    // Event loop
    loop {
        poll.poll(&mut events, None).expect("Poll failed");
        let mut to_close = Vec::new();

        for event in events.iter() {
            let token = event.token();

            if let Some(listener_config) = listener_configs
                .iter()
                .find(|lc| token == Token(lc.listener_index + 2))
            {
                let listener = &mut all_listeners[listener_config.listener_index];

                // Accept new connections in a loop
                loop {
                    match listener.accept() {
                        Ok((mut stream, _)) => {
                            let connection_token = Token(token_id);
                            println!("New connection on {}", listener.local_addr().unwrap());
                            token_id += 1;

                            poll.registry()
                                .register(&mut stream, connection_token, Interest::READABLE)
                                .expect("Failed to register new connection");

                            connections.insert(
                                connection_token,
                                (stream, Arc::clone(&listener_config.config)),
                            );
                        }
                        Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                            // No more connections to accept
                            break;
                        }
                        Err(e) => eprintln!("Error accepting connection: {}", e),
                    }
                }
            } else if let Some((stream, config)) = connections.get_mut(&token) {
                // Handle existing connection
                if let Err(e) = handle_client(stream, config) {
                    if e.kind() != ErrorKind::WouldBlock {
                        // Mark connection for closure
                        println!("Marking connection for closure due to error: {}", e);
                        to_close.push(token);
                    }
                }
            }
        }
        //  Close marked connections
        for token in to_close {
            if let Some((mut stream, _)) = connections.remove(&token) {
                poll.registry()
                    .deregister(&mut stream)
                    .expect("Failed to deregister stream");
            }
        }
    }
}
