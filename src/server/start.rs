use std::net::ToSocketAddrs;

use crate::server::state::*;
use crate::server::{Server, TcpListener};
use crate::server_config::ServerConfig;
use crate::type_aliases::Port;

pub fn start(configs: Vec<ServerConfig<'static>>) {
    let servers = get_servers(configs);
    let mut s = ServerState::init(servers);
    loop {
        s.poll();
        s.handle_events();
    }
}

fn bind_port(host: &str, port: &Port) -> Option<TcpListener> {
    // Use ToSocketAddrs to resolve the hostname to an IP address
    let addresses = format!("{}:{}", host, port).to_socket_addrs();

    match addresses {
        Ok(mut addr_iter) => {
            // Attempt to bind to the first resolved address
            if let Some(socket_addr) = addr_iter.next() {
                match TcpListener::bind(socket_addr) {
                    Ok(listener) => {
                        println!("Server listening on {}", socket_addr);
                        Some(listener)
                    }
                    Err(e) => {
                        eprintln!("Error: {e}. Unable to listen to: {}", socket_addr);
                        None
                    }
                }
            } else {
                eprintln!("Error: No addresses resolved for {}:{}", host, port);
                None
            }
        }
        Err(e) => {
            eprintln!("Error resolving address {}:{}. {e}", host, port);
            None
        }
    }
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
            default_error_path: None,
            body_size_limit: 0,
            routes: vec![],
        };
        assert!(get_servers(vec![server_config]).is_empty());
    }
}
