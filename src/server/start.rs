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

struct ServerState<'a> {
    poll: Poll,
    events: Events,
    token_id: usize,
    all_listeners: Vec<TcpListener>, // Replace with the actual type of your listeners
    clients: Vec<Client<'a>>, // Replace with the actual type of your clients
    connections: HashMap<Token, (TcpStream, Arc<ServerConfig<'a>>)>,
    to_close: Vec<Token>,
}

fn initialize_server_state(servers: Vec<Server<'static>>) -> ServerState<'static> {
    let mut poll = Poll::new().expect("Failed to create Poll instance");
    let events = Events::with_capacity(1024);
    let mut token_id = INITIAL_TOKEN_ID;
    let mut all_listeners = Vec::new();
    let mut clients = Vec::new();
    let connections = HashMap::new();
    let to_close = Vec::new();

    for server in servers {
        register_listeners(
            &mut poll,
            &mut all_listeners,
            &mut token_id,
            &mut clients,
            server,
        );
    }

    ServerState {
        poll,
        events,
        token_id,
        all_listeners,
        clients,
        connections,
        to_close,
    }
}

fn poll_and_handle_events(server_state: &mut ServerState<'static>) {
    server_state
        .poll
        .poll(&mut server_state.events, None)
        .expect("Poll failed");

    for event in server_state.events.iter() {
        let token = event.token();
        if let Some(client) = server_state.clients
            .iter()
            .find(|client| token == Token(client.id + INITIAL_TOKEN_ID))
        {
            let listener = &mut server_state.all_listeners[client.id];

            while accept_connection(&mut server_state.poll, listener, &mut server_state.token_id, client, &mut server_state.connections) {}
        }
        handle_existing_connection(&mut server_state.to_close, token, &mut server_state.connections);

        // Handle events using server_state
        // You can pass server_state to other functions as needed
    }
}

const INITIAL_TOKEN_ID: usize = 0;

pub fn start(servers: Vec<Server<'static>>) {
    let mut server_state = initialize_server_state(servers);
    loop {
        poll_and_handle_events(&mut server_state);
        close_marked_connections(&mut server_state);
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
            match e.kind() {
                ErrorKind::BrokenPipe => eprintln!("Client disconnected: {e}"),
                ErrorKind::WouldBlock => eprintln!("Client is blocking: {e}"),
                _ => eprintln!("Error handling client: {e}"),
            }
        }
    }
    to_close.push(token);
}

fn close_marked_connections(server_state: &mut ServerState<'static>) {
    for token in server_state.to_close.iter() {
        if let Some((mut stream, _)) = server_state.connections.remove(token) {
            server_state.poll.registry()
                .deregister(&mut stream)
                .expect("Failed to deregister stream");
        }
    }
}
