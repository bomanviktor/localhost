use crate::log;
use crate::log::*;
use crate::server::state::*;
use crate::server::{Server, SocketAddr, TcpListener};
use crate::server_config::server_config;

pub fn start(servers: Vec<Server<'static>>) {
    let mut s = ServerState::init(servers);
    loop {
        s.poll();
        s.handle_events();
    }
}

pub fn servers() -> Vec<Server<'static>> {
    let mut servers = Vec::new();

    for config in server_config() {
        let mut listeners = Vec::new();
        for port in &config.ports {
            listeners.push(bind_port(port, config.host));
        }
        // Make a server and push it to the servers vector
        servers.push(Server::new(listeners, config))
    }
    servers
}
fn bind_port(port: &crate::type_aliases::Port, host: &str) -> TcpListener {
    let address = format!("{}:{}", host, port);
    match TcpListener::bind(address.parse::<SocketAddr>().unwrap()) {
        Ok(listener) => {
            println!("Server listening on {}", address);
            log!(
                LogFileType::Server,
                format!("Server listening on {}", address)
            );

            listener
        }
        Err(e) => {
            eprintln!("Error: {}. Unable to listen to: {}", e, address);
            log!(
                LogFileType::Server,
                format!("Error: {}. Unable to listen to: {}", e, address)
            );
            std::process::exit(1)
        }
    }
}
