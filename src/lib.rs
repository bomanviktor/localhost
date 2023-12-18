pub mod type_aliases {
    pub type Port = u16;
    pub type Bytes = Vec<u8>;
    pub type Path<'a> = &'a str;
    pub type FileExtension<'a> = &'a str;
}
pub mod server_config {
    pub mod config;
    use crate::server::Port;
    use crate::server_config::route::Route;
    use crate::type_aliases::Path;
    pub use config::*;
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
    pub mod cgi;
    pub use cgi::*;
    pub mod handle;
    pub use handle::*;

    pub mod requests;
    pub use requests::*;

    pub mod routes;
    pub use routes::*;

    pub mod start;
    pub use start::*;

    pub mod responses;
    use crate::server_config::ServerConfig;
    pub use crate::type_aliases::Port;
    pub use responses::*;
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
