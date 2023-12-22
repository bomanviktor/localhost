pub mod handle;
pub use handle::*;

use crate::server_config::route::Route;
use crate::type_aliases::Bytes;
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
