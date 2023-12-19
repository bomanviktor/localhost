pub mod type_aliases {
    pub type Port = u16;
    pub type Bytes = Vec<u8>;
    pub type Path<'a> = &'a str;
    pub type FileExtension<'a> = &'a str;
}
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

pub mod server {
    pub mod handle;
    pub use handle::*;

    use crate::server_config::route::Route;
    use crate::type_aliases::Bytes;
    use http::header::{CONTENT_LENGTH, CONTENT_TYPE, HOST};
    use http::{Method, Request, Response, StatusCode};
    use std::io;
    use std::io::{Read, Write};

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
    pub mod cgi;

    pub use cgi::*;
    pub mod routes;
    pub use routes::*;
    pub mod start;
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
    pub struct Client<'a> {
        pub id: usize,
        pub config: Arc<ServerConfig<'a>>,
    }

    mod state {
        use super::*;
        pub struct ServerState<'a> {
            pub poll: Poll,
            pub events: Events,
            pub token_id: usize,
            pub all_listeners: Vec<TcpListener>,
            pub clients: Vec<Client<'a>>,
            pub connections: HashMap<Token, (TcpStream, Arc<ServerConfig<'a>>)>,
            pub to_close: Vec<Token>,
        }

        pub fn initialize_server_state(servers: Vec<Server<'static>>) -> ServerState<'static> {
            let poll = Poll::new().expect("Failed to create Poll instance");
            let events = Events::with_capacity(1024);
            let mut token_id = INITIAL_TOKEN_ID;
            let mut all_listeners = Vec::new();
            let mut clients = Vec::new();
            let connections = HashMap::new();
            let to_close = Vec::new();

            // Register all the listeners
            for server in servers {
                let config = Arc::new(server.config);

                server.listeners.into_iter().for_each(|listener| {
                    let id = all_listeners.len();
                    all_listeners.push(listener);
                    let token = Token(token_id);
                    token_id += 1;

                    poll.registry()
                        .register(&mut all_listeners[id], token, Interest::READABLE)
                        .expect("Failed to register listener");

                    clients.push(Client {
                        id,
                        config: Arc::clone(&config),
                    });
                });
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
    }
}
