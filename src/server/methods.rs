use super::{Method, Route};
use crate::server::content_type;
use crate::server::utils::{get_line, get_split_index};
use crate::server_config::ServerConfig;
use crate::type_aliases::Bytes;
use http::header::{ALLOW, CONTENT_LENGTH, CONTENT_TYPE, HOST};
use http::method::InvalidMethod;
use http::{Request, Response, StatusCode};
use std::fs;
use std::str::FromStr;

pub fn get_method(req: &str) -> Result<Method, InvalidMethod> {
    let line = get_line(req, 0);
    let method = get_split_index(line, 0);
    // "GET /path2 HTTP/1.1" -> "GET"
    Method::from_str(method)
}

pub fn method_is_allowed(method: &Method, route: &Route) -> bool {
    route.accepted_http_methods.contains(method)
}

pub fn handle_method(
    route: &Route,
    req: &Request<String>,
    config: &ServerConfig,
) -> Result<Response<Bytes>, StatusCode> {
    let resp = match *req.method() {
        Method::GET => get(req, config)?,
        Method::OPTIONS => options(route, req, config)?,
        // Method::HEAD => head(req, resp),
        Method::POST => post(req)?,
        Method::PUT => put(req)?,
        Method::PATCH => patch(req)?,
        Method::DELETE => delete(req)?,
        _ => get(req, config).unwrap_or_default(),
    };
    Ok(resp)
}
pub fn get(req: &Request<String>, config: &ServerConfig) -> Result<Response<Bytes>, StatusCode> {
    let path = &req.uri().to_string();
    let body = match fs::read(format!("src{path}")) {
        Ok(bytes) => bytes,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let resp = Response::builder()
        .version(req.version())
        .header(HOST, config.host)
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, content_type(path))
        .body(body)
        .unwrap();

    Ok(resp)
}

pub fn options(
    route: &Route,
    req: &Request<String>,
    config: &ServerConfig,
) -> Result<Response<Bytes>, StatusCode> {
    let allowed_methods = route
        .accepted_http_methods
        .iter()
        .map(|method| method.as_str())
        .collect::<Vec<&str>>()
        .join(", ");

    let resp = Response::builder()
        .version(req.version())
        .header(HOST, config.host)
        .status(StatusCode::OK)
        .header(ALLOW, allowed_methods)
        .body(vec![]) // Empty body for OPTIONS
        .unwrap();

    Ok(resp)
}

pub fn post(req: &Request<String>) -> Result<Response<Bytes>, StatusCode> {
    let path = &format!("src{}", req.uri().path());
    let body = req.body().as_bytes().to_vec();

    let resp = Response::builder()
        .status(StatusCode::OK)
        .version(req.version())
        .header(CONTENT_TYPE, content_type(path))
        .header(CONTENT_LENGTH, body.len())
        .body(body.clone())
        .unwrap();

    // Resource does not exist, so create it.
    if fs::metadata(path).is_err() {
        return match fs::write(path, body) {
            Ok(_) => Ok(resp),
            Err(_) => Err(StatusCode::BAD_REQUEST),
        };
    }

    let mut path = String::from(path);
    let end = path.rfind('.').unwrap_or(path.len());
    // If the file already exists, modify the path.
    // /foo.txt -> /foo(1).txt
    let mut i = 0;
    path.insert_str(end, &format!("({i})"));

    while fs::metadata(&path).is_ok() {
        path = path.replace(&format!("({i})"), &format!("({})", i + 1));
        i += 1;
    }

    match fs::write(&path, body) {
        Ok(_) => Ok(resp),
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

pub fn put(req: &Request<String>) -> Result<Response<Bytes>, StatusCode> {
    let path = &format!("src{}", req.uri().path());
    let bytes = req.body().as_bytes().to_vec();
    let resp = Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, content_type(path))
        .header(CONTENT_LENGTH, bytes.len())
        .body(bytes.clone())
        .unwrap();

    match fs::write(path, bytes) {
        Ok(_) => Ok(resp),
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

pub fn patch(req: &Request<String>) -> Result<Response<Bytes>, StatusCode> {
    let path = &format!("src{}", req.uri().path());
    let bytes = req.body().as_bytes().to_vec();
    let resp = Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, content_type(path))
        .header(CONTENT_LENGTH, bytes.len())
        .body(bytes.clone())
        .unwrap();

    match fs::metadata(path) {
        Ok(_) => match fs::write(path, bytes) {
            Ok(_) => Ok(resp),
            Err(_) => Err(StatusCode::BAD_REQUEST),
        },
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

pub fn delete(req: &Request<String>) -> Result<Response<Bytes>, StatusCode> {
    let path = &format!("src{}", req.uri().path());
    let body = match fs::read(format!("src{path}")) {
        Ok(bytes) => bytes,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    let resp = Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, content_type(path))
        .header(CONTENT_LENGTH, body.len())
        .body(body)
        .unwrap();

    match fs::remove_file(path) {
        Ok(_) => Ok(resp),
        Err(_) => match fs::remove_dir_all(path) {
            Ok(_) => Ok(resp),
            Err(_) => Err(StatusCode::BAD_REQUEST),
        },
    }
}
