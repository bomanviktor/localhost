pub mod server_config {
    pub mod config;
    pub use config::*;

    use crate::server::Port;
    use crate::server_config::route::Route;
    use crate::type_aliases::Path;
    use std::collections::HashMap;

    #[derive(Clone, Debug)]
    pub struct ServerConfig<'a> {
        pub host: &'a str,
        pub ports: Vec<Port>,
        pub default_error_paths: HashMap<http::StatusCode, Path<'a>>,
        pub body_size_limit: usize,
        pub routes: Vec<Route<'a>>,
    }

    pub mod builder {
        use super::*;
        #[derive(Debug)]
        pub struct ConfigBuilder<'a> {
            pub host: Option<&'a str>,
            pub ports: Option<Vec<Port>>,
            pub default_error_paths: Option<HashMap<http::StatusCode, Path<'a>>>,
            pub body_size_limit: Option<usize>,
            pub routes: Option<Vec<Route<'a>>>,
        }

        impl Default for ConfigBuilder<'_> {
            fn default() -> Self {
                Self::new()
            }
        }
        impl<'a> ConfigBuilder<'a> {
            pub fn new() -> ConfigBuilder<'a> {
                Self {
                    host: None,
                    ports: None,
                    default_error_paths: None,
                    body_size_limit: None,
                    routes: None,
                }
            }

            pub fn host(&mut self, host_addr: &'a str) -> &mut Self {
                self.host = Some(host_addr);
                self
            }

            pub fn ports(&mut self, ports: Vec<Port>) -> &mut Self {
                self.ports = Some(ports);
                self
            }

            pub fn default_error_paths(
                &mut self,
                paths: HashMap<http::StatusCode, &'a str>,
            ) -> &mut Self {
                self.default_error_paths = Some(paths);
                self
            }

            pub fn body_size_limit(&mut self, limit: usize) -> &mut Self {
                self.body_size_limit = Some(limit);
                self
            }

            pub fn routes(&mut self, routes: Vec<Route<'a>>) -> &mut Self {
                self.routes = Some(routes);
                self
            }

            pub fn build(&self) -> ServerConfig<'a> {
                ServerConfig {
                    host: self.host.expect("Invalid host"),
                    ports: self.ports.clone().expect("Invalid ports"),
                    default_error_paths: self.default_error_paths.clone().expect("Invalid paths"),
                    body_size_limit: self.body_size_limit.expect("Invalid size limit"),
                    routes: self.routes.clone().expect("Invalid routes"),
                }
            }
        }
    }

    pub mod route {
        use crate::server_config::route::cgi::Cgi;
        use crate::type_aliases::{FileExtension, Path};
        use std::collections::HashMap;

        #[derive(Clone, Debug)]
        pub struct Route<'a> {
            pub paths: Vec<Path<'a>>,
            pub accepted_http_methods: Vec<http::Method>,
            pub http_redirections: HashMap<Path<'a>, Path<'a>>, // From endpoint, to endpoint
            pub redirect_status_code: http::StatusCode,
            pub root_path: Option<Path<'a>>,
            pub default_if_url_is_dir: Path<'a>, // TODO: Implement
            pub default_if_request_is_dir: Path<'a>, // TODO: Implement
            pub cgi_def: HashMap<FileExtension<'a>, Cgi>,
            pub list_directory: bool, // TODO: Implement
            pub length_required: bool,
        }

        impl Route<'_> {
            pub fn format_path(&self, path: Path) -> String {
                if self.root_path.is_some() {
                    format!("{}{}", self.root_path.unwrap(), path)
                } else {
                    format!("src{}", path)
                }
            }
        }

        pub mod cgi {
            #[derive(Clone, Debug)]
            pub enum Cgi {
                Python,
                PHP,
                JavaScript,
                Cpp,
            }
        }
    }
}
pub mod type_aliases {
    pub type Port = u16;
    pub type Bytes = Vec<u8>;
    pub type Path<'a> = &'a str;
    pub type FileExtension<'a> = &'a str;
}

pub mod client {
    pub mod handle;
    pub use handle::*;

    pub mod requests;
    pub use requests::*;

    pub mod responses;
    pub use responses::*;
}
pub mod server {
    use crate::client::handle_client;
    use crate::server_config::config::server_config;
    use crate::server_config::ServerConfig;
    pub use crate::type_aliases::Port;

    use mio::net::TcpListener;
    use mio::{Events, Interest, Poll, Token};
    use std::collections::HashMap;
    use std::io::ErrorKind;

    use std::net::SocketAddr;
    use std::sync::Arc;

    #[derive(Debug)]
    pub struct Server<'a> {
        pub listeners: Vec<TcpListener>,
        pub config: ServerConfig<'a>,
    }

    impl<'a> Server<'a> {
        pub fn new(listeners: Vec<TcpListener>, config: ServerConfig<'a>) -> Self {
            Self { listeners, config }
        }
    }

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

    struct ListenerConfig<'a> {
        listener_index: usize,
        config: Arc<ServerConfig<'a>>,
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
}
