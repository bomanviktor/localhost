// Type aliases module
pub mod type_aliases {
    pub type Port = u16;
    pub type Bytes = Vec<u8>;
    pub type Path = String;
    pub type FileExtension = String;
}

// Server configuration module
pub mod server_config {
    pub mod config;
    use std::fs;

    use crate::server_config::route::Route;
    use crate::type_aliases::Port;
    pub use config::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct AppConfig {
        pub servers: Vec<ServerConfig>,
    }

    #[derive(Clone, Debug, Deserialize)]
    pub struct ServerConfig {
        pub host: String,
        pub ports: Vec<Port>,
        pub default_error_path: Option<String>,
        pub body_size_limit: usize,
        pub routes: Vec<Route>,
    }

    pub mod route {

        use serde::Deserialize;
        use std::collections::HashMap;

        #[derive(Clone, Debug, Deserialize)]
        pub struct Route {
            pub url_path: String,
            pub methods: Vec<String>,
            pub handler: Option<String>, // Updated to use String
            pub settings: Option<Settings>,
        }

        #[derive(Clone, Debug, Deserialize)]
        pub struct Settings {
            pub http_redirections: Option<Vec<String>>,
            pub redirect_status_code: Option<String>,
            pub root_path: Option<String>,
            pub default_if_url_is_dir: Option<String>,
            pub default_if_request_is_dir: Option<String>,
            pub cgi_def: Option<HashMap<String, String>>, // Updated to use String
            pub list_directory: bool,
        }
    }

    // Load configuration from TOML file
    pub fn load_config() -> Vec<ServerConfig> {
        let config_str = fs::read_to_string("config.toml").expect("Failed to read config.toml");
        let app_config: AppConfig =
            toml::from_str(&config_str).expect("Failed to parse config.toml");
        app_config.servers
    }
}

pub mod log {
    pub mod logging;
    pub use logging::*;
}

pub mod server {
    pub mod handle;
    pub use handle::*;

    use crate::server_config::route::Route;
    use crate::type_aliases::Bytes;
    use http::{Method, Request, Response, StatusCode};
    use std::io;
    use std::io::Read;

    use crate::server_config::ServerConfig;
    use mio::net::{TcpListener, TcpStream};
    use mio::{Events, Interest, Poll, Token};
    use std::collections::HashMap;
    use std::net::SocketAddr;

    use std::sync::Arc;
    pub mod requests;
    pub use requests::*;
    pub mod responses;
    pub use responses::*;
    pub mod methods;
    pub use methods::*;
    pub mod cgi;
    pub use cgi::*;
    pub mod routes;
    pub use routes::*;
    pub mod start;
    pub use start::*;

    pub mod sessions;
    pub use sessions::*;

    mod state;
    pub use state::*;

    #[derive(Debug)]
    pub struct Server {
        pub listeners: Vec<TcpListener>,
        pub config: ServerConfig,
    }

    impl Server {
        pub fn new(listeners: Vec<TcpListener>, config: ServerConfig) -> Self {
            Self { listeners, config }
        }
    }

    #[derive(Debug)]
    pub struct Listener {
        pub listener: TcpListener,
        pub token: Token,
        pub config: Arc<ServerConfig>,
    }

    impl Listener {
        pub fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
            self.listener.accept()
        }
    }
}
