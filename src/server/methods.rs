use super::{Bytes, Method, Request, Response, Route, ServerConfig, StatusCode};
use crate::log;
use crate::log::*;
use crate::server::content_type;
use crate::server::utils::{get_line, get_split_index};
use http::header::{ALLOW, CONTENT_LENGTH, CONTENT_TYPE, HOST};
use std::fs;
use std::str::FromStr;

pub fn get_method(req: &str) -> Result<Method, StatusCode> {
    let line = get_line(req, 0);
    let method = get_split_index(line, 0);
    // "GET /path2 HTTP/1.1" -> "GET"
    match Method::from_str(method) {
        Ok(method) => Ok(method),
        Err(e) => {
            log!(LogFileType::Server, format!("Error: {e}"));
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

pub fn method_is_allowed(method: &Method, route: &Route) -> bool {
    route.methods.contains(method)
}

pub fn handle_method(
    route: &Route,
    req: &Request<String>,
    config: &ServerConfig,
) -> Result<Response<Bytes>, StatusCode> {
    //==================================//
    //  Add middleware here. Example:   //
    //  Verify allowed content types.   //
    //==================================//

    let resp = match *req.method() {
        //==============//
        // SAFE METHODS //
        //==============//
        Method::GET => safe::get(req, config)?,
        Method::OPTIONS => safe::options(route, req, config)?,
        Method::HEAD => safe::head(req, config)?,
        Method::TRACE => safe::trace(req, config)?,
        //================//
        // UNSAFE METHODS //
        //================//
        Method::POST => not_safe::post(req, config)?,
        Method::PUT => not_safe::put(req, config)?,
        Method::PATCH => not_safe::patch(req, config)?,
        Method::DELETE => not_safe::delete(req, config)?,
        _ => {
            return {
                log!(
                    LogFileType::Server,
                    format!("Error: Bad Request {:?}", &req)
                );
                Err(StatusCode::BAD_REQUEST)
            }
        }
    };
    Ok(resp)
}

pub mod safe {
    use super::*;
    use crate::server::get_route;
    use crate::server::path::add_root_to_path;
    use http::header::{TRANSFER_ENCODING, VIA};
    use http::HeaderName;

    /// # STANDARD_HEADERS
    ///
    /// Make sure you adjust this to get the desired behaviour for get requests.
    /// TODO: Move this to a function.
    const STANDARD_HEADERS: [HeaderName; 1] = [TRANSFER_ENCODING];
    pub fn get(
        req: &Request<String>,
        config: &ServerConfig,
    ) -> Result<Response<Bytes>, StatusCode> {
        let route = &get_route(req, config).unwrap();
        let path = &add_root_to_path(route, req.uri().path());
        let body = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(_) => {
                log!(
                    LogFileType::Server,
                    format!("Error: Path does not exist {}", &path)
                );
                return Err(StatusCode::NOT_FOUND);
            }
        };

        let mut resp = Response::builder()
            .version(req.version())
            .header(HOST, config.host)
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, content_type(path))
            .header(CONTENT_LENGTH, body.len());

        for (key, value) in req.headers() {
            if STANDARD_HEADERS.contains(key) {
                resp = resp.header(key, value);
            }
        }

        match resp.body(body) {
            Ok(r) => Ok(r),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }

    pub fn head(
        req: &Request<String>,
        config: &ServerConfig,
    ) -> Result<Response<Bytes>, StatusCode> {
        let route = &get_route(req, config).unwrap();
        let path = &add_root_to_path(route, req.uri().path());
        let metadata = match fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(_) => {
                log!(
                    LogFileType::Server,
                    format!("Error: Could not get head for path '{}'", path)
                );
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        let resp = Response::builder()
            .version(req.version())
            .header(HOST, config.host)
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, content_type(path))
            .header(CONTENT_LENGTH, metadata.len().to_string()) // Set the Content-Length header
            .body(vec![]) // No body for HEAD
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(resp)
    }

    pub fn trace(
        req: &Request<String>,
        config: &ServerConfig,
    ) -> Result<Response<Bytes>, StatusCode> {
        // Check the Max-Forwards header
        let max_forwards = req
            .headers()
            .get("Max-Forwards")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<i32>().ok());

        if max_forwards == Some(0) {
            return Err(StatusCode::BAD_REQUEST);
        }

        // Update the Via header
        let existing_via = req.headers().get(VIA).map(|v| v.to_str().unwrap_or(""));

        let via = if let Some(via_header) = existing_via {
            format!("{}, {}", via_header, config.host)
        } else {
            config.host.to_string()
        };

        // Exclude sensitive headers and construct the request string
        let request_string = format!("{:?}\r\n", req)
            .lines()
            .filter(|line| !line.starts_with("Cookie:") && !line.starts_with("Authorization:"))
            .collect::<Vec<_>>()
            .join("\r\n");

        let resp = Response::builder()
            .version(req.version())
            .header(HOST, config.host)
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "message/http")
            .header("Via", via)
            .body(Bytes::from(request_string))
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(resp)
    }

    pub fn options(
        route: &Route,
        req: &Request<String>,
        config: &ServerConfig,
    ) -> Result<Response<Bytes>, StatusCode> {
        let allowed_methods = route
            .methods
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
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(resp)
    }
}

