use crate::server::{Request, Route, ServerConfig, StatusCode};

pub fn get_request(conf: &ServerConfig, req_str: &str) -> Result<Request<String>, StatusCode> {
    let version = match version::get_version(req_str) {
        Ok(v) => v,
        Err(_) => return Err(StatusCode::HTTP_VERSION_NOT_SUPPORTED),
    };

    let path = path::get_path(req_str);
    let method = match super::get_method(req_str) {
        Ok(method) => method,
        Err(_) => return Err(StatusCode::METHOD_NOT_ALLOWED),
    };

    let headers = headers::get_headers(req_str);
    let is_chunked = headers.iter().any(|header| {
        let parts: Vec<&str> = header.splitn(2, ": ").collect();
        parts.len() == 2
            && parts[0].eq_ignore_ascii_case("transfer-encoding")
            && parts[1].contains("chunked")
    });

    let body_start_index = req_str.find("\r\n\r\n").unwrap_or(req_str.len());
    let body_str = &req_str[body_start_index + 4..]; // "+ 4" to skip past "\r\n\r\n"

    let body = if is_chunked {
        match handle_chunked_body(body_str, conf.body_size_limit) {
            Ok(body) => body,
            Err(status) => return Err(status),
        }
    } else {
        body::get_body(req_str, conf.body_size_limit).unwrap_or_default()
    };

    // Constructing the request with parsed headers and body
    let mut request_builder = http::Request::builder()
        .method(method)
        .uri(path)
        .version(version);

    for header in headers {
        if let Some((key, value)) = headers::format_header(header) {
            request_builder = request_builder.header(key, value);
        }
    }

    match request_builder.body(body) {
        Ok(request) => Ok(request),
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

fn handle_chunked_body(body_str: &str, limit: usize) -> Result<String, StatusCode> {
    let mut body = String::new();
    let mut remaining_str = body_str;

    while !remaining_str.is_empty() {
        // Split at the first occurrence of CRLF
        if let Some((size_str, rest)) = remaining_str.split_once("\r\n") {
            // Parse the chunk size
            let chunk_size = match usize::from_str_radix(size_str.trim(), 16) {
                Ok(size) => size,
                Err(_) => return Err(StatusCode::BAD_REQUEST),
            };

            // Check for the end of the chunked body
            if chunk_size == 0 {
                break;
            }

            // Ensure there's enough data for the chunk
            if rest.len() < chunk_size {
                return Err(StatusCode::BAD_REQUEST);
            }

            // Extract the chunk data
            let (chunk_data, after_chunk) = rest.split_at(chunk_size);
            println!("Chunk data: {}", chunk_data);
            println!("After chunk: {}", after_chunk);
            body.push_str(chunk_data);

            // Check body size limit
            if body.len() > limit {
                return Err(StatusCode::PAYLOAD_TOO_LARGE);
            }

            // Prepare for the next iteration, skip past the chunk data and CRLF
            remaining_str = &after_chunk[2..]; // Assumes CRLF is always present
        } else {
            return Err(StatusCode::BAD_REQUEST); // Missing CRLF after chunk size
        }
    }

    Ok(body)
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
