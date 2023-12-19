use crate::server::{handle_client, Client, Server};
use crate::server_config::{server_config, ServerConfig};
use mio::net::{TcpListener, TcpStream};
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

const INITIAL_TOKEN_ID: usize = 0;

pub fn start(servers: Vec<Server<'static>>) {
    let mut poll = Poll::new().expect("Failed to create Poll instance");
    let mut events = Events::with_capacity(1024);
    let mut token_id = INITIAL_TOKEN_ID;
    let mut all_listeners = Vec::new();
    let mut listener_configs = Vec::new();
    let mut connections = HashMap::new();
    let mut to_close = Vec::new(); // List of connections to close

    for server in servers {
        register_listeners(
            &mut poll,
            &mut all_listeners,
            &mut token_id,
            &mut listener_configs,
            server,
        );
    }

    loop {
        poll.poll(&mut events, None).expect("Poll failed");

        for event in events.iter() {
            let token = event.token();

            // Find and accept the connection
            if let Some(listener_config) = listener_configs
                .iter()
                .find(|client| token == Token(client.id + INITIAL_TOKEN_ID))
            {
                let listener = &mut all_listeners[listener_config.id];

                loop {
                    if !accept_connection(
                        &mut poll,
                        listener,
                        &mut token_id,
                        listener_config,
                        &mut connections,
                    ) {
                        break;
                    }
                }
            }
            handle_existing_connection(&mut to_close, token, &mut connections);
        }

        close_marked_connections(&mut poll, &mut connections, &to_close);
    }
}

fn register_listeners(
    poll: &mut Poll,
    all_listeners: &mut Vec<TcpListener>,
    token_id: &mut usize,
    clients: &mut Vec<Client>,
    server: Server<'static>,
) {
    let config = Arc::new(server.config);

    server.listeners.into_iter().for_each(|listener| {
        let id = all_listeners.len();
        all_listeners.push(listener);
        let token = Token(*token_id);
        *token_id += 1;

        poll.registry()
            .register(&mut all_listeners[id], token, Interest::READABLE)
            .expect("Failed to register listener");

        clients.push(Client {
            id,
            config: Arc::clone(&config),
        });
    });
}

fn accept_connection<'a>(
    poll: &mut Poll,
    listener: &mut TcpListener, // Replace with the actual type of your listeners
    token_id: &mut usize,
    client: &Client<'a>,
    connections: &mut HashMap<Token, (TcpStream, Arc<ServerConfig<'a>>)>,
) -> bool {
    match listener.accept() {
        Ok((mut stream, _)) => {
            let connection_token = Token(*token_id);
            *token_id += 1;

            poll.registry()
                .register(&mut stream, connection_token, Interest::READABLE)
                .expect("Failed to register new connection");

            connections.insert(connection_token, (stream, Arc::clone(&client.config)));
            true
        }
        _ => false,
    }
}

fn handle_existing_connection(
    to_close: &mut Vec<Token>,
    token: Token,
    connections: &mut HashMap<Token, (TcpStream, Arc<ServerConfig>)>,
) {
    if let Some((stream, config)) = connections.get_mut(&token) {
        if let Err(e) = handle_client(stream, config) {
            if e.kind() != ErrorKind::WouldBlock {
                // Mark connection for closure
                println!("Marking connection for closure due to error: {}", e);
            } else {
                return;
            }
        }
    }
    to_close.push(token);
}

fn close_marked_connections(
    poll: &mut Poll,
    connections: &mut HashMap<Token, (TcpStream, Arc<ServerConfig>)>,
    to_close: &Vec<Token>,
) {
    for token in to_close {
        if let Some((mut stream, _)) = connections.remove(token) {
            poll.registry()
                .deregister(&mut stream)
                .expect("Failed to deregister stream");
        }
    }
}