mod not_safe {
    use super::*;
    use crate::server::get_route;
    use crate::server::path::add_root_to_path;
    pub fn post(
        req: &Request<String>,
        config: &ServerConfig,
    ) -> Result<Response<Bytes>, StatusCode> {
        let route = &get_route(req, config).unwrap();
        let path = &add_root_to_path(route, req.uri().path());
        let body = req.body().as_bytes().to_vec();

        let resp = Response::builder()
            .status(StatusCode::OK)
            .version(req.version())
            .header(CONTENT_TYPE, content_type(path))
            .header(CONTENT_LENGTH, body.len())
            .body(body.clone())
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Resource does not exist, so create it.
        if fs::metadata(path).is_err() {
            return match fs::write(path, body) {
                Ok(_) => Ok(resp),
                Err(e) => {
                    log!(LogFileType::Server, format!("Error: {e}"));
                    Err(StatusCode::BAD_REQUEST)
                }
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
            Err(e) => {
                log!(LogFileType::Server, format!("Error: {e}"));
                Err(StatusCode::BAD_REQUEST)
            }
        }
    }

    pub fn put(
        req: &Request<String>,
        config: &ServerConfig,
    ) -> Result<Response<Bytes>, StatusCode> {
        let route = &get_route(req, config).unwrap();
        let path = &add_root_to_path(route, req.uri().path());
        let bytes = req.body().as_bytes().to_vec();

        let resp = Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, content_type(path))
            .header(CONTENT_LENGTH, bytes.len())
            .body(bytes.clone())
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        match fs::write(path, bytes) {
            Ok(_) => Ok(resp),
            Err(_) => {
                log!(
                    LogFileType::Server,
                    format!("Error: Bad request {:?}", &req)
                );
                Err(StatusCode::BAD_REQUEST)
            }
        }
    }

    pub fn patch(
        req: &Request<String>,
        config: &ServerConfig,
    ) -> Result<Response<Bytes>, StatusCode> {
        let route = &get_route(req, config).unwrap();
        let path = &add_root_to_path(route, req.uri().path());
        let bytes = req.body().as_bytes().to_vec();

        let resp = Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, content_type(path))
            .header(CONTENT_LENGTH, bytes.len())
            .body(bytes.clone())
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        match fs::metadata(path) {
            Ok(_) => match fs::write(path, bytes) {
                Ok(_) => Ok(resp),
                Err(_) => {
                    log!(
                        LogFileType::Server,
                        format!("Error: Bad request {:?}", &req)
                    );
                    Err(StatusCode::BAD_REQUEST)
                }
            },
            Err(_) => {
                log!(
                    LogFileType::Server,
                    format!("Error: Bad request {:?}", &req)
                );
                Err(StatusCode::BAD_REQUEST)
            }
        }
    }

    pub fn delete(
        req: &Request<String>,
        config: &ServerConfig,
    ) -> Result<Response<Bytes>, StatusCode> {
        let route = &get_route(req, config).unwrap();
        let path = &add_root_to_path(route, req.uri().path());
        let body = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(_) => {
                log!(
                    LogFileType::Server,
                    format!("Error: Failed to read body {:?}", &req)
                );
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        let resp = Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, content_type(path))
            .header(CONTENT_LENGTH, body.len())
            .body(body)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        match fs::remove_file(path) {
            Ok(_) => Ok(resp),
            Err(_) => match fs::remove_dir_all(path) {
                Ok(_) => Ok(resp),
                Err(_) => {
                    log!(
                        LogFileType::Server,
                        format!("Error: Bad request {:?}", &req)
                    );
                    Err(StatusCode::BAD_REQUEST)
                }
            },
        }
    }
}
