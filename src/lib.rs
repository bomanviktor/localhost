pub mod server_config {
    pub mod config;
    use crate::server::Port;
    use crate::server_config::route::Route;
    use std::path::Path;

    #[derive(Clone, Debug)]
    pub struct ServerConfig<'a> {
        pub host: &'a str,
        pub ports: Vec<Port>,
        pub default_error_paths: Vec<&'a Path>,
        pub body_size_limit: u128,
        pub routes: Vec<Route<'a>>,
    }

    pub mod builder {
        use super::*;
        #[derive(Debug)]
        pub struct ConfigBuilder<'a> {
            pub host: Option<&'a str>,
            pub ports: Option<Vec<Port>>,
            pub default_error_paths: Option<Vec<&'a Path>>,
            pub body_size_limit: Option<u128>,
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

            pub fn default_error_paths(&mut self, paths: Vec<&'a Path>) -> &mut Self {
                self.default_error_paths = Some(paths);
                self
            }

            pub fn body_size_limit(&mut self, limit: u128) -> &mut Self {
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
        use crate::type_aliases::Endpoint;
        use std::collections::HashMap;
        use std::path::Path;

        #[derive(Clone, Debug)]
        pub struct Route<'a> {
            pub paths: Vec<&'a Path>,
            pub accepted_http_methods: Vec<http::Method>,
            pub http_redirections: HashMap<Endpoint<'a>, Endpoint<'a>>, // From endpoint, to endpoint
            pub default_if_url_is_dir: &'a Path,
            pub default_if_request_is_dir: &'a Path,
            pub cgi_def: HashMap<&'a str, Cgi>,
            pub list_directory: bool,
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
    pub type Endpoint<'a> = &'a str;
}

pub mod client {
    pub mod handle;
    pub use handle::*;
    pub struct Client {
        pub ip: String,
        // Add all required fields here
    }
}
pub mod server {
    pub use crate::client::Client;
    use crate::server_config::ServerConfig;
    pub use crate::type_aliases::Port;
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
}
