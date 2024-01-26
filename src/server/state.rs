use super::{
    Arc, Events, HashMap, Interest, Listener, Poll, Server, ServerConfig, TcpStream, Token,
};

use crate::log::*;
use std::io::ErrorKind;
#[cfg(unix)]
use std::os::fd::{AsRawFd, FromRawFd};
#[cfg(windows)]
use std::os::windows::io::{AsRawSocket, FromRawSocket};
use std::time::Duration;
use std::time::Instant;

pub const INITIAL_TOKEN_ID: usize = 0;

struct Connection<'a> {
    stream: TcpStream,
    config: Arc<ServerConfig<'a>>,
    last_activity: Instant,
}

impl<'a> Connection<'a> {
    fn new(stream: TcpStream, config: Arc<ServerConfig<'a>>) -> Self {
        Self {
            stream,
            config,
            last_activity: Instant::now(),
        }
    }
}
pub struct ServerState<'a> {
    poll: Poll,
    events: Events,
    token_id: usize,
    listeners: Vec<Listener<'a>>,
    connections: HashMap<Token, Connection<'a>>,
}
impl ServerState<'_> {
    pub fn init(servers: Vec<Server<'static>>) -> ServerState<'static> {
        let poll = Poll::new().expect("Failed to create Poll instance");
        let events = Events::with_capacity(4096);
        let mut token_id = INITIAL_TOKEN_ID;
        let mut listeners = Vec::new();
        let connections = HashMap::new();

        // Register all the listeners
        for server in servers {
            let config = Arc::new(server.config);

            server
                .listeners
                .into_iter()
                .enumerate()
                .for_each(|(id, mut listener)| {
                    token_id += 1;
                    let token = Token(id);
                    poll.registry()
                        .register(&mut listener, token, Interest::READABLE)
                        .expect("Failed to register listener");

                    listeners.push(Listener {
                        listener,
                        token,
                        config: Arc::clone(&config),
                    });
                });
        }

        ServerState {
            poll,
            events,
            token_id,
            listeners,
            connections,
        }
    }

    pub fn poll(&mut self) {
        self.poll
            .poll(&mut self.events, Some(Duration::from_millis(5000)))
            .expect("Poll failed");

        self.handle_timeout();
    }

    pub fn handle_events(&mut self) {
        for event in self.events.iter() {
            for listener in &self.listeners {
                while accept_connection(
                    &self.poll,
                    &mut self.token_id,
                    listener,
                    &mut self.connections,
                ) {}
            }
            handle_existing_connection(&self.poll, event.token(), &mut self.connections);
        }
    }

    fn handle_timeout(&mut self) {
        let now = Instant::now();
        let timeout_duration = Duration::from_millis(1000);

        // Remove connections that timed out from `connections` HashMap
        self.connections.retain(|_, conn| {
            if now.duration_since(conn.last_activity) > timeout_duration {
                self.poll
                    .registry()
                    .deregister(&mut conn.stream)
                    .expect("Failed to deregister stream due to timeout");
                false
            } else {
                true
            }
        });
    }
}

fn accept_connection<'a>(
    poll: &Poll,
    token_id: &mut usize,
    listener: &Listener<'a>,
    connections: &mut HashMap<Token, Connection<'a>>,
) -> bool {
    match listener.accept() {
        Ok((mut stream, _)) => {
            let linger_duration = match std::env::consts::OS {
                "macos" => Some(Duration::from_millis(100)),
                _ => None,
            };

            set_linger_option(&stream, linger_duration).expect("Failed to set linger option");

            if let Err(e) = stream.set_ttl(60) {
                log!(LogFileType::Server, format!("Error: {e}"));
            }

            let connection_token = Token(*token_id);
            *token_id += 1;

            poll.registry()
                .register(&mut stream, connection_token, Interest::READABLE)
                .expect("Failed to register new connection");

            connections.insert(
                connection_token,
                Connection::new(stream, Arc::clone(&listener.config)),
            );

            true
        }
        _ => false,
    }
}

fn handle_existing_connection(
    poll: &Poll,
    token: Token,
    connections: &mut HashMap<Token, Connection>,
) {
    let connection = match connections.get_mut(&token) {
        Some(connection) => connection,
        None => return,
    };

    if let Err(e) = crate::server::handle_connection(&mut connection.stream, &connection.config) {
        match e.kind() {
            ErrorKind::WouldBlock => {
                return; // Therefore, we keep the connection registered and return
            }
            _ => log!(LogFileType::Client, format!("Error handling client: {e}")),
        }
    }

    poll.registry()
        .deregister(&mut connection.stream)
        .expect("Failed to deregister stream");
    connections.remove(&token);
}

use crate::log;
use socket2::{SockRef, Socket};

// Function to set linger option on a mio TcpStream
fn set_linger_option(stream: &TcpStream, linger_duration: Option<Duration>) -> std::io::Result<()> {
    #[cfg(unix)]
    let socket = unsafe { Socket::from_raw_fd(stream.as_raw_fd()) };
    #[cfg(windows)]
    let socket = unsafe { Socket::from_raw_socket(stream.as_raw_socket()) };

    SockRef::from(&socket).set_linger(linger_duration)?;
    std::mem::forget(socket); // Important to avoid closing the file descriptor
    Ok(())
}
