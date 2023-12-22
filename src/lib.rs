pub mod handlers;
pub mod server;

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
        pub default_error_path: Option<Path<'a>>,
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
            fn(req: &Request<String>, conf: &ServerConfig) -> Result<Response<Bytes>, StatusCode>;

        #[derive(Clone, Debug)]
        pub struct Route<'a> {
            pub path: Path<'a>,
            pub methods: Vec<Method>,
            pub handler: Option<HandlerFunc>,
            pub settings: Option<Settings<'a>>,
        }

        #[derive(Clone, Debug)]
        pub struct Settings<'a> {
            pub http_redirections: Vec<Path<'a>>, // From endpoint, to path
            pub redirect_status_code: StatusCode,
            pub root_path: Option<Path<'a>>,
            pub default_if_url_is_dir: Path<'a>, // TODO: Implement
            pub default_if_request_is_dir: Path<'a>, // TODO: Implement
            pub cgi_def: HashMap<FileExtension<'a>, Cgi>,
            pub list_directory: bool, // TODO: Implement
        }

        impl Settings<'_> {
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
