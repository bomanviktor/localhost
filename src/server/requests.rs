use crate::server::body::get_body;
use crate::server::headers::{format_header, get_headers};
use crate::server::method::get_method;
use crate::server::path::get_path;
use crate::server::version::get_version;
use crate::server_config::route::Route;
use crate::server_config::ServerConfig;
use http::{Method, Request, StatusCode};
use std::str::FromStr;

pub fn get_request(conf: &ServerConfig, req_str: &str) -> Result<Request<String>, StatusCode> {
    let version = match get_version(req_str) {
        Ok(v) => v,
        Err(_) => return Err(StatusCode::HTTP_VERSION_NOT_SUPPORTED),
    };

    let path = get_path(req_str);
    let method = match get_method(req_str) {
        Ok(method) => method,
        Err(_) => return Err(StatusCode::METHOD_NOT_ALLOWED),
    };
    let mut request = Request::builder().method(method).uri(path).version(version);

    for header in get_headers(req_str) {
        if let Some((key, value)) = format_header(header) {
            request = request.header(key, value);
        }
    }

    let body = get_body(req_str, conf.body_size_limit).unwrap_or("".to_string());
    Ok(request.body(body).unwrap())
}

pub mod method {
    use super::{FromStr, Method, Route};
    use crate::server::utils::{get_line, get_split_index};
    use crate::type_aliases::{Bytes, Path};
    use http::method::InvalidMethod;
    use std::fs;

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
        path: Path,
        method: &Method,
        body: Option<Bytes>,
    ) -> Option<Bytes> {
        let path = &route.format_path(path);

        match *method {
            Method::GET => Some(get(path).unwrap_or_default()),
            Method::HEAD => None,
            Method::POST => {
                post(path, body.unwrap()).unwrap_or_default();
                None
            }
            Method::PUT => {
                put(path, body.unwrap()).unwrap_or_default();
                None
            }
            Method::PATCH => {
                patch(path, body.unwrap()).unwrap_or_default();
                None
            }
            Method::DELETE => {
                delete(path).unwrap_or_default();
                None
            }
            _ => Some(get(path).unwrap()),
        }
    }

    pub fn get(path: &str) -> std::io::Result<Bytes> {
        fs::read(path)
    }

    pub fn post(path: &str, bytes: Bytes) -> std::io::Result<()> {
        // Resource does not exist, so create it.
        if fs::metadata(path).is_err() {
            return fs::write(path, bytes);
        }

        let mut path = String::from(path); // Turn the path into String
        let end = path.rfind('.').unwrap_or(path.len());
        let mut i = 1;

        // If the file already exists, modify the path.
        // /foo.txt -> /foo(1).txt
        while fs::metadata(&path).is_ok() {
            path.truncate(end);
            path.push_str(&format!("({})", i));
            path.push_str(&path.clone()[end..]);
            i += 1;
        }

        fs::write(&path, bytes)
    }

    pub fn put(path: &str, bytes: Bytes) -> std::io::Result<()> {
        fs::write(path, bytes)
    }

    pub fn patch(path: &str, bytes: Bytes) -> std::io::Result<()> {
        match fs::metadata(path) {
            Ok(_) => fs::write(path, bytes),
            Err(e) => Err(e),
        }
    }

    pub fn delete(path: &str) -> std::io::Result<()> {
        match fs::remove_file(path) {
            Ok(_) => Ok(()),                    // Target was a file
            Err(_) => fs::remove_dir_all(path), // Target was a directory
        }
    }
}

pub mod path {
    use super::*;
    use crate::server::utils::{get_line, get_split_index};
    /// `path` gets the path from the `request`
    pub fn get_path(req: &str) -> &str {
        let line = get_line(req, 0);
        get_split_index(line, 1)
    }

    /// `path_exists` gets the `path` and the `index` of the `route` it was a part of if found.
    pub fn path_exists<'a>(
        requested_path: &'a str,
        routes: &[Route<'a>],
    ) -> Option<(usize, &'a str)> {
        for (i, route) in routes.iter().enumerate() {
            for &path in &route.paths {
                if path == requested_path {
                    return Some((i, path)); // Path is contained in the route
                }
                if route.http_redirections.contains_key(requested_path) {
                    // Path is a part of the redirections
                    let redirected_path = route.http_redirections.get(requested_path).unwrap();
                    return path_exists(redirected_path, routes); // Recursively call itself to check for the redirected path
                }
            }
        }
        None // Path does not exist in allowed paths or in redirections
    }
}

