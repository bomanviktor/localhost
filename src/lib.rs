pub mod server_config {
    pub mod config;
    pub use config::*;

    use crate::server::Port;
    use crate::server_config::route::Route;
    use std::collections::HashMap;

    #[derive(Clone, Debug)]
    pub struct ServerConfig<'a> {
        pub host: &'a str,
        pub ports: Vec<Port>,
        pub default_error_paths: HashMap<http::StatusCode, &'a str>,
        pub body_size_limit: usize,
        pub routes: Vec<Route<'a>>,
    }

    pub mod builder {
        use super::*;
        #[derive(Debug)]
        pub struct ConfigBuilder<'a> {
            pub host: Option<&'a str>,
            pub ports: Option<Vec<Port>>,
            pub default_error_paths: Option<HashMap<http::StatusCode, &'a str>>,
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
        use std::collections::HashMap;

        #[derive(Clone, Debug)]
        pub struct Route<'a> {
            pub paths: Vec<&'a str>,
            pub accepted_http_methods: Vec<http::Method>,
            pub http_redirections: HashMap<&'a str, &'a str>, // From endpoint, to endpoint
            pub redirect_status_code: http::StatusCode,       // TODO: Implement
            pub default_if_url_is_dir: &'a str,               // TODO: Implement
            pub default_if_request_is_dir: &'a str,           // TODO: Implement
            pub cgi_def: HashMap<&'a str, Cgi>,
            pub list_directory: bool, // TODO: Implement
            pub length_required: bool,
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
    use std::io::ErrorKind;
    use std::net::TcpListener;

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

    // Refactor this to its own module.
    pub fn start(mut servers: Vec<Server>) {
        loop {
            for server in &mut servers {
                for listener in &mut server.listeners {
                    match listener.accept() {
                        Ok((stream, _addr)) => handle_client(stream, &server.config),
                        Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                            // No incoming connections, continue to the next listener
                            continue;
                        }
                        Err(e) => eprintln!("Error accepting connection: {}", e),
                    }
                }
            }
            // Sleep for a short duration to avoid busy waiting
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}
