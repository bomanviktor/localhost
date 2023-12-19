use crate::server::state::*;
use crate::server::{
    handle_client, Arc, Client, HashMap, Interest, Poll, Server, SocketAddr, TcpListener,
    TcpStream, Token,
};
use crate::server_config::{server_config, ServerConfig};
use std::io::ErrorKind;
use std::time::Duration;

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

pub const INITIAL_TOKEN_ID: usize = 0;

pub fn start(servers: Vec<Server<'static>>) {
    let mut state = initialize_server_state(servers);
    loop {
        state.to_close.clear();
        poll_and_handle_events(&mut state);
        close_marked_connections(&mut state);
    }
}

fn poll_and_handle_events(state: &mut ServerState<'static>) {
    state
        .poll
        .poll(&mut state.events, Some(Duration::from_millis(50)))
        .expect("Poll failed");

    for event in state.events.iter() {
        let token = event.token();
        if let Some(client) = state
            .clients
            .iter()
            .find(|client| token == Token(client.id + INITIAL_TOKEN_ID))
        {
            let listener = &mut state.all_listeners[client.id];

            // Keep accepting connections until error.
            while accept_connection(
                &mut state.poll,
                listener,
                &mut state.token_id,
                client,
                &mut state.connections,
            ) {}
        }

        handle_existing_connection(&mut state.to_close, token, &mut state.connections);
    }
}

fn accept_connection<'a>(
    poll: &mut Poll,
    listener: &mut TcpListener,
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
            match e.kind() {
                ErrorKind::BrokenPipe => eprintln!("Client disconnected: {e}"),
                ErrorKind::WouldBlock => eprintln!("Client is blocking: {e}"),
                _ => eprintln!("Error handling client: {e}"),
            }
        }
    }
    to_close.push(token);
}

fn close_marked_connections(state: &mut ServerState<'static>) {
    for token in state.to_close.iter() {
        if let Some((mut stream, _)) = state.connections.remove(token) {
            state
                .poll
                .registry()
                .deregister(&mut stream)
                .expect("Failed to deregister stream");
        }
    }
}
