use crate::server::get_cookie;
use crate::server_config::ServerConfig;
use crate::type_aliases::Bytes;
use http::header::HOST;
use std::path::Path;

use http::{Request, Response, StatusCode};
pub mod sessions;
pub use sessions::*;

pub mod delete;
pub use delete::*;

pub mod upload;
use crate::server_config::route::Route;
pub use upload::*;

/// # get_target_location
///
/// Get the path to the target location. Returns an error if it cannot get the header or parse
/// the path
pub fn get_target_location(req: &Request<String>) -> Result<&Path, StatusCode> {
    let value = match req.headers().get(http::header::CONTENT_LOCATION) {
        Some(value) => value,
        None => return Err(StatusCode::BAD_REQUEST),
    };

    match value.to_str() {
        Ok(path) => Ok(Path::new(path)),
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

/// # route_From_path
///
/// Get the desired route from the server config based on the path. If for some reason it was
/// used without checking for the route, then it returns `None`
#[allow(dead_code)]
pub fn route_from_path<'a>(
    path: crate::type_aliases::Path,
    conf: &'a ServerConfig,
) -> Option<&'a Route<'a>> {
    conf.routes.iter().find(|route| route.path.eq(path))
}
