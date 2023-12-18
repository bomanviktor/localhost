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

    pub fn start(mut servers: Vec<Server>) {
        let mut poll = match Poll::new() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to create Poll instance: {}", e);
                return;
            }
        };
        let mut events = Events::with_capacity(128);
        let mut server_tokens = HashMap::new();
        let listeners = Vec::new();

        let mut token_id = 0; // Initialize a token counter

        let mut listener_tokens = HashMap::new();
        for server in servers.iter_mut() {
            let config = Arc::new(server.config.clone()); // Clone the config for shared ownership
            for listener in &mut server.listeners {
                let token = Token(token_id);
                token_id += 1; // Increment the token counter for the next listener

                // Register the listener to the poll instance
                match poll
                    .registry()
                    .register(listener, token, Interest::READABLE)
                {
                    Ok(_) => {
                        server_tokens.insert(token, Arc::clone(&config));
                        listener_tokens.insert(token, listener);
                    }
                    Err(e) => {
                        eprintln!("Error registering listener: {}", e);
                        continue;
                    }
                };
            }
        }

        let mut listener_indices = HashMap::new();
        for (index, listener) in listeners.iter().enumerate() {
            let token = listener_tokens
                .keys()
                .find(|&token| std::ptr::eq(listener_tokens[token], listener))
                .unwrap();
            listener_indices.insert(token, index);
        }

        loop {
            poll.poll(&mut events, None).unwrap();

            for event in events.iter() {
                println!("Event: {:?}", event);
                if let Some(config) = server_tokens.get(&event.token()) {
                    // Access the listener directly from listener_tokens map
                    if let Some(listener) = listener_tokens.get_mut(&event.token()) {
                        match listener.accept() {
                            Ok((stream, _)) => handle_client(stream, config),
                            Err(ref e) if e.kind() == ErrorKind::WouldBlock => (),
                            Err(e) => eprintln!("Error: {}", e),
                        }
                    }
                }
            }
        }
    }
}
