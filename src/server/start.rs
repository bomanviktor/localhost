use crate::server::state::*;
use crate::server::{Server, SocketAddr, TcpListener};
use crate::server_config::server_config;
use crate::type_aliases::Port;

pub fn start(servers: Vec<Server<'static>>) {
    let mut s = ServerState::init(servers);
    loop {
        s.poll();
        s.handle_events();
    }
}

fn bind_port(host: &str, port: &Port) -> Option<TcpListener> {
    let address = format!("{}:{}", host, port);
    let socket_addr = match address.parse::<SocketAddr>() {
        Ok(address) => address,
        Err(e) => {
            eprintln!("Error: {e}. Unable to listen to: {address}");
            return None;
        }
    };

    match TcpListener::bind(socket_addr) {
        Ok(listener) => {
            println!("Server listening on {address}");
            Some(listener)
        }
        Err(e) => {
            eprintln!("Error: {e}. Unable to listen to: {address}");
            None
        }
    }
}

#[test]
fn test_bind_port() {
    let invalid_port: Port = 123;
    assert!(bind_port("oogabooga", &invalid_port).is_none());
}

// Updated servers function
pub fn servers() -> Vec<Server<'static>> {
    let mut servers = Vec::new();

    for config in server_config() {
        if config.ports.is_empty() {
            eprintln!(
                "Error: no ports are specified for this instance of {}",
                config.host
            );
        }

        let listeners = config
            .ports
            .iter()
            .filter_map(|port| bind_port(config.host, port))
            .collect::<Vec<_>>();

        if !listeners.is_empty() {
            // Only create a server if there are successful listeners
            servers.push(Server::new(listeners, config));
        }
    }
    servers
}
