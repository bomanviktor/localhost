pub mod server_config {
    pub mod config;
    pub use config::*;

    use crate::server_config::route::Route;
    use crate::type_aliases::{Path, Port};
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
        use crate::type_aliases::Port;
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
        use crate::server::Cgi;
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
    }
}
pub mod type_aliases {
    pub type Port = u16;
    pub type Bytes = Vec<u8>;
    pub type Path<'a> = &'a str;
    pub type FileExtension<'a> = &'a str;
}

pub mod server {
    pub mod handle;

    pub use handle::*;
    use mio::net::TcpListener;
    use std::sync::Arc;

    pub mod requests;
    pub use requests::*;

    pub mod responses;
    pub use responses::*;

    pub mod cgi;
    pub use cgi::*;

    pub mod routes;
    pub use routes::*;
    pub mod start;
    use crate::server_config::ServerConfig;
    pub use start::*;

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

    #[derive(Debug)]
    struct Client<'a> {
        id: usize,
        config: Arc<ServerConfig<'a>>,
    }
}
