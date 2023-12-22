use std::path::Path;
use crate::server::get_cookie;
use crate::server_config::ServerConfig;
use crate::type_aliases::Bytes;
use http::header::HOST;

use http::{Request, Response, StatusCode};
pub mod sessions;
pub use sessions::*;

pub mod delete;
pub use delete::*;

pub fn get_path(req: &Request<String>) -> Result<&Path, StatusCode> {
    let value = match req.headers().get(http::header::CONTENT_LOCATION) {
        Some(value) => value,
        None => return Err(StatusCode::BAD_REQUEST),
    };

    match value.to_str() {
        Ok(path) => Ok(Path::new(path)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
