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

    pub mod route {
        use crate::cgi::Cgi;
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

pub mod cgi {
    #[derive(Clone, Debug)]
    pub enum Cgi {
        Ada,
        C,
        CSharp,
        Cpp,
        D,
        Erlang,
        Fortran,
        Go,
        Groovy,
        Haskell,
        Java,
        JavaScript,
        Julia,
        Kotlin,
        Lua,
        Nim,
        ObjectiveC,
        OCaml,
        Pascal,
        Perl,
        PHP,
        Python,
        R,
        Ruby,
        Rust,
        Scala,
        Shell,
        Swift,
        TypeScript,
        Zig,
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

    pub fn start(mut servers: Vec<Server>) {
        loop {
            for server in &mut servers {
                for listener in &mut server.listeners {
                    match listener.accept() {
                        Ok((mut stream, _addr)) => handle_client(&mut stream, &server.config),
                        Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                            // No incoming connections, continue to the next listener
                            continue;
                        }
                        Err(e) => eprintln!("Error accepting connection: {}", e),
                    }
                }
            }
            // Sleep for a short duration to avoid busy waiting
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
}
