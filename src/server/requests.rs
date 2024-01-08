use crate::log::log;
use crate::server::{Request, Route, ServerConfig, StatusCode};

pub fn get_request(conf: &ServerConfig, req_str: &str) -> Result<Request<String>, StatusCode> {
    let version = match version::get_version(req_str) {
        Ok(v) => v,
        Err(v) => {
            log("server", format!("Error: Incorrect version '{}'", v));
            return Err(StatusCode::HTTP_VERSION_NOT_SUPPORTED);
        }
    };

    let path = path::get_path(req_str);
    let method = match super::get_method(req_str) {
        Ok(method) => method,
        Err(method) => {
            log(
                "server",
                format!("Error: Method not allowed '{}' on path '{}'", method, path),
            );
            return Err(StatusCode::METHOD_NOT_ALLOWED);
        }
    };

    let mut request = Request::builder().method(method).uri(path).version(version);

    for header in headers::get_headers(req_str) {
        if let Some((key, value)) = headers::format_header(header) {
            request = request.header(key, value);
        }
    }

    let body = body::get_body(req_str, conf.body_size_limit).unwrap_or_default();
    match request.body(body) {
        Ok(request) => Ok(request),
        Err(request) => {
            log("server", format!("Error: Failed to get body {}", request));
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

pub mod path {
    use super::*;
    use crate::server::utils::{get_line, get_split_index};
    use crate::type_aliases::Path;

    /// `path` gets the path from the `request`
    pub fn get_path(req: &str) -> &str {
        let line = get_line(req, 0);
        get_split_index(line, 1)
    }

    /// `path_exists` gets the `path` and the `index` of the `route` it was a part of if found.
    pub fn path_exists<'a>(
        requested_path: Path<'a>,
        routes: &[Route<'a>],
    ) -> Option<(usize, Path<'a>)> {
        for (i, route) in routes.iter().enumerate() {
            if route.path == requested_path {
                return Some((i, route.path));
            }
            if route.settings.is_none() {
                continue;
            }
            if route
                .settings
                .clone()
                .unwrap()
                .http_redirections
                .contains(&requested_path)
            {
                return Some((i, route.path));
            }
        }
        None // Path does not exist in allowed paths or in redirections
    }
}

pub mod version {
    use crate::log::log;
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
            _ => {
                log(
                    "server",
                    format!("Error: Version not supported {}", version_str),
                );
                Err(StatusCode::HTTP_VERSION_NOT_SUPPORTED)
            }
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
