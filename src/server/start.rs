use std::net::ToSocketAddrs;
use std::process::exit;

use crate::server::{Server, ServerState, TcpListener};
use crate::server_config::ServerConfig;
use crate::type_aliases::Port;

pub fn start(configs: Vec<ServerConfig<'static>>) {
    let servers = get_servers(configs);
    if servers.is_empty() {
        eprintln!("No servers were added. Exit program.");
        exit(1);
    }
    let mut s = ServerState::init(servers);
    loop {
        s.poll();
        s.handle_events();
    }
}

fn bind_port(host: &str, port: &Port) -> Option<TcpListener> {
    // Use ToSocketAddrs to resolve the hostname to an IP address
    let host_and_port = format!("{host}:{port}");
    let mut addresses = match host_and_port.to_socket_addrs() {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("Error resolving address {host}:{port}. {e}");
            return None;
        }
    };

    if let Some(socket_addr) = addresses.next() {
        return match TcpListener::bind(socket_addr) {
            Ok(listener) => {
                println!("Server listening on {host_and_port}");
                Some(listener)
            }
            Err(e) => {
                eprintln!("Error: {e}. Unable to listen to: {host_and_port}");
                None
            }
        };
    }
    None
}

pub fn get_servers(configs: Vec<ServerConfig<'static>>) -> Vec<Server<'static>> {
    let mut servers = Vec::new();
    for config in configs {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::init_logs;
    #[test]
    fn test_bind_port() {
        // Invalid address
        let valid_port: Port = 8080;
        let invalid_addr = "foo";
        assert!(bind_port(invalid_addr, &valid_port).is_none());

        init_logs();
        // Invalid ports
        let invalid_port: Port = 1;
        let valid_addr = "127.0.0.1";
        assert!(bind_port(valid_addr, &invalid_port).is_none());
    }

    #[test]
    fn test_get_servers() {
        let server_config = ServerConfig {
            host: "127.0.0.1",
            ports: vec![],
            custom_error_path: None,
            body_size_limit: 0,
            routes: vec![],
        };
        assert!(get_servers(vec![server_config]).is_empty());
    }
}
