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

    #[derive(Clone, Debug)]
    pub struct ServerConfig<'a> {
        pub host: &'a str,
        pub ports: Vec<Port>,
        pub custom_error_path: Option<Path<'a>>,
        pub body_size_limit: usize,
        pub routes: Vec<Route<'a>>,
    }

    pub mod route {
        use crate::server::Cgi;
        use crate::server_config::ServerConfig;
        use crate::type_aliases::{Bytes, FileExtension, Path};
        use http::{Method, Request, Response, StatusCode};
        use std::collections::HashMap;

        pub type HandlerFunc =
            fn(req: &Request<Bytes>, conf: &ServerConfig) -> Result<Response<Bytes>, StatusCode>;

        #[derive(Clone, Debug)]
        pub struct Route<'a> {
            pub url_path: Path<'a>,
            pub methods: Vec<Method>,
            pub handler: Option<HandlerFunc>,
            pub settings: Option<Settings<'a>>,
        }

        #[derive(Clone, Debug)]
        pub struct Settings<'a> {
            pub http_redirections: Option<Vec<Path<'a>>>, // From endpoint, to path
            pub redirect_status_code: Option<StatusCode>,
            pub root_path: Option<Path<'a>>,
            pub default_if_url_is_dir: Option<Path<'a>>, // TODO: Implement
            pub default_if_request_is_dir: Option<Path<'a>>, // TODO: Implement
            pub cgi_def: Option<HashMap<FileExtension<'a>, Cgi>>,
            pub list_directory: bool,
        }
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
    pub struct Listener<'a> {
        pub listener: TcpListener,
        pub token: Token,
        pub config: Arc<ServerConfig<'a>>,
    }

    impl Listener<'_> {
        pub fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
            self.listener.accept()
        }
    }
}