pub mod version {
    use http::{StatusCode, Version};

    pub fn get_version(req: &str) -> Result<Version, StatusCode> {
        let version_str = req
            .split_whitespace()
            .find(|s| s.contains("HTTP/"))
            .unwrap_or("HTTP/1.1");

        match version_str {
            "HTTP/0.9" => Ok(Version::HTTP_09),
            "HTTP/1.0" => Ok(Version::HTTP_10),
            "HTTP/1.1" => Ok(Version::HTTP_11),
            "HTTP/2.0" => Ok(Version::HTTP_2),
            "HTTP/3.0" => Ok(Version::HTTP_3),
            _ => Err(StatusCode::HTTP_VERSION_NOT_SUPPORTED),
        }
    }
}

pub mod headers {
    use crate::server::utils::get_split_index;

    pub fn get_headers(req: &str) -> Vec<&str> {
        // Remove the body from the request
        let head = req
            .trim_end_matches('\n')
            .trim_end()
            .split("\r\n\r\n")
            .collect::<Vec<&str>>()[0];

        head.trim_end()
            .split("\r\n")
            .filter(|line| !line.contains("HTTP/"))
            .collect::<Vec<&str>>()
    }

    pub fn format_header(header: &str) -> Option<(&str, &str)> {
        let key_value = header
            .trim_end_matches('\0')
            .trim_end()
            .split(": ")
            .collect::<Vec<&str>>();

        if key_value.len() == 2 {
            Some((key_value[0], key_value[1]))
        } else {
            None
        }
    }

    /// `set_cookies` takes care of the `Set-Cookie` header
    pub fn set_cookies(req: &str) -> Option<Vec<&str>> {
        let cookies = req
            .split("\r\n")
            .filter(|l| l.contains("Set-Cookie"))
            .map(|cookie| get_split_index(cookie, 1)) // "Set-Cookie: foo_bar=baz" -> "foo_bar=baz"
            .collect::<Vec<&str>>();

        if !cookies.is_empty() {
            Some(cookies)
        } else {
            None
        }
    }

    /// `get_cookies` takes care of the `Cookie` header
    pub fn get_cookies(req: &str) -> Option<Vec<&str>> {
        if let Some(cookies) = req.split("\r\n").find(|line| line.contains("Cookie")) {
            return Some(cookies.split(';').collect::<Vec<&str>>());
        }
        None
    }

    /// `get_content_type` gets the `Content-Length` header for state changing methods
    pub fn get_content_length(req: &str) -> Option<&str> {
        if let Some(line) = req.split("\r\n").find(|&l| l.contains("Content-Length")) {
            let content_length = get_split_index(line, 1);
            // "Content-Length: 1337" -> "1337"
            return Some(content_length);
        }
        None
    }

    /// `get_content_type` gets the `Content-Type` header for state changing methods
    pub fn get_content_type(req: &str) -> Option<&str> {
        if let Some(line) = req.split('\n').find(|&l| l.contains("Content-Type")) {
            let content_type = get_split_index(line, 1);
            // "Content-Type: text/html" -> "text/html"
            Some(content_type)
        } else {
            None
        }
    }
}

pub mod body {
    pub fn get_body(req: &str, limit: usize) -> Option<String> {
        let body = req
            .trim_end_matches('\0')
            .split("\r\n\r\n")
            .skip(1)
            .collect::<Vec<&str>>()
            .join("\r\n\r\n");

        if body.len() <= limit {
            Some(body)
        } else {
            None
        }
    }
}

pub mod utils {
    use crate::type_aliases::Bytes;

    /// `get_split_index` gets the `&str` at `index` after performing `split_whitespace`
    pub fn get_split_index(str: &str, index: usize) -> &str {
        let lines = str.split_whitespace().collect::<Vec<&str>>();
        if lines.is_empty() {
            ""
        } else if index > lines.len() {
            lines[0]
        } else {
            lines[index]
        }
    }

    /// `get_line` gets the `&str` at `index` after performing `split('\n')`
    pub fn get_line(str: &str, index: usize) -> &str {
        let lines = str
            .trim_end_matches('\0')
            .split("\r\n")
            .collect::<Vec<&str>>();
        if lines.is_empty() {
            ""
        } else if index > lines.len() {
            lines[0]
        } else {
            lines[index]
        }
    }

    pub fn to_bytes(str: &str) -> Bytes {
        str.as_bytes().to_vec()
    }
}
